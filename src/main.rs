//! Translator - Dioxus Desktop Application
//!
//! A schema-driven translation app using Dioxus framework.
//! All UI is generated from schemas via `dioxus-shared::DynamicPage`.
//!
//! **KEY PRINCIPLE**: The app has ZERO hardcoded UI pages.
//! All UI is rendered from schema JSON via `DynamicPage`.

use dioxus::prelude::*;
use dioxus_desktop::tao;
use dioxus_desktop::Config;
use dioxus_shared::get_theme_css;
use dioxus_shared::mcp::bridge::{EvalRequest, McpBridge};
use dioxus_shared::schema::AppSchema;
use dioxus_shared::storage::SignalStore;
use dioxus_shared::themes::{ThemeMode, ThemeVariant};
use dioxus_shared::ui::components::action_bus::{ActionBus, AppAction};
use dioxus_shared::ui::components::ThemeProvider;
use dioxus_shared::ui::DynamicPage;
use dioxus_shared::AlgorithmRegistry;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use translator::application::AppState;
use translator::domain::{AppSettings, GlossaryItem, GlossaryService, SettingsService};
use translator::infrastructure::{GlossaryStorage, HistoryStorage, SettingsStorage};

fn get_schema_path() -> PathBuf {
    // Priority 1: runtime symlink ~/.local/share/com.tcs.translator/schema.json
    //             -> /home/dmitriy/Projects/schemas/translator.json
    if let Ok(home) = std::env::var("HOME") {
        let runtime = PathBuf::from(&home).join(".local/share/com.tcs.translator/schema.json");
        if runtime.exists() {
            return runtime;
        }
    }
    // Priority 2: canonical schema in repo (for dev without runtime install)
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(&manifest_dir).join("schemas/translator.json")
}

fn load_schema() -> AppSchema {
    let path = get_schema_path();
    if !path.exists() {
        eprintln!("Schema not found at {:?}, falling back to embedded", path);
        let schema_json = include_str!("../schemas/translator.json");
        serde_json::from_str(schema_json).expect("Invalid embedded schema")
    } else {
        let content = std::fs::read_to_string(&path).expect("Failed to read schema");
        serde_json::from_str(&content).expect("Invalid schema at path")
    }
}

#[derive(Clone, Props)]
struct AppProps {
    schema: AppSchema,
    bridge_state: Arc<dioxus_shared::mcp::bridge::BridgeState>,
    app_state: Arc<AppState>,
}

impl PartialEq for AppProps {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.app_state, &other.app_state)
            && self.schema == other.schema
            && Arc::ptr_eq(&self.bridge_state, &other.bridge_state)
    }
}

#[derive(Clone, Props, PartialEq)]
struct ActionProcessorProps {
    bus: ActionBus,
}

#[component]
fn ActionProcessor(mut props: ActionProcessorProps) -> Element {
    let mut prev_len = use_signal(|| 0usize);
    let current_len = props.bus.dispatch.read().len();
    let app_state = use_context::<Arc<AppState>>();

    if current_len != *prev_len.read() {
        let mut bus = props.bus.clone();
        while let Some(action) = bus.pop_action() {
            handle_action(&mut bus, action, app_state.clone());
        }
        prev_len.set(current_len);
    }

    if let Some(nav) = props.bus.pop_navigate() {
        props.bus.current_route.write().clone_from(&nav.route);
    }

    rsx! { Fragment {} }
}

