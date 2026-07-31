//! Translator - Dioxus Desktop Application
//!
//! A schema-driven translation app using Dioxus framework.
//! All UI is generated from schemas via `dioxus-shared::DynamicPage`.
//!
//! **KEY PRINCIPLE**: The app has ZERO hardcoded UI pages.
//! All UI is rendered from schema JSON via `DynamicPage`.

use dioxus::prelude::*;
use dioxus_desktop::Config;
use dioxus_desktop::tao;
use dioxus_shared::ui::DynamicPage;
use dioxus_shared::schema::Schema;
use dioxus_shared::get_theme_css;
use dioxus_shared::storage::SignalStore;
use dioxus_shared::AlgorithmRegistry;
use dioxus_shared::themes::{ThemeMode, ThemeVariant};
use dioxus_shared::ui::components::action_bus::{ActionBus, AppAction};
use dioxus_shared::ui::components::ThemeProvider;
use dioxus_plugin_mcp_bridge::{McpBridge, EvalRequest};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use translator::TranslationService;
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
    PathBuf::from(&manifest_dir).join("../schemas/translator.json")
}

fn load_schema() -> Schema {
    let path = get_schema_path();
    if !path.exists() {
        eprintln!("Schema not found at {:?}, falling back to embedded", path);
        let schema_json = include_str!("../../schemas/translator.json");
        serde_json::from_str(schema_json).expect("Invalid embedded schema")
    } else {
        let content = std::fs::read_to_string(&path).expect("Failed to read schema");
        serde_json::from_str(&content).expect("Invalid schema at path")
    }
}

#[derive(Clone, Props)]
struct AppProps {
    schema: Schema,
    bridge_state: Arc<dioxus_plugin_mcp_bridge::BridgeState>,
    store: Arc<SignalStore>,
}

impl PartialEq for AppProps {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.store, &other.store)
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
    let store = use_context::<Arc<SignalStore>>();
    let registry = use_context::<Arc<AlgorithmRegistry>>();

    if current_len != *prev_len.read() {
        let mut bus = props.bus.clone();
        while let Some(action) = bus.pop_action() {
            handle_action(&mut bus, action, store.clone(), registry.clone());
        }
        prev_len.set(current_len);
    }

    if let Some(nav) = props.bus.pop_navigate() {
        props.bus.current_route.write().clone_from(&nav.route);
    }

    rsx! { Fragment {} }
}

fn handle_action(bus: &mut ActionBus, action: AppAction, store: Arc<SignalStore>, registry: Arc<AlgorithmRegistry>) {
    match action.name.as_str() {
        "translate" => {
            let source_text = bus.get_binding("source_text").unwrap_or_default();
            let source_lang = bus.get_binding("source_lang").unwrap_or_else(|| "en".to_string());
            let target_lang = bus.get_binding("target_lang").unwrap_or_else(|| "es".to_string());

            if source_text.trim().is_empty() {
                bus.set_binding("translated_text", "");
                return;
            }

            match TranslationService::translate(&source_text, &source_lang, &target_lang) {
                Ok(result) => {
                    let text = result
                        .data
                        .as_ref()
                        .map(|d| d.translated_text.clone())
                        .unwrap_or_default();
                    bus.set_binding("translated_text", &text);
                    // Record history entry through shared store.
                    let entry_id = format!("tx-{}", chrono::Utc::now().timestamp_millis());
                    let _ = store.set(
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
                let mut storage = GlossaryStorage::new(store.clone(), registry.clone());
                let GlossaryItem { id, .. } = storage.add_item(term.clone(), term_translation.clone());
                bus.set_binding("glossary_last_id", &id);
                bus.clear_binding("term");
                bus.clear_binding("term_translation");
            }
        }
        "save_settings" => {
            let default_source = bus.get_binding("default_source_lang").unwrap_or_default();
            let default_target = bus.get_binding("default_target_lang").unwrap_or_default();
            let theme = if bus.is_dark_mode() { "dark" } else { "light" }.to_string();
            let current = SettingsStorage::new(store.clone()).load_settings();
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
            match SettingsStorage::new(store.clone()).save_settings(&updated) {
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
    provide_context(props.store.clone());

    let bus = ActionBus::new("/translate");
    provide_context(bus.clone());

    let dark_mode = bus.is_dark_mode();
    let current_route = bus.current_route();

    let registry = Arc::new(AlgorithmRegistry::new());
    provide_context(registry.clone());

    let theme_mode = if dark_mode { ThemeMode::Dark } else { ThemeMode::Light };
    let theme_variant = ThemeVariant::MaterialDesign3;

    // Eval loop: poll pending_eval_requests and process via webview.
    // DesktopContext = Rc<DesktopService> is NOT Send — must stay on Dioxus
    // main thread. The loop yields with sleep to avoid hard-blocking the event loop.
    let desktop = dioxus_desktop::use_window();
    let bridge_state = props.bridge_state.clone();
    use_effect(move || {
        let bridge_state = bridge_state.clone();
        let desktop = desktop.clone();
        loop {
            if bridge_state.is_shutdown() {
                break;
            }

            // Process eval requests via webview (stays on main thread)
            let requests = bridge_state.dequeue_eval_requests();
            if !requests.is_empty() {
                for request in requests {
                    let result = process_eval_on_main_thread(&desktop, &request);
                    bridge_state.enqueue_js_result(request.id, result);
                }
            }

            // Yield to event loop briefly
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

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
                        schema: props.schema.clone(),
                        initial_route: current_route,
                        bus: bus.clone()
                    }
                }
            }
        }
    }
}

pub use translator::bridge::{bridge_consumer_loop, invoke_app_command, invoke_ui_action};

/// Process evaluate_js or dom_snapshot request via the webview.
/// Called from use_effect on the Dioxus main thread where DesktopContext is valid.
///
/// Uses `evaluate_script_with_callback` to capture JS return values via a
/// blocking oneshot channel — safe because we're on the main thread and the
/// webview's WRYWebView mutex is unlocked during the eval.
fn process_eval_on_main_thread(
    desktop: &dioxus_desktop::DesktopContext,
    request: &EvalRequest,
) -> String {
    use std::sync::mpsc;
    // DesktopService { pub webview: WebView }
    let webview = &desktop.webview;
    let (tx, rx) = mpsc::channel();

    let script = match request.method.as_str() {
        "evaluate_js" => {
            let code = request
                .params
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            // Wrap in JSON serializer so arbitrary JS values become strings
            format!(
                "(function(){{try{{return JSON.stringify((function(){{return eval({})}}())}}catch(e){{return JSON.stringify({{error:e.toString()}})}})}})()",
                serde_json::Value::String(code.to_string())
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
        _ => return serde_json::json!({ "error": "unknown eval method" }).to_string(),
    };

    let _ = webview.evaluate_script_with_callback(&script, move |result: String| {
        let _ = tx.send(result);
    });

    rx.recv().unwrap_or_else(|_| r#"{"error":"callback never fired"}"#.to_string())
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
    let _history = HistoryStorage::new(store.clone()); // hydrate the store on startup
    let _settings = SettingsStorage::new(store.clone()); // hydrate settings

    let schema = load_schema();

    let desktop_config = Config::new()
        .with_window(tao::window::WindowBuilder::new()
            .with_title("Translator")
            .with_inner_size(tao::dpi::LogicalSize::new(800, 600)));

    let dom = VirtualDom::new_with_props(App, AppProps {
        schema,
        bridge_state: state,
        store,
    });

    use dioxus_desktop::launch::launch_virtual_dom;
    launch_virtual_dom(dom, desktop_config);
}

