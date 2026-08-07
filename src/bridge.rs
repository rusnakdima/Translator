//! Bridge layer - MCP bridge consumer and command/action dispatchers
//!
//! These functions run on a dedicated thread polling the MCP command queue.
//! Extracted from main.rs so they can be tested without the Dioxus binary.

use crate::application::TranslationService;
use crate::domain::{AppSettings, TranslationResponse, SUPPORTED_LANGUAGES};
use dioxus_shared::logger::{LogEntry, Logger};
use dioxus_shared::mcp::bridge::{
    build_commands_list, handle_bridge_command, is_bridge_method, AppMetadata, BridgeState,
    Response,
};
use dioxus_shared::AppError;
use std::sync::Arc;
use std::thread;

/// Names of app-specific commands registered by the Translator. Shared
/// generic bridge methods are listed separately via `BRIDGE_METHODS`; this
/// list MUST contain only commands `invoke_app_command` actually handles.
const APP_COMMANDS: &[&str] = &[
    "translator.languages.list",
    "translator.translate",
    "translator.settings.save",
];

/// One-shot helper that injects the app identity, log source, and
/// app-registered commands into the bridge state. Called from
/// `main.rs` (or directly from a test) before the consumer loop starts.
pub fn inject_app_identity(state: &BridgeState) {
    state.set_app_metadata(AppMetadata::new(
        "translator",
        env!("CARGO_PKG_VERSION"),
        "translator",
    ));
    state.set_log_source("translator".to_string());
    state.set_app_commands(APP_COMMANDS.iter().map(|s| s.to_string()).collect());
}

