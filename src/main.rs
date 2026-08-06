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
use dioxus_shared::schema::{CanvasElement, Schema};
use dioxus_shared::storage::SignalStore;
use dioxus_shared::themes::{ThemeMode, ThemeVariant};
use dioxus_shared::ui::components::action_bus::{ActionBus, AppAction};
use dioxus_shared::ui::components::ThemeProvider;
use dioxus_shared::ui::DynamicPage;
use dioxus_shared::AlgorithmRegistry;
use dioxus_shared::I18nStore;
use dioxus_shared::{log_debug, log_error, log_info, log_warn};
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use translator::domain::{AppSettings, SettingsService};
use translator::infrastructure::translation::detect_language;
use translator::infrastructure::SettingsStorage;
use translator::TranslationService;

/// Canonical schema source (repo) used to seed the JSON DB on first launch.
fn get_canonical_schema_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(&manifest_dir).join("../schemas/translator.json")
}

/// Load the schema via `nosql_orm` (through `dioxus-shared`). All schema reads
/// go through the JSON DB; the repo file is only used to seed the DB once.
fn load_schema() -> Schema {
    use dioxus_shared::storage::{create_json_provider, load_schema_from_db, save_schema_to_db};

    let data_dir = translator::infrastructure::get_settings_path()
        .parent()
        .map(|p| PathBuf::from(p))
        .unwrap_or_else(|| PathBuf::from(".translator-data"));
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    let provider = rt
        .block_on(create_json_provider(&data_dir))
        .expect("Failed to create JsonProvider");

    // Priority 1: schema already in the JSON DB.
    if let Ok(Some(doc)) = rt.block_on(load_schema_from_db(&provider, "translator")) {
        return serde_json::from_value(doc).expect("Invalid schema in JSON DB");
    }

    // Priority 2: seed the DB from the canonical repo schema, then read back.
    let canonical = get_canonical_schema_path();
    let content = std::fs::read_to_string(&canonical).expect("Failed to read canonical schema");
    let value: serde_json::Value =
        serde_json::from_str(&content).expect("Invalid canonical schema");

    rt.block_on(save_schema_to_db(&provider, "translator", &value))
        .expect("Failed to seed schema into JSON DB");

    serde_json::from_value(value).expect("Invalid seeded schema")
}

#[derive(Clone, Props)]
struct AppProps {
    schema: Schema,
    bridge_state: Arc<dioxus_shared::mcp::bridge::BridgeState>,
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
    schema: Schema,
}