fn handle_action(
    bus: &mut ActionBus,
    action: AppAction,
    app_state: Arc<AppState>,
) {
    match action.name.as_str() {
        "translate" => {
            let source_text = bus.get_binding("source_text").unwrap_or_default();
            let source_lang = bus
                .get_binding("source_lang")
                .unwrap_or_else(|| "en".to_string());
            let target_lang = bus
                .get_binding("target_lang")
                .unwrap_or_else(|| "es".to_string());

            if source_text.trim().is_empty() {
                bus.set_binding("translated_text", "");
                return;
            }

            match app_state.translation_service.lock().unwrap().translate(&source_text, &source_lang, &target_lang) {
                Ok(result) => {
                    let text = result
                        .data
                        .as_ref()
                        .map(|d| d.translated_text.clone())
                        .unwrap_or_default();
                    bus.set_binding("translated_text", &text);
                    // Record history entry through shared store.
                    let entry_id = format!("tx-{}", chrono::Utc::now().timestamp_millis());
                    app_state.signal_store.set(
                        "history.last_entry",
                        serde_json::json!({
                            "id": entry_id,
                            "query": source_text,
                            "result": text,
                            "source_lang": source_lang,
                            "target_lang": target_lang,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        }),
                    );
                }
                Err(e) => {
                    bus.set_binding("translated_text", &format!("Error: {}", e));
                }
            }
        }
        "add_term" => {
            let term = bus.get_binding("term").unwrap_or_default();
            let term_translation = bus.get_binding("term_translation").unwrap_or_default();
            if !term.is_empty() && !term_translation.is_empty() {
                let storage = GlossaryStorage::new(
                    app_state.signal_store.clone(),
                    app_state.algorithm_registry.clone(),
                );
                let item = storage.add_item(term.clone(), term_translation.clone());
                bus.set_binding("glossary_last_id", &item.id);
                bus.clear_binding("term");
                bus.clear_binding("term_translation");
            }
        }
        "save_settings" => {
            let default_source = bus.get_binding("default_source_lang").unwrap_or_default();
            let default_target = bus.get_binding("default_target_lang").unwrap_or_default();
            let theme = if bus.is_dark_mode() { "dark" } else { "light" }.to_string();
            let current = SettingsStorage::new(app_state.signal_store.clone()).load_settings();
            let settings = AppSettings {
                default_source_lang: if default_source.is_empty() {
                    current.default_source_lang
                } else {
                    default_source
                },
                default_target_lang: if default_target.is_empty() {
                    current.default_target_lang
                } else {
                    default_target
                },
                ..current
            };
            let _ = settings.theme; // silence field-order warning
            let updated = AppSettings { theme, ..settings };
            match SettingsStorage::new(app_state.signal_store.clone()).save_settings(&updated) {
                Ok(()) => bus.set_binding("settings_status", "saved"),
                Err(e) => bus.set_binding("settings_status", &format!("Error: {e}")),
            }
        }
        "toggle_theme" => {
            bus.toggle_theme();
        }
        "swap_languages" => {
            let source = bus.get_binding("source_lang").unwrap_or_default();
            let target = bus.get_binding("target_lang").unwrap_or_default();
            if !source.is_empty() && !target.is_empty() {
                bus.set_binding("source_lang", &target);
                bus.set_binding("target_lang", &source);
            }
        }
        "show-shortcuts" => {
            bus.open_modal("shortcuts-modal");
        }
        "close" => {
            bus.close_modal();
        }
        "clear_text" => {
            bus.clear_binding("source_text");
            bus.clear_binding("translated_text");
        }
        "copy_result" => {
            if let Some(text) = bus.get_binding("translated_text") {
                if !text.is_empty() {
                    eprintln!("Copy to clipboard requested: {text}");
                }
            }
        }
        _ => {
            eprintln!("Unknown action: {} from {}", action.name, action.source);
        }
    }
}