/// Polls pending bridge requests and posts a Response for each.
///
/// Polled from a dedicated thread so MCP calls stop timing out.
/// - Synchronous commands (ping, app_info, commands_list, commands_invoke,
///   screenshot, logs_read, logs_tail): handled directly here.
/// - evaluate_js, dom_snapshot, page_snapshot, ui_invoke_action: routed to the
///   typed queues drained by the Dioxus-executor processor (`spawn_forever`
///   loop in the app's `ActionProcessor`), which delivers each response
///   exactly once via `BridgeState::set_response`.
/// - dom_query, event_simulate, computed_styles, css_audit, navigate,
///   component_tree, get_schema: routed through the shared bridge into typed
///   queues (also drained on the Dioxus executor where the webview lives).
pub fn bridge_consumer_loop(state: Arc<BridgeState>) {
    loop {
        if state.is_shutdown() {
            break;
        }

        // Process any pending JS eval results stored by the Dioxus eval loop
        for (id, result) in state.dequeue_js_results() {
            state.set_response(
                id,
                Response {
                    // Pass the raw JS eval result directly — process_request returns it
                    // as the JSON-RPC result field without additional wrapping
                    result: Some(serde_json::json!(result)),
                    error: None,
                },
            );
        }

        for cmd in state.dequeue_all() {
            // eval commands: route to pending_eval_requests; bridge_consumer_loop
            // handles them by draining pending_js_results at top of next iteration
            if matches!(cmd.method.as_str(), "evaluate_js" | "dom_snapshot") {
                state.enqueue_eval_request(dioxus_shared::mcp::bridge::EvalRequest {
                    id: cmd.id,
                    method: cmd.method,
                    payload: cmd.params,
                });
                continue;
            }

            // page_snapshot: route to pending_page_snapshot_requests for Dioxus main thread
            if cmd.method == "page_snapshot" {
                state.enqueue_page_snapshot_request(
                    dioxus_shared::mcp::bridge::PageSnapshotRequest { id: cmd.id.clone() },
                );
                continue;
            }

            // dom_query/event_simulate/computed_styles/css_audit and the
            // schema-derived inspection methods (navigate/component_tree/
            // get_schema): routed by the shared bridge into typed queues
            // drained on the Dioxus executor where the webview lives.
            if matches!(
                cmd.method.as_str(),
                "dom_query"
                    | "event_simulate"
                    | "computed_styles"
                    | "css_audit"
                    | "navigate"
                    | "component_tree"
                    | "get_schema"
            ) {
                handle_bridge_command(&state, &cmd);
                continue;
            }

            let response = match cmd.method.as_str() {
                // Synchronous commands (handled directly here)
                "ping" => Response {
                    result: Some(serde_json::json!({ "pong": true })),
                    error: None,
                },
                "app_info" => {
                    let metadata = state.app_metadata().unwrap_or_else(|| {
                        AppMetadata::new(
                            "translator",
                            env!("CARGO_PKG_VERSION"),
                            state.log_source(),
                        )
                    });
                    let info = metadata.to_app_info(env!("CARGO_PKG_VERSION"));
                    match serde_json::to_value(info) {
                        Ok(value) => Response {
                            result: Some(value),
                            error: None,
                        },
                        Err(e) => Response {
                            result: None,
                            error: Some(format!("app_info serialization: {e}")),
                        },
                    }
                }
                "bridge_status" => {
                    let bound_port = state.bound_port().unwrap_or(0);
                    Response {
                        result: Some(serde_json::json!({
                            "bound_port": bound_port,
                            "host": "127.0.0.1",
                            "listening": state.is_listening(),
                            "protocol_version": "2.0",
                        })),
                        error: None,
                    }
                }
                "initialize" => Response {
                    result: Some(serde_json::json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": { "tools": true },
                        "serverInfo": {
                            "name": "translator",
                            "version": env!("CARGO_PKG_VERSION"),
                        }
                    })),
                    error: None,
                },
                "commands_list" => {
                    // Shared bridge methods (css_audit, etc.) come first,
                    // app-registered commands are appended after.
                    Response {
                        result: Some(build_commands_list(&state.app_commands())),
                        error: None,
                    }
                }
                "logs_read" => {
                    let lines = cmd
                        .params
                        .get("lines")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(50) as usize;
                    let filter = cmd
                        .params
                        .get("filter")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_lowercase());

                    let all_entries = Logger::global().get_entries().unwrap_or_default();

                    let source_tag = state.log_source();
                    let formatted: Vec<String> = all_entries
                        .into_iter()
                        .map(|entry: LogEntry| {
                            let source = entry.source.as_deref().unwrap_or(source_tag.as_str());
                            format!(
                                "[{} {} {}] {}",
                                entry.level, entry.timestamp, source, entry.message
                            )
                        })
                        .filter(|log| {
                            filter
                                .as_ref()
                                .is_none_or(|f| log.to_lowercase().contains(f))
                        })
                        .rev()
                        .take(lines)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();

                    Response {
                        result: Some(serde_json::json!({
                            "lines": formatted,
                            "count": formatted.len(),
                            "source": source_tag,
                        })),
                        error: None,
                    }
                }
                "logs_tail" => {
                    let lines = cmd
                        .params
                        .get("lines")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(50) as usize;
                    let filter = cmd
                        .params
                        .get("filter")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_lowercase());

                    let buffer = state.get_logs();
                    let formatted: Vec<String> = buffer
                        .into_iter()
                        .filter(|log| {
                            filter
                                .as_ref()
                                .is_none_or(|f| log.to_lowercase().contains(f))
                        })
                        .rev()
                        .take(lines)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect();

                    Response {
                        result: Some(serde_json::json!({
                            "lines": formatted,
                            "count": formatted.len(),
                            "source": "bridge",
                        })),
                        error: None,
                    }
                }
                "screenshot" => {
                    let format = cmd
                        .params
                        .get("format")
                        .and_then(|v| v.as_str())
                        .unwrap_or("png")
                        .to_lowercase();
                    let quality = cmd
                        .params
                        .get("quality")
                        .and_then(|v| v.as_u64())
                        .map(|q| q as u32)
                        .unwrap_or(90);

                    use xcap::Monitor;
                    match Monitor::all() {
                        Ok(monitors) if !monitors.is_empty() => {
                            let monitor = &monitors[0];
                            match monitor.capture_image() {
                                Ok(img) => {
                                    let width = img.width();
                                    let height = img.height();
                                    match encode_screenshot(&img, &format, quality) {
                                        Ok(bytes) => Response {
                                            result: Some(serde_json::json!({
                                                "format": format,
                                                "quality": quality,
                                                "width": width,
                                                "height": height,
                                                "data": base64::Engine::encode(
                                                    &base64::engine::general_purpose::STANDARD,
                                                    bytes
                                                )
                                            })),
                                            error: None,
                                        },
                                        Err(e) => Response {
                                            result: None,
                                            error: Some(e),
                                        },
                                    }
                                }
                                Err(e) => Response {
                                    result: None,
                                    error: Some(format!("capture failed: {e}")),
                                },
                            }
                        }
                        _ => Response {
                            result: None,
                            error: Some("no monitors available".to_string()),
                        },
                    }
                }
                "commands_invoke" => {
                    let name = cmd
                        .params
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let payload = cmd
                        .params
                        .get("payload")
                        .cloned()
                        .unwrap_or(serde_json::json!({}));
                    if name == "translator.translate" {
                        // Executor drain (main.rs) sets `translated_text`; never
                        // re-enter handle_action from this path.
                        state.enqueue_command_invoke_request(
                            dioxus_shared::mcp::bridge::CommandInvokeRequest {
                                id: cmd.id.clone(),
                                name: name.to_string(),
                                payload,
                            },
                        );
                        continue;
                    }
                    match invoke_app_command(name, &payload, &state) {
                        Ok(value) => Response {
                            result: Some(value),
                            error: None,
                        },
                        Err(e) => Response {
                            result: None,
                            error: Some(e.to_string()),
                        },
                    }
                }
                "ui_invoke_action" => {
                    let action = cmd
                        .params
                        .get("action")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let payload = cmd
                        .params
                        .get("payload")
                        .cloned()
                        .unwrap_or(serde_json::json!({}));
                    state.enqueue_ui_action_request(dioxus_shared::mcp::bridge::ActionRequest {
                        id: cmd.id.clone(),
                        action: action.to_string(),
                        payload,
                    });
                    // Response set asynchronously by main thread after dispatch
                    continue;
                }
                other => {
                    if is_bridge_method(other) {
                        // Generic bridge method the Translator loop does
                        // not implement itself; surface as method-not-found
                        // so the client knows to look elsewhere.
                        Response {
                            result: None,
                            error: Some(format!("Method not found: {other} (code -32601)")),
                        }
                    } else {
                        Response {
                            result: None,
                            error: Some(format!("unsupported bridge method: {other}")),
                        }
                    }
                }
            };
            state.set_response(cmd.id, response);
        }
        thread::sleep(std::time::Duration::from_millis(25));
    }
}