#[component]
fn ActionProcessor(mut props: ActionProcessorProps) -> Element {
    let mut prev_len = use_signal(|| 0usize);
    let current_len = props.bus.dispatch.read().len();
    let store = use_context::<Arc<SignalStore>>();
    let registry = use_context::<Arc<AlgorithmRegistry>>();
    let bridge_state = use_context::<Arc<dioxus_shared::mcp::bridge::BridgeState>>();

    if current_len != *prev_len.read() {
        let mut bus = props.bus.clone();
        while let Some(action) = bus.pop_action() {
            log_info!("[action] {} from {}", action.name, action.source);
            handle_action(&mut bus, action, store.clone());
        }
        prev_len.set(current_len);
    }

    if let Some(nav) = props.bus.pop_navigate() {
        log_info!("[nav] {} (params: {:?})", nav.route, nav.params);
        props.bus.current_route.write().clone_from(&nav.route);
    }

    // Poll pending UI action requests from MCP bridge and dispatch via ActionBus.
    // This runs on the Dioxus main thread so ActionBus (!Send) is safe to use.
    let bus_for_effect = props.bus.clone();
    let store_for_effect = store.clone();
    let _registry_for_effect = registry.clone();
    let bridge_state_for_effect = bridge_state.clone();
    use_effect(move || {
        // Clone Arcs here so the async block takes ownership of the clones.
        let bus = bus_for_effect.clone();
        let store = store_for_effect.clone();
        let bridge_state = bridge_state_for_effect.clone();
        spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
            loop {
                interval.tick().await;

                let requests = bridge_state.dequeue_ui_action_requests();
                if requests.is_empty() {
                    continue;
                }

                let mut bus = bus.clone();

                for req in requests {
                    log_info!("[mcp] action={} payload={}", req.action, req.payload);
                    let action = dioxus_shared::ui::components::action_bus::AppAction {
                        name: req.action.clone(),
                        source: "mcp".to_string(),
                        payload: Some(req.payload.clone()),
                    };
                    handle_action(&mut bus, action, store.clone());

                    bridge_state.set_response(
                        req.id,
                        dioxus_shared::mcp::bridge::Response {
                            result: Some(serde_json::json!({ "ok": true })),
                            error: None,
                        },
                    );
                }
            }
        });
    });

    // A ticker signal drives a main-thread effect that drains page_snapshot
    // requests and returns page structure. Tick-based polling replaces the
    // unreliable spawned async loop approach.
    let page_snapshot_tick = use_signal(|| 0u32);
    let bridge_state_for_snapshot = bridge_state.clone();
    use_effect(move || {
        let mut tick = page_snapshot_tick;
        spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                tick.set(tick() + 1);
            }
        });
    });
    let schema_for_snapshot = props.schema.clone();
    let bus_for_snapshot = props.bus.clone();
    use_effect(move || {
        let _ = page_snapshot_tick(); // reactive dependency: re-run on each tick
        let bridge_state = bridge_state_for_snapshot.clone();
        let schema = schema_for_snapshot.clone();
        let bus = bus_for_snapshot.clone();
        for req in bridge_state.dequeue_page_snapshot_requests() {
            let current_route = bus.current_route();
            let page = schema
                .pages
                .iter()
                .find(|p| p.route == current_route)
                .or_else(|| schema.pages.first());

            let snapshot = serde_json::json!({
                "route": current_route,
                "pages": schema.pages.iter().map(|p| {
                    serde_json::json!({
                        "id": p.id,
                        "title": p.title,
                        "route": p.route,
                        "layout": p.layout,
                        "elements": serialize_elements(&p.elements),
                    })
                }).collect::<Vec<_>>(),
                "current_page": page.map(|p| {
                    serde_json::json!({
                        "id": p.id,
                        "title": p.title,
                        "route": p.route,
                        "layout": p.layout,
                        "elements": serialize_elements(&p.elements),
                    })
                }),
                "bindings": bus.bindings.read().clone(),
                "theme": if bus.is_dark_mode() { "dark" } else { "light" },
            });

            bridge_state.set_response(
                req.id,
                dioxus_shared::mcp::bridge::Response {
                    result: Some(snapshot),
                    error: None,
                },
            );
        }
    });

    // NOTE: evaluate_js and dom_snapshot are routed to pending_eval_requests by
    // bridge_consumer_loop. They must be processed on the Dioxus main thread
    // because DesktopService (!Send) cannot be moved into a spawned task.
    // A ticker signal drives a main-thread effect that drains eval requests,
    // evaluates via the webview, and enqueues results for the bridge consumer.
    let desktop_for_eval = dioxus_desktop::use_window();
    let eval_tick = use_signal(|| 0u32);
    let bridge_state_for_eval = bridge_state.clone();
    use_effect(move || {
        let mut tick = eval_tick;
        spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                tick.set(tick() + 1);
            }
        });
    });
    use_effect(move || {
        let _ = eval_tick(); // reactive dependency: re-run on each tick
        let desktop = desktop_for_eval.clone();
        let bridge_state = bridge_state_for_eval.clone();
        for req in bridge_state.dequeue_eval_requests() {
            log_info!("[mcp] eval method={}", req.method);
            let result = process_eval_on_main_thread(&desktop, &req);
            bridge_state.enqueue_js_result(req.id, result);
        }
    });

    rsx! { Fragment {} }
}

/// Recursively serialize a list of CanvasElements to JSON-friendly structure
fn serialize_elements(elements: &[CanvasElement]) -> Vec<serde_json::Value> {
    elements
        .iter()
        .map(|el| {
            serde_json::json!({
                "id": el.id,
                "component": el.component,
                "classes": el.classes,
                "visible": el.visible,
                "props": el.props,
                "binding": el.data_binding.as_ref().map(|b| b.field.clone()),
                "children": serialize_elements(&el.children),
            })
        })
        .collect()
}

