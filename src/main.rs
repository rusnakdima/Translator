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
use dioxus_shared::mcp::bridge::McpBridge;
use dioxus_shared::mcp::bridge::{
    generate_css_audit_js, process_computed_styles_sync, process_dom_query_sync,
    process_event_simulate_sync,
};
use dioxus_shared::schema::{CanvasElement, Schema};
use dioxus_shared::storage::SignalStore;
use dioxus_shared::themes::{ThemeMode, ThemeVariant};
use dioxus_shared::ui::components::action_bus::{ActionBus, AppAction};
use dioxus_shared::ui::components::ThemeProvider;
use dioxus_shared::ui::DynamicPage;
use dioxus_shared::AlgorithmRegistry;
use dioxus_shared::I18nStore;
use dioxus_shared::{log_debug, log_error, log_info, log_warn};
use dioxus_shared::logger::Logger;
use std::path::PathBuf;
use std::sync::Arc;
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

/// Requests bridge shutdown when the Dioxus component owning the processor is
/// torn down. Clear ownership: the app's processor is the single owner of the
/// shutdown signal; the bridge accept loop, WebSocket handlers, and the
/// `bridge_consumer_loop` thread all observe `BridgeState::is_shutdown()` and
/// exit cleanly (no leaked tasks/processes).
#[derive(Clone)]
struct BridgeShutdownOnDrop(Arc<dioxus_shared::mcp::bridge::BridgeState>);

impl Drop for BridgeShutdownOnDrop {
    fn drop(&mut self) {
        log_info!("[bridge] processor teardown: requesting bridge shutdown");
        self.0.request_shutdown();
    }
}