/// Encode a captured screenshot honoring the requested `format`/`quality`.
/// PNG is lossless (quality is recorded, not applied); JPEG honors
/// `1..=100` quality. Any other format returns an explicit error string so
/// the caller gets an actionable unsupported-format response.
fn encode_screenshot(
    img: &image::RgbaImage,
    format: &str,
    quality: u32,
) -> Result<Vec<u8>, String> {
    use image::ImageEncoder;
    let width = img.width();
    let height = img.height();
    match format {
        "png" => {
            let mut bytes = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut bytes);
            encoder
                .write_image(img.as_raw(), width, height, image::ExtendedColorType::Rgba8)
                .map_err(|e| format!("PNG encoding failed: {e}"))?;
            Ok(bytes)
        }
        "jpg" | "jpeg" => {
            let rgb = image::DynamicImage::ImageRgba8(img.clone()).to_rgb8();
            let mut bytes = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut bytes,
                quality.clamp(1, 100) as u8,
            );
            encoder
                .write_image(rgb.as_raw(), width, height, image::ExtendedColorType::Rgb8)
                .map_err(|e| format!("JPEG encoding failed: {e}"))?;
            Ok(bytes)
        }
        other => Err(format!(
            "unsupported screenshot format: {other} (supported: png, jpg, jpeg)"
        )),
    }
}