fn handle_action(bus: &mut ActionBus, action: AppAction, store: Arc<SignalStore>) {
    match action.name.as_str() {
        "translate" => {
            let source_text = bus.get_binding("source_text").unwrap_or_default();
            let source_lang_raw = bus
                .get_binding("source_lang")
                .unwrap_or_else(|| "en".to_string());
            let target_lang = bus
                .get_binding("target_lang")
                .unwrap_or_else(|| "es".to_string());

            let final_source_lang = if source_lang_raw == "auto" {
                if source_text.is_empty() {
                    "ru".to_string()
                } else {
                    detect_language(&source_text)
                }
            } else {
                source_lang_raw
            };

            if source_text.trim().is_empty() {
                log_warn!("[translate] empty source text");
                bus.set_binding("translated_text", "");
                return;
            }

            match TranslationService::translate(&source_text, &final_source_lang, &target_lang) {
                Ok(result) => {
                    let text = result
                        .data
                        .as_ref()
                        .map(|d| d.translated_text.clone())
                        .unwrap_or_default();
                    bus.set_binding("translated_text", &text);
                    log_info!(
                        "[translate] {} -> {}: {} chars",
                        final_source_lang,
                        target_lang,
                        text.len()
                    );
                    // Record history entry through shared store.
                    let entry_id = format!("tx-{}", chrono::Utc::now().timestamp_millis());
                    store.set(
                        "history.last_entry",
                        serde_json::json!({
                            "id": entry_id,
                            "query": source_text,
                            "result": text,
                            "source_lang": final_source_lang,
                            "target_lang": target_lang,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        }),
                    );
                }
                Err(e) => {
                    log_error!("[translate] error: {}", e);
                    bus.set_binding("translated_text", &format!("Error: {}", e));
                }
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
                Ok(()) => {
                    log_info!("[settings] saved");
                    bus.set_binding("settings_status", "saved");
                }
                Err(e) => {
                    log_error!("[settings] save error: {}", e);
                    bus.set_binding("settings_status", &format!("Error: {e}"));
                }
            }
        }
        "toggle_theme" => {
            bus.toggle_theme();
            let new_theme = if bus.is_dark_mode() { "dark" } else { "light" };
            let current = store
                .get("settings.value")
                .and_then(|value| serde_json::from_value::<AppSettings>(value).ok())
                .unwrap_or_default();
            if let Ok(value) = serde_json::to_value(&current) {
                store.set("settings.value", value);
            }
            log_info!("[theme] toggled to {}", new_theme);
        }
        "swap_languages" => {
            let source = bus.get_binding("source_lang").unwrap_or_default();
            let target = bus.get_binding("target_lang").unwrap_or_default();
            if !source.is_empty() && !target.is_empty() {
                log_info!("[translate] swap languages: {} <-> {}", source, target);
                bus.set_binding("source_lang", &target);
                bus.set_binding("target_lang", &source);
            }
        }
        "show-shortcuts" => {
            log_info!("[ui] open modal: shortcuts-modal");
            bus.open_modal("shortcuts-modal");
        }
        "close" => {
            log_info!("[ui] close modal");
            bus.close_modal();
        }
        "clear_text" => {
            log_debug!("[ui] clear text");
            bus.clear_binding("source_text");
            bus.clear_binding("translated_text");
        }
        "copy_result" => {
            if let Some(text) = bus.get_binding("translated_text") {
                if !text.is_empty() {
                    let len = text.len();
                    match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(text)) {
                        Ok(()) => log_info!("[clipboard] copied {} chars", len),
                        Err(e) => log_error!("[clipboard] copy error: {}", e),
                    }
                }
            }
        }
        _ => {
            log_warn!(
                "[action] unknown action: {} from {}",
                action.name,
                action.source
            );
        }
    }
}