#[component]
fn App(props: AppProps) -> Element {
    provide_context(props.bridge_state.clone());
    provide_context(props.app_state.clone());

    let bus = ActionBus::new("/");
    provide_context(bus.clone());

    let dark_mode = bus.is_dark_mode();
    let current_route = bus.current_route();

    let theme_mode = if dark_mode {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    let theme_variant = ThemeVariant::MaterialDesign3;

    // Eval bridge: MCP evaluate_js/dom_snapshot must run on the main thread
    // (DesktopContext is !Send). A background watcher polls the bridge queue
    // and bumps `eval_wake` only when work arrives; the effect below then does
    // the webview work. The UI thread never busy-polls — a previous 1ms
    // self-toggling effect here froze rendering and left the window blank.
    let desktop = dioxus_desktop::use_window();
    let bridge_state = props.bridge_state.clone();
    let bridge_state_for_eval = props.bridge_state.clone();
    let mut eval_wake = use_signal_sync(|| 0u64);

    // Spawn-once: background watcher thread (allowed to poll at 25ms).
    use_effect(move || {
        let bridge_state = bridge_state.clone();
        let mut eval_wake = eval_wake;
        std::thread::spawn(move || {
            while !bridge_state.is_shutdown() {
                if bridge_state.pending_eval_count() > 0 {
                    let current = *eval_wake.read();
                    eval_wake.set(current + 1);
                }
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
        });
    });

    use_effect(move || {
        let _tick = eval_wake();
        let requests = bridge_state_for_eval.dequeue_eval_requests();
        if !requests.is_empty() {
            dispatch_eval_requests(&desktop, requests, bridge_state_for_eval.clone());
        }
    });


    // Declarative data sources: pages declare `data_sources` + element
    // `data_binding`; providers registered here resolve them at load time.
    let mut registry = dioxus_shared::ui::data_registry::DataRegistry::new();
    {
        let store = props.app_state.signal_store.clone();
        registry.register(
            "history_recent",
            std::sync::Arc::new(move |_params| {
                let store = store.clone();
                Box::pin(async move {
                    Ok(store
                        .get("history.last_entry")
                        .unwrap_or(serde_json::json!({})))
                })
            }),
        );
    }

    rsx! {
        style { {get_theme_css()} }
        ThemeProvider {
            initial_mode: theme_mode,
            initial_variant: theme_variant,
            ActionProcessor { bus: bus.clone() }
            div {
                class: if dark_mode { "dark" } else { "" },
                div {
                    class: "min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors",
                    DynamicPage {
                        schema: props.schema.clone().into(),
                        initial_route: current_route,
                        bus: bus.clone(),
                        registry: Some(registry),
                    }
                }
            }
        }
    }
}

pub use translator::bridge::{bridge_consumer_loop, invoke_app_command, invoke_ui_action};

/// Dispatch evaluate_js/dom_snapshot requests through the webview.
///
/// Called from use_effect on the Dioxus main thread where DesktopContext is
/// valid. MUST NOT block: wry delivers the evaluation callback on the main
/// thread, so waiting here for it deadlocks. The callback posts the result
/// straight into the bridge queue; the consumer loop turns it into the MCP
/// response.
fn dispatch_eval_requests(
    desktop: &dioxus_desktop::DesktopContext,
    requests: Vec<EvalRequest>,
    bridge_state: std::sync::Arc<dioxus_shared::mcp::bridge::BridgeState>,
) {
    use std::sync::Arc;
    let webview = &desktop.webview;

    for request in requests {
        let id = request.id.clone();
        let script = match request.method.as_str() {
            "evaluate_js" => {
                let code = request
                    .params
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                // Insert the user code directly (it was eval'd anyway); wrap
                // the result in JSON.stringify inside a single-element array —
                // webkitgtk's JSCValue::to_json drops bare primitives.
                format!(
                    "(function(){{try{{return [JSON.stringify((function(){{return ({code})}}()))]}}catch(e){{return [JSON.stringify({{error:e.toString()}})]}}}})()",
                    code = code
                )
            }
            "dom_snapshot" => {
                let selector = request.params.get("selector").and_then(|v| v.as_str());
                if let Some(sel) = selector {
                    let escaped = sel.replace('\\', "\\\\").replace('\'', "\\'");
                    format!(
                        "(function(){{var el=document.querySelector('{}');return el?el.outerHTML:null}})()",
                        escaped
                    )
                } else {
                    "(function(){return document.body.innerHTML})()".to_string()
                }
            }
            _ => {
                bridge_state.enqueue_js_result(
                    id,
                    serde_json::json!({ "error": "unknown eval method" }).to_string(),
                );
                continue;
            }
        };

        let callback_id = id.clone();
        let callback_state = bridge_state.clone();
        if let Err(e) =
            webview.evaluate_script_with_callback(&script, move |result: String| {
                // Unwrap the single-element array used for webkit serialization.
                let unwrapped = serde_json::from_str::<Vec<String>>(&result)
                    .ok()
                    .and_then(|v| v.into_iter().next())
                    .unwrap_or(result);
                callback_state.enqueue_js_result(callback_id.clone(), unwrapped);
            })
        {
            eprintln!("evaluate_script failed: {e}");
        }
    }
}

fn main() {
    let (bridge, state) = McpBridge::new(9223);

    thread::spawn(move || {
        println!("MCP Bridge listening on ws://127.0.0.1:9223");
        bridge.run();
    });

    // Drain the bridge queue in a worker thread so MCP `ui.snapshot`,
    // `evaluate_js`, and `screenshot` no longer time out. The consumer
    // responds with explanatory payloads; full webview evaluation will be
    // wired once the Dioxus 0.8-alpha API is stable.
    let consumer_state = state.clone();
    thread::spawn(move || bridge_consumer_loop(consumer_state));

    let store = Arc::new(SignalStore::new());
    let registry = Arc::new(AlgorithmRegistry::new());
    let _history = HistoryStorage::new(store.clone()); // hydrate the store on startup
    let _settings = SettingsStorage::new(store.clone()); // hydrate settings

    // Create AppState with all services wired
    let app_state = Arc::new(AppState::new(store, registry));

    let schema = load_schema();

    let desktop_config = Config::new().with_window(
        tao::window::WindowBuilder::new()
            .with_title("Translator")
            .with_inner_size(tao::dpi::LogicalSize::new(800, 600)),
    );

    let dom = VirtualDom::new_with_props(
        App,
        AppProps {
            schema,
            bridge_state: state,
            app_state,
        },
    );

    use dioxus_desktop::launch::launch_virtual_dom;
    launch_virtual_dom(dom, desktop_config);
}