/// Synchronous dispatch for bridge-invoked app commands. Returns JSON to send
/// back to the MCP client. This is the MVP — full per-command registry is
/// Phase 4 work per [`COMMAND_REGISTRY.md`](../DOCS/MIGRATION/COMMAND_REGISTRY.md).
pub fn invoke_app_command(
    name: &str,
    payload: &serde_json::Value,
    state: &Arc<dioxus_shared::mcp::bridge::BridgeState>,
) -> Result<serde_json::Value, AppError> {
    match name {
        "translator.languages.list" => {
            let langs = SUPPORTED_LANGUAGES
                .iter()
                .map(|(code, name)| serde_json::json!({ "code": code, "name": name }))
                .collect::<Vec<_>>();
            Ok(serde_json::json!(langs))
        }
        "translator.translate" => {
            let text = payload.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let source = payload
                .get("source_lang")
                .and_then(|v| v.as_str())
                .unwrap_or("en");
            let target = payload
                .get("target_lang")
                .and_then(|v| v.as_str())
                .unwrap_or("es");
            match TranslationService::translate(text, source, target) {
                Ok(response) => {
                    let data = response.data.unwrap_or(TranslationResponse {
                        translated_text: text.to_string(),
                    });
                    Ok(serde_json::json!({
                        "translated_text": data.translated_text,
                        "source_lang": source,
                        "target_lang": target,
                        "status": response.status,
                        "message": response.message,
                    }))
                }
                Err(e) => Err(e),
            }
        }
        "translator.settings.save" => {
            let settings: AppSettings = serde_json::from_value(payload.clone())
                .map_err(|e| AppError::ValidationError(format!("invalid settings payload: {e}")))?;
            let path = crate::infrastructure::get_settings_path();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AppError::Io(format!("failed to create settings directory: {e}"))
                })?;
            }
            let content = serde_json::to_string_pretty(&settings)
                .map_err(|e| AppError::Io(format!("failed to serialize settings: {e}")))?;
            std::fs::write(&path, content)
                .map_err(|e| AppError::Io(format!("failed to write settings file: {e}")))?;
            Ok(serde_json::json!({ "ok": true, "saved": settings }))
        }
        other => Err(AppError::NotFound(format!("unknown command: {other}"))),
    }
}