#[component]
fn App(props: AppProps) -> Element {
    provide_context(props.bridge_state.clone());
    provide_context(props.store.clone());

    let bus = ActionBus::new("/");
    provide_context(bus.clone());

    let dark_mode = bus.is_dark_mode();

    let registry = Arc::new(AlgorithmRegistry::new());
    provide_context(registry.clone());

    let i18n_store = Arc::new(I18nStore::new());
    provide_context(i18n_store.clone());

    // TODO: Load translations from file using i18n_store.load_from_dir_sync(path) or load_from_file(path).
    // Note: load_from_file() is async (requires tokio runtime) and cannot be called in this synchronous context.
    // Translation file path not yet defined - needs integration with app's asset/schema loading.

    // layout_variant signal for DynamicPage element filtering (provided as Signal<String>)
    let layout_variant = use_signal(|| "all".to_string());
    provide_context(layout_variant);

    let theme_mode = if dark_mode {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    let theme_variant = props
        .store
        .get("settings.value")
        .and_then(|v| serde_json::from_value::<AppSettings>(v).ok())
        .map(|s| ThemeVariant::from_name(&s.theme_variant))
        .unwrap_or(ThemeVariant::MaterialDesign3);

    // Note: dioxus::spawn with tokio::time::sleep does NOT yield properly in Dioxus 0.8
    // executor (not a tokio runtime). ActionBus and DesktopContext are !Send so
    // tokio::spawn cannot be used. UI action dispatch deferred to ActionProcessor component.
    let _bridge_state = props.bridge_state.clone();
    let bus_for_children = bus.clone();

    // Log app startup only once using a static flag
    use std::sync::atomic::{AtomicBool, Ordering};
    static STARTED: AtomicBool = AtomicBool::new(false);
    if !STARTED.swap(true, Ordering::Relaxed) {
        log_info!("[app] Translator started on route /translate");
    }

    rsx! {
        ThemeProvider {
            initial_mode: theme_mode,
            initial_variant: theme_variant,
            ActionProcessor { bus: bus_for_children.clone(), schema: props.schema.clone() }
            DynamicPage {
                schema: props.schema.clone(),
                initial_route: "/".to_string(),
                bus: bus.clone(),
            }
        }
    }
}

pub use translator::bridge::{bridge_consumer_loop, invoke_app_command, invoke_ui_action};

fn process_eval_on_main_thread(
    desktop: &dioxus_desktop::DesktopContext,
    request: &EvalRequest,
) -> String {
    let webview = &desktop.webview;

    let script = match request.method.as_str() {
        "evaluate_js" => {
            let code = request
                .payload
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            format!(
                "(function(){{try{{return JSON.stringify((function(){{return eval({})}}())}}catch(e){{return JSON.stringify({{error:e.toString()}})}})}})()",
                serde_json::Value::String(code.to_string())
            )
        }
        "dom_snapshot" => {
            let selector = request.payload.get("selector").and_then(|v| v.as_str());
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

    type WaitPair = (Mutex<Option<String>>, Condvar);
    let result: Arc<WaitPair> = Arc::new((Mutex::new(None::<String>), Condvar::new()));
    let result2 = Arc::clone(&result);
    let script_for_cb = script.clone();
    match webview.evaluate_script_with_callback(&script_for_cb, move |result: String| {
        let (lock, cvar): &WaitPair = &result2;
        let mut data = lock.lock().unwrap();
        *data = Some(result);
        cvar.notify_one();
    }) {
        Ok(()) => {
            let (lock, cvar): &WaitPair = &result;
            let data = lock.lock().unwrap();
            let (mut_data, timeout_result) = cvar
                .wait_timeout(data, std::time::Duration::from_secs(5))
                .unwrap();
            if let Some(v) = &*mut_data {
                v.clone()
            } else if timeout_result.timed_out() {
                serde_json::json!({ "error": "callback never fired" }).to_string()
            } else {
                serde_json::json!({ "error": "unknown state" }).to_string()
            }
        }
        Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
    }
}

fn main() {
    let (bridge, state) = McpBridge::new(9223);
    let listener = match bridge.bind() {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("MCP Bridge cannot bind exactly to 127.0.0.1:9223: {error}");
            std::process::exit(1);
        }
    };

    thread::spawn(move || {
        println!("MCP Bridge listening on ws://127.0.0.1:9223");
        if let Err(error) = bridge.run_with_listener(listener) {
            eprintln!("MCP Bridge stopped: {error}");
        }
    });

    // Drain the bridge queue in a worker thread so MCP `ui.snapshot`,
    // `evaluate_js`, and `screenshot` no longer time out. The consumer
    // responds with explanatory payloads; full webview evaluation will be
    // wired once the Dioxus 0.8-alpha API is stable.
    let consumer_state = state.clone();
    thread::spawn(move || bridge_consumer_loop(consumer_state));

    let store = Arc::new(SignalStore::new());
    let _settings = SettingsStorage::new(store.clone()); // hydrate settings

    let schema = load_schema();

    let desktop_config = Config::new()
        .with_window(tao::window::WindowBuilder::new()
            .with_title("Translator")
            .with_inner_size(tao::dpi::LogicalSize::new(800, 600)))
        // Inject theme CSS into <head> via custom head (not body <style> tag)
        // This ensures CSS is parsed before body content renders
        .with_custom_head(format!(
            r#"<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Rounded:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200&display=swap" rel="stylesheet">
<style>{}</style>"#,
            get_theme_css()
        ));

    let dom = VirtualDom::new_with_props(
        App,
        AppProps {
            schema,
            bridge_state: state,
            store,
        },
    );

    use dioxus_desktop::launch::launch_virtual_dom;
    launch_virtual_dom(dom, desktop_config);
}