#[component]
fn ActionProcessor(mut props: ActionProcessorProps) -> Element {
    let mut prev_len = use_signal(|| 0usize);
    let current_len = props.bus.dispatch.read().len();
    let store = use_context::<Arc<SignalStore>>();
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

    // Single Dioxus-executor bridge processor: `spawn_forever` keeps the
    // ActionBus, DesktopContext, and signals on the Dioxus executor (!Send —
    // never tokio::spawn / std threads); every queue drains here and each
    // response is set exactly once via `BridgeState::set_response`.
    let desktop = dioxus_desktop::use_window();
    let bus_for_loop = props.bus.clone();
    let store_for_loop = store.clone();
    let schema_for_loop = props.schema.clone();
    let bridge_state_for_loop = bridge_state.clone();
    use_hook(move || {
        let processor_bridge_state = bridge_state_for_loop.clone();
        dioxus_core::spawn_forever(async move {
            let bridge_state = processor_bridge_state;
            let mut bus = bus_for_loop;
            let store = store_for_loop;
            let schema = schema_for_loop;
            log_info!("[bridge] dioxus-executor bridge processor started");
            // Wait for the webview to finish initializing.
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            loop {
                if bridge_state.is_shutdown() {
                    log_info!("[bridge] dioxus-executor bridge processor stopping");
                    break;
                }
                for req in bridge_state.dequeue_ui_action_requests() {
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

                for req in bridge_state.dequeue_command_invoke_requests() {
                    if req.name == "translator.translate" {
                        log_info!("[mcp] command {} id={}", req.name, req.id);
                        let text = req
                            .payload
                            .get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let source = req
                            .payload
                            .get("source_lang")
                            .and_then(|v| v.as_str())
                            .unwrap_or("en");
                        let target = req
                            .payload
                            .get("target_lang")
                            .and_then(|v| v.as_str())
                            .unwrap_or("es");
                        let (value, error) = match TranslationService::translate(text, source, target)
                        {
                            Ok(response) => {
                                let translated_text = response
                                    .data
                                    .as_ref()
                                    .map(|d| d.translated_text.clone())
                                    .unwrap_or_default();
                                bus.set_binding("translated_text", &translated_text);
                                let entry_id =
                                    format!("tx-{}", chrono::Utc::now().timestamp_millis());
                                store.set(
                                    "history.last_entry",
                                    serde_json::json!({
                                        "id": entry_id,
                                        "query": text,
                                        "result": translated_text,
                                        "source_lang": source,
                                        "target_lang": target,
                                        "timestamp": chrono::Utc::now().to_rfc3339(),
                                    }),
                                );
                                (
                                    Some(serde_json::json!({
                                        "translated_text": translated_text,
                                        "source_lang": source,
                                        "target_lang": target,
                                        "status": response.status,
                                        "message": response.message,
                                    })),
                                    None,
                                )
                            }
                            Err(e) => (None, Some(e.to_string())),
                        };
                        bridge_state.set_response(
                            req.id,
                            dioxus_shared::mcp::bridge::Response {
                                result: value,
                                error,
                            },
                        );
                    } else {
                        match invoke_app_command(&req.name, &req.payload, &bridge_state) {
                            Ok(value) => bridge_state.set_response(
                                req.id,
                                dioxus_shared::mcp::bridge::Response {
                                    result: Some(value),
                                    error: None,
                                },
                            ),
                            Err(e) => bridge_state.set_response(
                                req.id,
                                dioxus_shared::mcp::bridge::Response {
                                    result: None,
                                    error: Some(e.to_string()),
                                },
                            ),
                        }
                    }
                }

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
                        "binding_values": binding_values(&bus),
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

                for req in bridge_state.dequeue_schema_requests() {
                    log_info!("[mcp] get_schema id={}", req.id);
                    let mut schema_value =
                        serde_json::to_value(&schema).unwrap_or(serde_json::json!({}));
                    if let Some(obj) = schema_value.as_object_mut() {
                        obj.insert("route".to_string(), serde_json::json!(bus.current_route()));
                        obj.insert(
                            "bindings".to_string(),
                            serde_json::json!(bus.bindings.read().clone()),
                        );
                        obj.insert("binding_values".to_string(), binding_values(&bus));
                    }
                    bridge_state.set_response(
                        req.id,
                        dioxus_shared::mcp::bridge::Response {
                            result: Some(schema_value),
                            error: None,
                        },
                    );
                }

                for req in bridge_state.dequeue_component_tree_requests() {
                    log_info!("[mcp] component_tree id={}", req.id);
                    let current_route = bus.current_route();
                    let bindings = bus.bindings.read().clone();
                    let page = schema
                        .pages
                        .iter()
                        .find(|p| p.route == current_route)
                        .or_else(|| schema.pages.first());
                    let tree = serde_json::json!({
                        "route": current_route,
                        "page_id": page.as_ref().map(|p| p.id.as_str()),
                        "components": page
                            .as_ref()
                            .map(|p| serialize_tree_with_values(&p.elements, &bindings))
                            .unwrap_or_default(),
                    });
                    bridge_state.set_response(
                        req.id,
                        dioxus_shared::mcp::bridge::Response {
                            result: Some(tree),
                            error: None,
                        },
                    );
                }

                for req in bridge_state.dequeue_navigate_requests() {
                    let valid_routes: Vec<&str> =
                        schema.pages.iter().map(|p| p.route.as_str()).collect();
                    if valid_routes.contains(&req.route.as_str()) {
                        log_info!("[mcp] navigate route={} id={}", req.route, req.id);
                        bus.navigate(&req.route, None);
                        bridge_state.set_response(
                            req.id,
                            dioxus_shared::mcp::bridge::Response {
                                result: Some(serde_json::json!({
                                    "navigated": true,
                                    "route": req.route,
                                })),
                                error: None,
                            },
                        );
                    } else {
                        bridge_state.set_response(
                            req.id,
                            dioxus_shared::mcp::bridge::Response {
                                result: None,
                                error: Some(format!(
                                    "unknown route: {} (valid routes: {})",
                                    req.route,
                                    valid_routes.join(", ")
                                )),
                            },
                        );
                    }
                }

                for req in bridge_state.dequeue_dom_query_requests() {
                    log_info!("[mcp] dom_query selector={:?} id={}", req.selector, req.id);
                    let js = process_dom_query_sync(&req);
                    let eval_req = dioxus_shared::mcp::bridge::EvalRequest {
                        id: req.id.clone(),
                        method: "dom_query".to_string(),
                        payload: serde_json::json!({ "js": js }),
                    };
                    let result =
                        dioxus_shared::mcp::bridge::process_webview_eval_async(&desktop, &eval_req)
                            .await;
                    bridge_state.set_response(
                        req.id,
                        dioxus_shared::mcp::bridge::Response {
                            result: Some(serde_json::Value::String(result)),
                            error: None,
                        },
                    );
                }

                for req in bridge_state.dequeue_event_simulate_requests() {
                    log_info!(
                        "[mcp] event_simulate {} on {} id={}",
                        req.event_type,
                        req.selector,
                        req.id
                    );
                    let js = process_event_simulate_sync(&req);
                    let eval_req = dioxus_shared::mcp::bridge::EvalRequest {
                        id: req.id.clone(),
                        method: "event_simulate".to_string(),
                        payload: serde_json::json!({ "js": js }),
                    };
                    let result =
                        dioxus_shared::mcp::bridge::process_webview_eval_async(&desktop, &eval_req)
                            .await;
                    bridge_state.set_response(
                        req.id,
                        dioxus_shared::mcp::bridge::Response {
                            result: Some(serde_json::Value::String(result)),
                            error: None,
                        },
                    );
                }

                for req in bridge_state.dequeue_computed_styles_requests() {
                    log_info!("[mcp] computed_styles {} id={}", req.selector, req.id);
                    let js = process_computed_styles_sync(&req);
                    let eval_req = dioxus_shared::mcp::bridge::EvalRequest {
                        id: req.id.clone(),
                        method: "computed_styles".to_string(),
                        payload: serde_json::json!({ "js": js }),
                    };
                    let result =
                        dioxus_shared::mcp::bridge::process_webview_eval_async(&desktop, &eval_req)
                            .await;
                    bridge_state.set_response(
                        req.id,
                        dioxus_shared::mcp::bridge::Response {
                            result: Some(serde_json::Value::String(result)),
                            error: None,
                        },
                    );
                }

                for req in bridge_state.dequeue_css_audit_requests() {
                    log_info!("[mcp] css_audit selector={:?} id={}", req.selector, req.id);
                    let js = generate_css_audit_js(req.selector.as_deref());
                    let eval_req = dioxus_shared::mcp::bridge::EvalRequest {
                        id: req.id.clone(),
                        method: "css_audit".to_string(),
                        payload: serde_json::json!({ "js": js }),
                    };
                    let result =
                        dioxus_shared::mcp::bridge::process_webview_eval_async(&desktop, &eval_req)
                            .await;
                    bridge_state.set_response(
                        req.id,
                        dioxus_shared::mcp::bridge::Response {
                            result: Some(serde_json::json!({ "audit": result })),
                            error: None,
                        },
                    );
                }

                for req in bridge_state.dequeue_eval_requests() {
                    log_info!("[mcp] eval method={}", req.method);
                    let result =
                        dioxus_shared::mcp::bridge::process_webview_eval_async(&desktop, &req)
                            .await;
                    bridge_state.set_response(
                        req.id,
                        dioxus_shared::mcp::bridge::Response {
                            result: Some(serde_json::Value::String(result)),
                            error: None,
                        },
                    );
                }

                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        });
        BridgeShutdownOnDrop(bridge_state_for_loop)
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

/// Serialize a CanvasElement tree into schema-derived component-tree nodes,
/// each carrying the live binding value for every bound element.
fn serialize_tree_with_values(
    elements: &[CanvasElement],
    bindings: &std::collections::HashMap<String, String>,
) -> Vec<serde_json::Value> {
    elements
        .iter()
        .map(|el| {
            let binding = el.data_binding.as_ref().map(|b| b.field.clone());
            let value = binding
                .as_ref()
                .and_then(|f| bindings.get(f))
                .cloned()
                .unwrap_or_default();
            serde_json::json!({
                "id": el.id,
                "component": el.component,
                "binding": binding,
                "value": value,
                "children": serialize_tree_with_values(&el.children, bindings),
            })
        })
        .collect()
}

/// Snapshot the primary translator bindings with sensible defaults so bridge
/// inspection responses are deterministic even before the user has typed.
fn binding_values(bus: &ActionBus) -> serde_json::Value {
    serde_json::json!({
        "source_text": bus.get_binding("source_text").unwrap_or_default(),
        "source_lang": bus.get_binding("source_lang").unwrap_or_else(|| "en".to_string()),
        "target_lang": bus.get_binding("target_lang").unwrap_or_else(|| "es".to_string()),
        "translated_text": bus.get_binding("translated_text").unwrap_or_default(),
    })
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

fn main() {
    let (bridge, state) = McpBridge::new(9223);
    // Forward every Logger::global() entry into the bridge log buffer so the
    // bridge-backed `logs_tail`/`logs_read` report real, bridge-sourced entries.
    Logger::global().set_bridge_state(state.clone());
    translator::bridge::inject_app_identity(&state);
    log_info!("[startup] MCP bridge wiring complete; binding 127.0.0.1:9223");
    let listener = match bridge.bind() {
        Ok(listener) => listener,
        Err(error) => {
            log_error!(
                "[startup] MCP bridge cannot bind exactly to 127.0.0.1:9223: {error}"
            );
            eprintln!("MCP Bridge cannot bind exactly to 127.0.0.1:9223: {error}");
            std::process::exit(1);
        }
    };
    log_info!("[startup] MCP bridge bound 127.0.0.1:9223; advertising listening state");

    thread::spawn(move || {
        log_info!("[bridge] MCP bridge accept loop starting on ws://127.0.0.1:9223");
        if let Err(error) = bridge.run_with_listener(listener) {
            log_error!("[bridge] MCP bridge accept loop stopped: {error}");
        }
        log_info!("[bridge] MCP bridge accept loop ended");
    });

    // Drain the bridge command queue on a dedicated worker thread; webview
    // work (eval/page snapshot/UI actions) is routed to typed queues drained
    // by the Dioxus-executor processor in ActionProcessor.
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