/// Synchronous dispatch for UI actions invoked from MCP.
///
/// Implemented actions return `{"ok": true, "action": "<name>"}`. Actions
/// listed here but not wired in `handle_action` (e.g. `add_term`) return
/// `{"error": "not-implemented", "action": "<name>"}` so callers can detect
/// the gap. Unknown actions return `AppError::NotFound`.
pub fn invoke_ui_action(
    action: &str,
    _payload: &serde_json::Value,
) -> Result<serde_json::Value, AppError> {
    const IMPLEMENTED_UI_ACTIONS: &[&str] = &[
        "translate",
        "save_settings",
        "toggle_theme",
        "swap_languages",
        "show-shortcuts",
        "close",
        "clear_text",
        "copy_result",
    ];
    const UNIMPLEMENTED_UI_ACTIONS: &[&str] = &["add_term"];

    if IMPLEMENTED_UI_ACTIONS.contains(&action) {
        return Ok(serde_json::json!({ "ok": true, "action": action }));
    }
    if UNIMPLEMENTED_UI_ACTIONS.contains(&action) {
        return Ok(serde_json::json!({
            "error": "not-implemented",
            "action": action,
        }));
    }
    Err(AppError::NotFound(format!("unknown ui action: {action}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus_shared::mcp::bridge::Command;
    use std::time::Duration;

    #[test]
    fn bridge_consumer_loop_ping_returns_pong() {
        let state = Arc::new(BridgeState::new());
        state.enqueue(Command {
            id: "1".to_string(),
            method: "ping".to_string(),
            params: serde_json::json!({}),
            received_at: std::time::Instant::now(),
        });

        let s = state.clone();
        let handle = std::thread::spawn(move || bridge_consumer_loop(s));

        std::thread::sleep(Duration::from_millis(50));

        state.request_shutdown();
        let _ = handle.join();

        let resp = state.get_response("1").expect("no response for id 1");
        assert!(
            resp.result.is_some(),
            "expected result, got error: {:?}",
            resp.error
        );
        let result = resp.result.unwrap();
        assert_eq!(result.pointer("/pong"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn bridge_consumer_loop_app_info_uses_injected_metadata() {
        let state = Arc::new(BridgeState::new());
        inject_app_identity(&state);
        state.enqueue(Command {
            id: "2".to_string(),
            method: "app_info".to_string(),
            params: serde_json::json!({}),
            received_at: std::time::Instant::now(),
        });

        let s = state.clone();
        let handle = std::thread::spawn(move || bridge_consumer_loop(s));

        std::thread::sleep(Duration::from_millis(50));

        state.request_shutdown();
        let _ = handle.join();

        let resp = state.get_response("2").expect("no response for id 2");
        assert!(
            resp.result.is_some(),
            "expected result, got error: {:?}",
            resp.error
        );
        let result = resp.result.unwrap();
        assert_eq!(
            result.pointer("/name"),
            Some(&serde_json::json!("translator"))
        );
        assert!(result.pointer("/version").is_some());
        assert!(result.pointer("/platform").is_some());
        assert_eq!(
            result.pointer("/dioxus_version").and_then(|v| v.as_str()),
            Some(env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn bridge_consumer_loop_app_info_falls_back_without_injection() {
        let state = Arc::new(BridgeState::new());
        state.enqueue(Command {
            id: "2b".to_string(),
            method: "app_info".to_string(),
            params: serde_json::json!({}),
            received_at: std::time::Instant::now(),
        });

        let s = state.clone();
        let handle = std::thread::spawn(move || bridge_consumer_loop(s));

        std::thread::sleep(Duration::from_millis(50));

        state.request_shutdown();
        let _ = handle.join();

        let resp = state.get_response("2b").expect("no response for id 2b");
        let result = resp.result.expect("expected result");
        assert!(result.pointer("/name").and_then(|v| v.as_str()).is_some());
        assert!(result
            .pointer("/platform")
            .and_then(|v| v.as_str())
            .is_some());
    }

    #[test]
    fn bridge_consumer_loop_commands_list_includes_css_audit_and_app_commands() {
        let state = Arc::new(BridgeState::new());
        inject_app_identity(&state);
        state.enqueue(Command {
            id: "cl".to_string(),
            method: "commands_list".to_string(),
            params: serde_json::json!({}),
            received_at: std::time::Instant::now(),
        });

        let s = state.clone();
        let handle = std::thread::spawn(move || bridge_consumer_loop(s));

        std::thread::sleep(Duration::from_millis(50));

        state.request_shutdown();
        let _ = handle.join();

        let resp = state.get_response("cl").expect("no response for cl");
        let result = resp.result.expect("expected result");
        let arr = result.as_array().expect("array");
        assert!(
            arr.iter()
                .any(|v| v.get("name").and_then(|n| n.as_str()) == Some("css_audit")),
            "css_audit must be in commands_list (got {result})"
        );
        assert!(
            arr.iter()
                .any(|v| v.as_str() == Some("translator.translate")),
            "translator.translate must be in commands_list"
        );
    }

    #[test]
    fn bridge_consumer_loop_ui_invoke_action_uses_payload_field() {
        let state = Arc::new(BridgeState::new());
        state.enqueue(Command {
            id: "ui-p".to_string(),
            method: "ui_invoke_action".to_string(),
            params: serde_json::json!({
                "action": "toggle_theme",
                "payload": { "value": true },
            }),
            received_at: std::time::Instant::now(),
        });

        let s = state.clone();
        let handle = std::thread::spawn(move || bridge_consumer_loop(s));

        std::thread::sleep(Duration::from_millis(50));

        state.request_shutdown();
        let _ = handle.join();

        let pending = state.dequeue_ui_action_requests();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].action, "toggle_theme");
        assert_eq!(pending[0].payload, serde_json::json!({ "value": true }));
    }

    #[test]
    fn bridge_consumer_loop_unknown_method() {
        let state = Arc::new(BridgeState::new());
        state.enqueue(Command {
            id: "3".to_string(),
            method: "completely_unknown_method".to_string(),
            params: serde_json::json!({}),
            received_at: std::time::Instant::now(),
        });

        let s = state.clone();
        let handle = std::thread::spawn(move || bridge_consumer_loop(s));

        std::thread::sleep(Duration::from_millis(50));

        state.request_shutdown();
        let _ = handle.join();

        let resp = state.get_response("3").expect("no response for id 3");
        assert!(resp.error.is_some(), "expected error for unknown method");
        let err = resp.error.unwrap();
        assert!(err.contains("unsupported bridge method"));
    }

    // --- invoke_app_command tests ---

    #[test]
    fn invoke_app_command_languages_list() {
        let state = Arc::new(dioxus_shared::mcp::bridge::BridgeState::new());
        let result =
            invoke_app_command("translator.languages.list", &serde_json::json!({}), &state)
                .expect("expected Ok");
        let langs = result.as_array().expect("expected array");
        assert!(!langs.is_empty());
        assert_eq!(langs[0].pointer("/code"), Some(&serde_json::json!("en")));
    }

    #[test]
    fn invoke_app_command_translate_same_lang_passthrough() {
        let payload = serde_json::json!({
            "text": "hello",
            "source_lang": "en",
            "target_lang": "en"
        });
        let state = Arc::new(dioxus_shared::mcp::bridge::BridgeState::new());
        let result =
            invoke_app_command("translator.translate", &payload, &state).expect("expected Ok");
        assert_eq!(
            result.pointer("/translated_text"),
            Some(&serde_json::json!("hello"))
        );
        assert_eq!(
            result.pointer("/message"),
            Some(&serde_json::json!("Same language"))
        );
    }

    #[test]
    fn invoke_app_command_translate_empty_input_is_validation_error() {
        let payload = serde_json::json!({
            "text": "   ",
            "source_lang": "en",
            "target_lang": "es"
        });
        let state = Arc::new(dioxus_shared::mcp::bridge::BridgeState::new());
        let result =
            invoke_app_command("translator.translate", &payload, &state);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn invoke_app_translate_unsupported_lang_is_validation_error() {
        let payload = serde_json::json!({
            "text": "hello",
            "source_lang": "en",
            "target_lang": "zz"
        });
        let state = Arc::new(dioxus_shared::mcp::bridge::BridgeState::new());
        let result =
            invoke_app_command("translator.translate", &payload, &state);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unsupported language"));
    }

    #[test]
    fn invoke_app_command_unknown() {
        let state = Arc::new(dioxus_shared::mcp::bridge::BridgeState::new());
        let result =
            invoke_app_command("translator.does_not_exist", &serde_json::json!({}), &state);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("unknown command"));
    }

    // --- invoke_ui_action tests ---

    #[test]
    fn invoke_ui_action_toggle_theme() {
        let result = invoke_ui_action("toggle_theme", &serde_json::json!({})).expect("expected Ok");
        assert_eq!(result.pointer("/ok"), Some(&serde_json::json!(true)));
        assert_eq!(
            result.pointer("/action"),
            Some(&serde_json::json!("toggle_theme"))
        );
    }

    #[test]
    fn invoke_ui_action_add_term_not_implemented() {
        let result = invoke_ui_action("add_term", &serde_json::json!({})).expect("expected Ok");
        assert_eq!(
            result.pointer("/error"),
            Some(&serde_json::json!("not-implemented"))
        );
        assert_eq!(
            result.pointer("/action"),
            Some(&serde_json::json!("add_term"))
        );
        assert!(result.pointer("/ok").is_none());
    }

    #[test]
    fn invoke_ui_action_save_settings() {
        let result =
            invoke_ui_action("save_settings", &serde_json::json!({})).expect("expected Ok");
        assert_eq!(
            result.pointer("/action"),
            Some(&serde_json::json!("save_settings"))
        );
    }

    #[test]
    fn invoke_ui_action_translate() {
        let result = invoke_ui_action("translate", &serde_json::json!({})).expect("expected Ok");
        assert_eq!(
            result.pointer("/action"),
            Some(&serde_json::json!("translate"))
        );
    }

    #[test]
    fn invoke_ui_action_swap_languages() {
        let result =
            invoke_ui_action("swap_languages", &serde_json::json!({})).expect("expected Ok");
        assert_eq!(
            result.pointer("/action"),
            Some(&serde_json::json!("swap_languages"))
        );
    }

    #[test]
    fn invoke_ui_action_show_shortcuts() {
        let result =
            invoke_ui_action("show-shortcuts", &serde_json::json!({})).expect("expected Ok");
        assert_eq!(
            result.pointer("/action"),
            Some(&serde_json::json!("show-shortcuts"))
        );
    }

    #[test]
    fn invoke_ui_action_close() {
        let result = invoke_ui_action("close", &serde_json::json!({})).expect("expected Ok");
        assert_eq!(result.pointer("/action"), Some(&serde_json::json!("close")));
    }

    #[test]
    fn invoke_ui_action_clear_text() {
        let result = invoke_ui_action("clear_text", &serde_json::json!({})).expect("expected Ok");
        assert_eq!(
            result.pointer("/action"),
            Some(&serde_json::json!("clear_text"))
        );
    }

    #[test]
    fn invoke_ui_action_copy_result() {
        let result = invoke_ui_action("copy_result", &serde_json::json!({})).expect("expected Ok");
        assert_eq!(
            result.pointer("/action"),
            Some(&serde_json::json!("copy_result"))
        );
    }

    #[test]
    fn invoke_ui_action_unknown() {
        let result = invoke_ui_action("definitely_not_a_real_action", &serde_json::json!({}));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("unknown ui action"));
    }

    // --- evaluate_js / dom_snapshot routing tests ---

    #[test]
    fn bridge_consumer_loop_evaluate_js_routes_to_pending_eval_requests() {
        let state = Arc::new(BridgeState::new());
        let cmd_id = "eval-1".to_string();
        state.enqueue(Command {
            id: cmd_id.clone(),
            method: "evaluate_js".to_string(),
            params: serde_json::json!({ "code": "1 + 1" }),
            received_at: std::time::Instant::now(),
        });

        let s = state.clone();
        let handle = std::thread::spawn(move || bridge_consumer_loop(s));

        // Wait for the command to be dequeued and routed
        std::thread::sleep(Duration::from_millis(100));

        // Shutdown before checking results
        state.request_shutdown();
        let _ = handle.join();

        // Response should NOT be set directly (eval commands are routed to pending_eval_requests)
        // The response would be set by the use_effect eval loop, but since there's no eval loop
        // in this test, we just verify the command was dequeued and nothing was set.
        assert!(
            state.get_response(&cmd_id).is_none(),
            "evaluate_js should not have a response set directly by bridge_consumer_loop"
        );

        // Verify the eval request was enqueued
        let eval_requests = state.dequeue_eval_requests();
        assert_eq!(eval_requests.len(), 1, "expected 1 eval request enqueued");
        assert_eq!(eval_requests[0].id, cmd_id);
        assert_eq!(eval_requests[0].method, "evaluate_js");
        assert_eq!(
            eval_requests[0].payload.get("code"),
            Some(&serde_json::json!("1 + 1"))
        );
    }

    #[test]
    fn bridge_consumer_loop_dom_snapshot_routes_to_pending_eval_requests() {
        let state = Arc::new(BridgeState::new());
        let cmd_id = "dom-1".to_string();
        state.enqueue(Command {
            id: cmd_id.clone(),
            method: "dom_snapshot".to_string(),
            params: serde_json::json!({ "selector": "#root" }),
            received_at: std::time::Instant::now(),
        });

        let s = state.clone();
        let handle = std::thread::spawn(move || bridge_consumer_loop(s));

        std::thread::sleep(Duration::from_millis(100));

        state.request_shutdown();
        let _ = handle.join();

        assert!(
            state.get_response(&cmd_id).is_none(),
            "dom_snapshot should not have a response set directly"
        );
        let eval_requests = state.dequeue_eval_requests();
        assert_eq!(eval_requests.len(), 1);
        assert_eq!(eval_requests[0].id, cmd_id);
        assert_eq!(eval_requests[0].method, "dom_snapshot");
        assert_eq!(
            eval_requests[0].payload.get("selector"),
            Some(&serde_json::json!("#root"))
        );
    }

    #[test]
    fn bridge_consumer_loop_delivers_pending_js_results() {
        let state = Arc::new(BridgeState::new());
        let cmd_id = "js-result-1".to_string();

        // Simulate the use_effect loop having stored a JS eval result.
        // The JS engine returns JSON-serialized values as strings,
        // e.g., eval("1+1") → "2", eval("'hello'") → "\"hello\""
        state.enqueue_js_result(cmd_id.clone(), "2".to_string());

        // Enqueue a ping command to keep the loop running
        state.enqueue(Command {
            id: "ping-cmd".to_string(),
            method: "ping".to_string(),
            params: serde_json::json!({}),
            received_at: std::time::Instant::now(),
        });

        let s = state.clone();
        let handle = std::thread::spawn(move || bridge_consumer_loop(s));

        std::thread::sleep(Duration::from_millis(100));

        state.request_shutdown();
        let _ = handle.join();

        // The JS result should have been delivered as a response
        let resp = state.get_response(&cmd_id);
        assert!(resp.is_some(), "expected response for js result id");
        let resp = resp.unwrap();
        assert!(
            resp.result.is_some(),
            "expected result, got error: {:?}",
            resp.error
        );
        // The raw JS result string "2" becomes a JSON string "2" when wrapped in json!()
        assert_eq!(resp.result.unwrap(), serde_json::json!("2"));
    }

    // --- schema-derived / webview inspection routing ---

    #[test]
    fn bridge_consumer_loop_routes_inspection_methods_to_typed_queues() {
        let state = Arc::new(BridgeState::new());
        let cases = [
            ("navigate", serde_json::json!({ "route": "/settings" })),
            ("component_tree", serde_json::json!({})),
            ("get_schema", serde_json::json!({})),
            ("dom_query", serde_json::json!({ "selector": "#x" })),
            (
                "event_simulate",
                serde_json::json!({ "event_type": "click", "selector": "#b" }),
            ),
            (
                "computed_styles",
                serde_json::json!({ "selector": "#x", "properties": ["color"] }),
            ),
            ("css_audit", serde_json::json!({ "selector": "#main" })),
        ];
        for (i, (method, params)) in cases.iter().enumerate() {
            state.enqueue(Command {
                id: format!("rt-{i}"),
                method: method.to_string(),
                params: params.clone(),
                received_at: std::time::Instant::now(),
            });
        }

        let s = state.clone();
        let handle = std::thread::spawn(move || bridge_consumer_loop(s));

        std::thread::sleep(Duration::from_millis(100));

        state.request_shutdown();
        let _ = handle.join();

        assert_eq!(state.dequeue_navigate_requests().len(), 1);
        assert_eq!(state.dequeue_component_tree_requests().len(), 1);
        assert_eq!(state.dequeue_schema_requests().len(), 1);
        assert_eq!(state.dequeue_dom_query_requests().len(), 1);
        assert_eq!(state.dequeue_event_simulate_requests().len(), 1);
        assert_eq!(state.dequeue_computed_styles_requests().len(), 1);
        assert_eq!(state.dequeue_css_audit_requests().len(), 1);
        assert!(
            state.get_response("rt-0").is_none(),
            "inspection methods must be routed to queues, not answered directly"
        );
        assert_eq!(
            state.dequeue_navigate_requests()[0].route,
            "/settings"
        );
    }

    // --- logs_tail ---

    #[test]
    fn bridge_consumer_loop_logs_tail_returns_bridge_entries() {
        let state = Arc::new(BridgeState::new());
        inject_app_identity(&state);
        state.append_log("[INFO] bridge log line one".to_string());
        state.append_log("[WARN] bridge log line two".to_string());
        state.enqueue(Command {
            id: "lt".to_string(),
            method: "logs_tail".to_string(),
            params: serde_json::json!({ "lines": 10 }),
            received_at: std::time::Instant::now(),
        });

        let s = state.clone();
        let handle = std::thread::spawn(move || bridge_consumer_loop(s));

        std::thread::sleep(Duration::from_millis(50));

        state.request_shutdown();
        let _ = handle.join();

        let resp = state.get_response("lt").expect("no response for lt");
        let result = resp.result.expect("expected result");
        assert_eq!(
            result.pointer("/source"),
            Some(&serde_json::json!("bridge"))
        );
        let lines = result
            .pointer("/lines")
            .and_then(|v| v.as_array())
            .expect("lines array");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].as_str(), Some("[INFO] bridge log line one"));
    }

    #[test]
    fn bridge_consumer_loop_logs_tail_filters_by_substring() {
        let state = Arc::new(BridgeState::new());
        state.append_log("[INFO] translate done".to_string());
        state.append_log("[INFO] theme toggled".to_string());
        state.enqueue(Command {
            id: "ltf".to_string(),
            method: "logs_tail".to_string(),
            params: serde_json::json!({ "filter": "theme" }),
            received_at: std::time::Instant::now(),
        });

        let s = state.clone();
        let handle = std::thread::spawn(move || bridge_consumer_loop(s));

        std::thread::sleep(Duration::from_millis(50));

        state.request_shutdown();
        let _ = handle.join();

        let resp = state.get_response("ltf").expect("no response for ltf");
        let result = resp.result.expect("expected result");
        let lines = result
            .pointer("/lines")
            .and_then(|v| v.as_array())
            .expect("lines array");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].as_str(), Some("[INFO] theme toggled"));
    }

    // --- screenshot format/quality ---

    #[test]
    fn encode_screenshot_honors_format_and_quality() {
        use image::{Rgba, RgbaImage};
        let mut img = RgbaImage::new(2, 2);
        img.put_pixel(0, 0, Rgba([255u8, 0, 0, 255]));
        img.put_pixel(1, 1, Rgba([0u8, 0, 255, 255]));

        let png = encode_screenshot(&img, "png", 90).expect("png ok");
        assert!(!png.is_empty());

        let jpeg = encode_screenshot(&img, "jpeg", 80).expect("jpeg ok");
        assert!(!jpeg.is_empty());

        let err = encode_screenshot(&img, "bmp", 90).unwrap_err();
        assert!(err.contains("unsupported screenshot format"), "got: {err}");
        assert!(err.contains("bmp"), "error should name the format: {err}");
    }
}
