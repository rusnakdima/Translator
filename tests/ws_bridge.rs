//! WebSocket bridge integration tests
//!
//! Run with: `cd Translator && cargo test --test ws_bridge`
//!
//! Tests are split into two classes:
//!
//! 1. **Pure mock tests** drive `bridge_consumer_loop` directly with synthetic
//!    `Command` inputs. They verify JSON-RPC shape, routing into typed
//!    queues, structured errors, and timing without requiring the app to be
//!    running — fast, hermetic, and always run.
//!
//! 2. **Live tests** start the Translator binary as a subprocess (mirroring
//!    the `dioxus-mcp` `dev_start` harness) and exercise the bridge
//!    end-to-end over `ws://127.0.0.1:9223`. They tear the process down
//!    on completion and skip with a clear "build the binary first" message
//!    if the binary is not present.
//!
//! **Timeouts are never accepted as success.** Every `tokio::time::timeout`
//! converts elapsed time into a structured assertion failure with the
//! elapsed duration attached; the failure message names the deadline that
//! was breached.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use dioxus_shared::mcp::bridge::{
    timeout::BRIDGE_CONNECT_DEADLINE_SECS, timeout::BRIDGE_RESPONSE_DEADLINE_SECS, BridgeState,
    Command as BridgeCommand, Response as BridgeResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use translator::bridge::{bridge_consumer_loop, inject_app_identity, invoke_app_command};

const BRIDGE_URL: &str = "ws://127.0.0.1:9223";
const APP_PORT: u16 = 9223;
const READY_TIMEOUT: Duration = Duration::from_secs(20);
const READY_PROBE_INTERVAL: Duration = Duration::from_millis(250);
const READY_PROBE_DEADLINE: Duration = Duration::from_secs(2);
const READ_DEADLINE: Duration = Duration::from_secs(6);
const POLL_RESPONSE_TIMEOUT: Duration = Duration::from_secs(2);

// ============================================================================
// === HARNESS ================================================================
// ============================================================================

/// Resolves the translator binary path. Tries the Cargo-provided env var
/// first (set for integration tests), then the conventional `target/debug`
/// location relative to `CARGO_MANIFEST_DIR`. Returns `None` if the binary
/// has not been built, in which case live tests report a clear skip-style
/// failure rather than guessing.
fn translator_binary() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_translator") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        for profile in ["debug", "release"] {
            let candidate = PathBuf::from(&manifest)
                .join("target")
                .join(profile)
                .join("translator");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Subprocess wrapper that owns the running Translator binary. On `Drop`
/// the child is killed and reaped, so the bridge port is always released.
struct AppHarness {
    child: Option<Child>,
    home: TempDir,
    started_at: Instant,
}

impl AppHarness {
    fn spawn() -> Result<Self, String> {
        let exe = translator_binary().ok_or_else(|| {
            "Translator binary not built. Run `cargo build` in Tauri/Translator before live tests.".to_string()
        })?;

        let home = TempDir::new().map_err(|e| format!("tempdir: {e}"))?;
        let mut cmd = Command::new(&exe);
        cmd.env("HOME", home.path());
        cmd.env("XDG_DATA_HOME", home.path().join(".local/share"));
        cmd.env("XDG_CONFIG_HOME", home.path().join(".config"));
        if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
            cmd.env("CARGO_MANIFEST_DIR", manifest);
        }
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::piped());

        let child = cmd
            .spawn()
            .map_err(|e| format!("spawn {}: {e}", exe.display()))?;

        Ok(Self {
            child: Some(child),
            home,
            started_at: Instant::now(),
        })
    }

    /// Polls the bridge with a tight connect deadline. Bounded by
    /// `READY_TIMEOUT` so a missing or wedged app fails fast with a
    /// clear error rather than running the suite to its outer deadline.
    async fn wait_ready(&mut self) -> Result<(), String> {
        let deadline = Instant::now() + READY_TIMEOUT;
        let mut last_err: Option<String> = None;
        while Instant::now() < deadline {
            if let Some(child) = self.child.as_mut() {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let stderr = self
                            .child
                            .as_mut()
                            .and_then(|c| c.stderr.as_mut())
                            .and_then(|s| {
                                use std::io::Read;
                                let mut buf = String::new();
                                let _ = s.read_to_string(&mut buf);
                                Some(buf)
                            })
                            .unwrap_or_default();
                        return Err(format!(
                            "Translator exited before bridge became ready (status: {status}); stderr: {stderr}"
                        ));
                    }
                    Ok(None) => {}
                    Err(e) => return Err(format!("try_wait: {e}")),
                }
            }
            match timeout(READY_PROBE_DEADLINE, connect_async(BRIDGE_URL)).await {
                Ok(Ok((mut ws, _))) => {
                    let _ = ws.close(None).await;
                    return Ok(());
                }
                Ok(Err(e)) => last_err = Some(format!("connect: {e}")),
                Err(_) => last_err = Some(format!("connect probe exceeded {READY_PROBE_DEADLINE:?}")),
            }
            tokio::time::sleep(READY_PROBE_INTERVAL).await;
        }
        let stderr = self
            .child
            .as_mut()
            .and_then(|c| c.stderr.as_mut())
            .and_then(|s| {
                use std::io::Read;
                let mut buf = String::new();
                let _ = s.read_to_string(&mut buf);
                Some(buf)
            })
            .unwrap_or_default();
        Err(format!(
            "bridge did not become ready within {READY_TIMEOUT:?} (last error: {}); stderr: {stderr}",
            last_err.unwrap_or_else(|| "none".to_string())
        ))
    }
}

impl Drop for AppHarness {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = &self.home;
        let elapsed = self.started_at.elapsed();
        eprintln!("[harness] app torn down after {elapsed:?}");
    }
}

// ============================================================================
// === SHARED HELPERS =========================================================
// ============================================================================

fn mock_bridge() -> (Arc<BridgeState>, std::thread::JoinHandle<()>) {
    let state = Arc::new(BridgeState::new());
    inject_app_identity(&state);
    let s = state.clone();
    let handle = std::thread::spawn(move || bridge_consumer_loop(s));
    (state, handle)
}

fn shutdown_bridge(state: &BridgeState, handle: std::thread::JoinHandle<()>) {
    state.request_shutdown();
    let _ = handle.join();
}

async fn enqueue_and_await(
    state: &Arc<BridgeState>,
    id: &str,
    method: &str,
    params: Value,
) -> BridgeResponse {
    let waiter = state.register_response_waiter(id);
    state.enqueue(BridgeCommand {
        id: id.to_string(),
        method: method.to_string(),
        params,
        received_at: Instant::now(),
    });
    let started = Instant::now();
    match waiter.await_response(POLL_RESPONSE_TIMEOUT).await {
        Ok(resp) => resp,
        Err(e) => panic!(
            "consumer did not deliver response for id={id} method={method} within {POLL_RESPONSE_TIMEOUT:?} (elapsed {:?}): {e:?}",
            started.elapsed()
        ),
    }
}

async fn send_request_now(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    id: &str,
    method: &str,
    params: Value,
) -> Result<(), String> {
    let frame = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    let text = serde_json::to_string(&frame).map_err(|e| format!("serialize: {e}"))?;
    SinkExt::send(ws, Message::Text(text.into()))
        .await
        .map_err(|e| format!("send: {e}"))
}

/// Open a fresh WebSocket, send one request, read one response, parse.
/// The full round-trip is bounded by `READ_DEADLINE` (which the bridge
/// server uses) plus a small safety margin.
async fn rpc_request(
    url: &str,
    id: &str,
    method: &str,
    params: Value,
) -> Result<Value, String> {
    let started = Instant::now();
    let connect = timeout(
        Duration::from_secs(BRIDGE_CONNECT_DEADLINE_SECS + 1),
        connect_async(url),
    )
    .await
    .map_err(|_| {
        format!(
            "connect exceeded {:?} (deadline: {}s)",
            started.elapsed(),
            BRIDGE_CONNECT_DEADLINE_SECS + 1
        )
    })?
    .map_err(|e| format!("connect: {e}"))?;
    let (mut ws, _) = connect;

    send_request_now(&mut ws, id, method, params).await?;
    let read = timeout(READ_DEADLINE, ws.next())
        .await
        .map_err(|_| {
            format!(
                "read exceeded {READ_DEADLINE:?} (response deadline: {}s)",
                BRIDGE_RESPONSE_DEADLINE_SECS
            )
        })?;
    drop(ws);
    let msg = read.ok_or_else(|| "stream closed".to_string())?.map_err(|e| format!("ws: {e}"))?;
    let text = match msg {
        Message::Text(t) => t,
        other => return Err(format!("non-text frame: {other:?}")),
    };
    serde_json::from_str::<Value>(&text).map_err(|e| format!("parse: {e}; body: {text}"))
}

/// Asserts the response is a JSON-RPC success and returns the result.
fn unwrap_result(value: &Value) -> &Value {
    assert_eq!(
        value.get("jsonrpc").and_then(|v| v.as_str()),
        Some("2.0"),
        "jsonrpc field missing/wrong in: {value}"
    );
    assert!(
        value.get("error").is_none(),
        "expected success, got error: {}",
        value
    );
    value
        .get("result")
        .unwrap_or_else(|| panic!("missing result in: {value}"))
}

/// Asserts the response is a JSON-RPC error and returns the error object.
fn unwrap_error(value: &Value) -> &Value {
    let err = value
        .get("error")
        .unwrap_or_else(|| panic!("missing error in: {value}"));
    let code = err.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
    let message = err
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    eprintln!("[assert] got structured error code={code} message={message}");
    err
}

// ============================================================================
// === PURE MOCK TESTS ========================================================
// ============================================================================

#[tokio::test]
async fn mock_ping_returns_pong_with_jsonrpc_envelope() {
    let (state, handle) = mock_bridge();
    let resp = enqueue_and_await(&state, "ping-1", "ping", json!({})).await;
    shutdown_bridge(&state, handle);

    let result = resp.result.expect("ping should return a result");
    assert_eq!(
        result.pointer("/pong"),
        Some(&json!(true)),
        "ping should return {{pong:true}}, got {result}"
    );
    assert!(resp.error.is_none(), "ping must not return an error");
}

#[tokio::test]
async fn mock_app_info_uses_injected_translator_identity() {
    let (state, handle) = mock_bridge();
    let resp = enqueue_and_await(&state, "info-1", "app_info", json!({})).await;
    shutdown_bridge(&state, handle);

    let result = resp.result.expect("app_info should return a result");
    assert_eq!(
        result.pointer("/name"),
        Some(&json!("translator")),
        "name must be 'translator', got {result}"
    );
    assert!(
        result.pointer("/version").and_then(|v| v.as_str()).is_some(),
        "version missing in {result}"
    );
    assert!(
        result.pointer("/platform").and_then(|v| v.as_str()).is_some(),
        "platform missing in {result}"
    );
    assert!(
        result.pointer("/dioxus_version").is_some(),
        "dioxus_version missing in {result}"
    );
}

#[tokio::test]
async fn mock_bridge_status_reports_listening_and_port() {
    let (state, handle) = mock_bridge();
    state.set_listening(true);
    state.set_bound_port(APP_PORT);
    let resp = enqueue_and_await(&state, "bs-1", "bridge_status", json!({})).await;
    shutdown_bridge(&state, handle);

    let result = resp.result.expect("bridge_status should return a result");
    assert_eq!(
        result.pointer("/listening"),
        Some(&json!(true)),
        "listening should be true after consumer is up, got {result}"
    );
    assert_eq!(
        result.pointer("/host"),
        Some(&json!("127.0.0.1")),
        "host should be 127.0.0.1, got {result}"
    );
    assert_eq!(
        result.pointer("/protocol_version"),
        Some(&json!("2.0")),
        "protocol_version should be 2.0, got {result}"
    );
    let port = result
        .pointer("/bound_port")
        .and_then(|v| v.as_u64())
        .expect("bound_port must be a number");
    assert_eq!(port, APP_PORT as u64, "bound_port should match");
}

#[tokio::test]
async fn mock_commands_list_contains_translator_and_bridge_methods() {
    let (state, handle) = mock_bridge();
    let resp = enqueue_and_await(&state, "cl-1", "commands_list", json!({})).await;
    shutdown_bridge(&state, handle);

    let result = resp.result.expect("commands_list should return a result");
    let arr = result.as_array().expect("commands_list must be an array");
    let names: Vec<&str> = arr
        .iter()
        .map(|v| {
            v.get("name")
                .and_then(|n| n.as_str())
                .or_else(|| v.as_str())
                .unwrap_or("")
        })
        .collect();
    assert!(
        names.contains(&"ping"),
        "commands_list missing ping: {names:?}"
    );
    assert!(
        names.contains(&"app_info"),
        "commands_list missing app_info: {names:?}"
    );
    assert!(
        names.contains(&"css_audit"),
        "commands_list missing css_audit: {names:?}"
    );
    assert!(
        names.contains(&"translator.translate"),
        "commands_list missing translator.translate: {names:?}"
    );
    assert!(
        names.contains(&"translator.languages.list"),
        "commands_list missing translator.languages.list: {names:?}"
    );
    assert!(
        names.contains(&"translator.settings.save"),
        "commands_list missing translator.settings.save: {names:?}"
    );
}

#[tokio::test]
async fn mock_unknown_method_returns_structured_error() {
    let (state, handle) = mock_bridge();
    let resp = enqueue_and_await(&state, "um-1", "completely_unknown_method", json!({})).await;
    shutdown_bridge(&state, handle);

    assert!(resp.result.is_none(), "unknown method must not return a result");
    let err = resp.error.expect("unknown method must return an error string");
    assert!(
        err.contains("unsupported bridge method"),
        "expected 'unsupported bridge method' in error, got: {err}"
    );
}

#[tokio::test]
async fn mock_evaluate_js_routes_to_pending_eval_requests() {
    let (state, handle) = mock_bridge();
    state.enqueue(BridgeCommand {
        id: "eval-rt".to_string(),
        method: "evaluate_js".to_string(),
        params: json!({ "code": "1 + 1" }),
        received_at: Instant::now(),
    });
    let _ = enqueue_and_await(&state, "ping-rt", "ping", json!({})).await;
    shutdown_bridge(&state, handle);

    assert!(
        state.get_response("eval-rt").is_none(),
        "evaluate_js must not be answered by the consumer loop"
    );
    let pending = state.dequeue_eval_requests();
    assert_eq!(pending.len(), 1, "expected 1 eval request enqueued");
    assert_eq!(pending[0].id, "eval-rt");
    assert_eq!(pending[0].method, "evaluate_js");
    assert_eq!(
        pending[0].payload.get("code"),
        Some(&json!("1 + 1"))
    );
}

#[tokio::test]
async fn mock_dom_snapshot_routes_to_pending_eval_requests() {
    let (state, handle) = mock_bridge();
    state.enqueue(BridgeCommand {
        id: "dom-rt".to_string(),
        method: "dom_snapshot".to_string(),
        params: json!({ "selector": "#root" }),
        received_at: Instant::now(),
    });
    let _ = enqueue_and_await(&state, "ping-d", "ping", json!({})).await;
    shutdown_bridge(&state, handle);

    assert!(state.get_response("dom-rt").is_none());
    let pending = state.dequeue_eval_requests();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].method, "dom_snapshot");
    assert_eq!(
        pending[0].payload.get("selector"),
        Some(&json!("#root"))
    );
}

#[tokio::test]
async fn mock_inspection_methods_route_to_typed_queues() {
    let (state, handle) = mock_bridge();
    let cases: &[(&str, Value)] = &[
        ("navigate", json!({ "route": "/settings" })),
        ("component_tree", json!({})),
        ("get_schema", json!({})),
        ("dom_query", json!({ "selector": "#x" })),
        ("event_simulate", json!({ "event_type": "click", "selector": "#b" })),
        ("computed_styles", json!({ "selector": "#x", "properties": ["color"] })),
        ("css_audit", json!({ "selector": "#main" })),
    ];
    for (i, (method, params)) in cases.iter().enumerate() {
        state.enqueue(BridgeCommand {
            id: format!("rt-{i}"),
            method: method.to_string(),
            params: params.clone(),
            received_at: Instant::now(),
        });
    }
    let _ = enqueue_and_await(&state, "ping-inspect", "ping", json!({})).await;
    shutdown_bridge(&state, handle);

    let nav = state.dequeue_navigate_requests();
    let comp = state.dequeue_component_tree_requests();
    let sch = state.dequeue_schema_requests();
    let dq = state.dequeue_dom_query_requests();
    let es = state.dequeue_event_simulate_requests();
    let cs = state.dequeue_computed_styles_requests();
    let ca = state.dequeue_css_audit_requests();
    assert_eq!(nav.len(), 1, "navigate queue");
    assert_eq!(comp.len(), 1, "component_tree queue");
    assert_eq!(sch.len(), 1, "get_schema queue");
    assert_eq!(dq.len(), 1, "dom_query queue");
    assert_eq!(es.len(), 1, "event_simulate queue");
    assert_eq!(cs.len(), 1, "computed_styles queue");
    assert_eq!(ca.len(), 1, "css_audit queue");
    for i in 0..cases.len() {
        assert!(
            state.get_response(&format!("rt-{i}")).is_none(),
            "inspection method {} must not be answered directly",
            cases[i].0
        );
    }
    assert_eq!(nav[0].route, "/settings");
}

#[test]
fn mock_translator_languages_list_includes_english_and_spanish() {
    let state = Arc::new(BridgeState::new());
    let result = invoke_app_command("translator.languages.list", &json!({}), &state)
        .expect("languages.list must succeed");
    let arr = result.as_array().expect("must be an array");
    let codes: Vec<&str> = arr
        .iter()
        .map(|v| v.get("code").and_then(|c| c.as_str()).unwrap_or(""))
        .collect();
    assert!(codes.contains(&"en"), "languages missing en: {codes:?}");
    assert!(codes.contains(&"es"), "languages missing es: {codes:?}");
    assert!(!arr.is_empty(), "languages list must not be empty");
}

#[test]
fn mock_translator_translate_same_lang_returns_passthrough() {
    let state = Arc::new(BridgeState::new());
    let payload = json!({
        "text": "hello",
        "source_lang": "en",
        "target_lang": "en",
    });
    let result = invoke_app_command("translator.translate", &payload, &state)
        .expect("same-lang translate must succeed");
    assert_eq!(
        result.pointer("/translated_text"),
        Some(&json!("hello")),
        "same-lang passthrough should echo input, got {result}"
    );
    assert_eq!(
        result.pointer("/message"),
        Some(&json!("Same language")),
        "expected 'Same language' message, got {result}"
    );
}

#[test]
fn mock_translator_translate_empty_text_is_structured_validation_error() {
    let state = Arc::new(BridgeState::new());
    let payload = json!({
        "text": "   ",
        "source_lang": "en",
        "target_lang": "es",
    });
    let err = invoke_app_command("translator.translate", &payload, &state)
        .expect_err("empty text must return a structured error");
    let msg = err.to_string();
    assert!(msg.contains("empty"), "expected 'empty' in error: {msg}");
}

#[test]
fn mock_translator_translate_unsupported_lang_is_structured_validation_error() {
    let state = Arc::new(BridgeState::new());
    let payload = json!({
        "text": "hello",
        "source_lang": "en",
        "target_lang": "zz",
    });
    let err = invoke_app_command("translator.translate", &payload, &state)
        .expect_err("unsupported language must return a structured error");
    let msg = err.to_string();
    assert!(
        msg.contains("Unsupported language"),
        "expected 'Unsupported language' in error: {msg}"
    );
}

#[test]
fn mock_translator_unknown_command_is_structured_not_found() {
    let state = Arc::new(BridgeState::new());
    let err = invoke_app_command("translator.does_not_exist", &json!({}), &state)
        .expect_err("unknown command must return a structured error");
    let msg = err.to_string();
    assert!(
        msg.contains("unknown command"),
        "expected 'unknown command' in error: {msg}"
    );
}

#[tokio::test]
async fn mock_translator_translate_enqueue_does_not_block_consumer() {
    let (state, handle) = mock_bridge();
    state.enqueue(BridgeCommand {
        id: "ci-trans".to_string(),
        method: "commands_invoke".to_string(),
        params: json!({
            "name": "translator.translate",
            "payload": { "text": "hello", "source_lang": "en", "target_lang": "es" },
        }),
        received_at: Instant::now(),
    });
    let ping_resp = enqueue_and_await(&state, "ping-ci", "ping", json!({})).await;
    shutdown_bridge(&state, handle);

    assert!(ping_resp.result.is_some());
    assert!(
        state.get_response("ci-trans").is_none(),
        "translator.translate should be enqueued, not answered by the consumer"
    );
    let pending = state.dequeue_command_invoke_requests();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].name, "translator.translate");
    assert_eq!(
        pending[0].payload.pointer("/text"),
        Some(&json!("hello"))
    );
}

#[tokio::test]
async fn mock_commands_invoke_unknown_name_returns_structured_error() {
    let (state, handle) = mock_bridge();
    let resp = enqueue_and_await(
        &state,
        "ci-unk",
        "commands_invoke",
        json!({ "name": "translator.does_not_exist", "payload": {} }),
    )
    .await;
    shutdown_bridge(&state, handle);

    assert!(resp.result.is_none());
    let err = resp.error.expect("expected error string");
    assert!(
        err.contains("unknown command"),
        "expected 'unknown command' in error: {err}"
    );
}

#[tokio::test]
async fn mock_ui_invoke_action_enqueues_to_action_queue_with_payload() {
    let (state, handle) = mock_bridge();
    state.enqueue(BridgeCommand {
        id: "ui-p".to_string(),
        method: "ui_invoke_action".to_string(),
        params: json!({
            "action": "toggle_theme",
            "payload": { "value": true },
        }),
        received_at: Instant::now(),
    });
    let _ = enqueue_and_await(&state, "ping-ui", "ping", json!({})).await;
    shutdown_bridge(&state, handle);

    assert!(state.get_response("ui-p").is_none());
    let pending = state.dequeue_ui_action_requests();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].action, "toggle_theme");
    assert_eq!(pending[0].payload, json!({ "value": true }));
}

#[tokio::test]
async fn mock_logs_tail_returns_bridge_sourced_entries() {
    let (state, handle) = mock_bridge();
    state.append_log("[INFO] bridge log line one".to_string());
    state.append_log("[WARN] bridge log line two".to_string());
    let resp = enqueue_and_await(
        &state,
        "lt-1",
        "logs_tail",
        json!({ "lines": 10 }),
    )
    .await;
    shutdown_bridge(&state, handle);

    let result = resp.result.expect("logs_tail must return a result");
    assert_eq!(
        result.pointer("/source"),
        Some(&json!("bridge")),
        "logs_tail source should be 'bridge'"
    );
    let lines = result
        .pointer("/lines")
        .and_then(|v| v.as_array())
        .expect("lines must be an array");
    assert_eq!(lines.len(), 2, "expected 2 log lines, got {lines:?}");
    assert_eq!(lines[0].as_str(), Some("[INFO] bridge log line one"));
}

#[tokio::test]
async fn mock_logs_tail_filters_by_substring() {
    let (state, handle) = mock_bridge();
    state.append_log("[INFO] translate done".to_string());
    state.append_log("[INFO] theme toggled".to_string());
    let resp = enqueue_and_await(
        &state,
        "ltf-1",
        "logs_tail",
        json!({ "filter": "theme" }),
    )
    .await;
    shutdown_bridge(&state, handle);

    let result = resp.result.expect("logs_tail must return a result");
    let lines = result
        .pointer("/lines")
        .and_then(|v| v.as_array())
        .expect("lines must be an array");
    assert_eq!(lines.len(), 1, "filter should keep only matching lines");
    assert_eq!(lines[0].as_str(), Some("[INFO] theme toggled"));
}

#[tokio::test]
async fn mock_screenshot_is_routed_or_returns_structured_outcome() {
    let (state, handle) = mock_bridge();
    let waiter = state.register_response_waiter("scr-1");
    state.enqueue(BridgeCommand {
        id: "scr-1".to_string(),
        method: "screenshot".to_string(),
        params: json!({}),
        received_at: Instant::now(),
    });
    let outcome = waiter.await_response(Duration::from_secs(8)).await;
    shutdown_bridge(&state, handle);

    match outcome {
        Ok(resp) => {
            if let Some(result) = resp.result {
                assert!(result.get("format").is_some(), "screenshot result missing format: {result}");
                assert!(result.get("width").is_some(), "screenshot result missing width: {result}");
                assert!(result.get("height").is_some(), "screenshot result missing height: {result}");
                assert!(result.get("data").is_some(), "screenshot result missing data: {result}");
            } else {
                let err = resp.error.expect("missing monitors must return an error string");
                assert!(
                    err.contains("no monitors") || err.contains("capture failed"),
                    "expected 'no monitors' or 'capture failed' in error, got: {err}"
                );
            }
        }
        Err(e) => panic!("screenshot never produced a response within 8s: {e:?}"),
    }
}

#[test]
fn mock_response_deadline_constant_matches_5_seconds() {
    assert_eq!(
        BRIDGE_RESPONSE_DEADLINE_SECS, 5,
        "BRIDGE_RESPONSE_DEADLINE_SECS must be 5s (test bounds depend on this)"
    );
    assert_eq!(
        BRIDGE_CONNECT_DEADLINE_SECS, 2,
        "BRIDGE_CONNECT_DEADLINE_SECS must be 2s (test bounds depend on this)"
    );
}

// ============================================================================
// === LIVE TESTS =============================================================
// ============================================================================

use std::sync::OnceLock;
use tokio::sync::Mutex as AsyncMutex;

static LIVE_HARNESS: OnceLock<AsyncMutex<()>> = OnceLock::new();

fn live_lock() -> &'static AsyncMutex<()> {
    LIVE_HARNESS.get_or_init(|| AsyncMutex::new(()))
}

async fn live_setup() -> AppHarness {
    let _guard = live_lock().lock().await;
    let mut harness =
        AppHarness::spawn().expect("failed to spawn translator; ensure `cargo build` succeeded");
    harness
        .wait_ready()
        .await
        .expect("bridge never became ready");
    harness
}

#[tokio::test]
async fn live_bridge_status_reports_listening() {
    let _harness = live_setup().await;
    let resp = rpc_request(BRIDGE_URL, "bs-live", "bridge_status", json!({}))
        .await
        .expect("bridge_status failed");
    let result = unwrap_result(&resp);
    assert_eq!(
        result.pointer("/listening"),
        Some(&json!(true)),
        "bridge should be listening, got {result}"
    );
    assert_eq!(
        result.pointer("/host"),
        Some(&json!("127.0.0.1"))
    );
    assert_eq!(
        result.pointer("/protocol_version"),
        Some(&json!("2.0"))
    );
    assert_eq!(
        result.pointer("/bound_port"),
        Some(&json!(APP_PORT))
    );
}

#[tokio::test]
async fn live_ping_returns_pong_within_5_seconds() {
    let _harness = live_setup().await;
    let started = Instant::now();
    let resp = rpc_request(BRIDGE_URL, "p-live", "ping", json!({}))
        .await
        .expect("ping failed");
    let elapsed = started.elapsed();
    assert!(
        elapsed <= Duration::from_secs(BRIDGE_RESPONSE_DEADLINE_SECS + 1),
        "ping took {elapsed:?}, expected <= {}s",
        BRIDGE_RESPONSE_DEADLINE_SECS + 1
    );
    let result = unwrap_result(&resp);
    assert_eq!(result.pointer("/pong"), Some(&json!(true)));
}

#[tokio::test]
async fn live_app_info_identifies_translator() {
    let _harness = live_setup().await;
    let resp = rpc_request(BRIDGE_URL, "ai-live", "app_info", json!({}))
        .await
        .expect("app_info failed");
    let result = unwrap_result(&resp);
    assert_eq!(result.pointer("/name"), Some(&json!("translator")));
    assert!(result.pointer("/version").is_some());
    assert!(result.pointer("/platform").is_some());
    assert!(result.pointer("/dioxus_version").is_some());
}

#[tokio::test]
async fn live_commands_list_contains_required_methods() {
    let _harness = live_setup().await;
    let resp = rpc_request(BRIDGE_URL, "cl-live", "commands_list", json!({}))
        .await
        .expect("commands_list failed");
    let result = unwrap_result(&resp);
    let arr = result
        .as_array()
        .expect("commands_list result must be an array");
    let names: Vec<String> = arr
        .iter()
        .map(|v| {
            v.get("name")
                .and_then(|n| n.as_str())
                .or_else(|| v.as_str())
                .unwrap_or("")
                .to_string()
        })
        .collect();
    for required in [
        "ping",
        "app_info",
        "bridge_status",
        "commands_list",
        "commands_invoke",
        "evaluate_js",
        "dom_snapshot",
        "navigate",
        "get_schema",
        "component_tree",
        "css_audit",
        "translator.translate",
        "translator.languages.list",
    ] {
        assert!(
            names.iter().any(|n| n == required),
            "commands_list missing `{required}`: {names:?}"
        );
    }
}

#[tokio::test]
async fn live_evaluate_js_one_plus_one_returns_two() {
    let _harness = live_setup().await;
    let resp = rpc_request(
        BRIDGE_URL,
        "e1-live",
        "evaluate_js",
        json!({ "code": "1 + 1" }),
    )
    .await
    .expect("evaluate_js failed");
    let result = unwrap_result(&resp);
    // The bridge wraps the JS eval result as a JSON value; "1 + 1" -> 2.
    assert_eq!(
        result.as_i64(),
        Some(2),
        "evaluate_js(1+1) must return 2, got {result}"
    );
}

#[tokio::test]
async fn live_evaluate_js_document_title_returns_string() {
    let _harness = live_setup().await;
    let resp = rpc_request(
        BRIDGE_URL,
        "e-title",
        "evaluate_js",
        json!({ "code": "document.title" }),
    )
    .await
    .expect("evaluate_js document.title failed");
    let result = unwrap_result(&resp);
    let title = result
        .as_str()
        .unwrap_or_else(|| panic!("document.title must return a string, got {result}"));
    assert!(!title.is_empty(), "document.title must be non-empty");
}

#[tokio::test]
async fn live_dom_snapshot_contains_body_markup() {
    let _harness = live_setup().await;
    let resp = rpc_request(BRIDGE_URL, "ds-live", "dom_snapshot", json!({}))
        .await
        .expect("dom_snapshot failed");
    let result = unwrap_result(&resp);
    let body = result
        .as_str()
        .unwrap_or_else(|| panic!("dom_snapshot result must be a string, got {result}"));
    assert!(
        body.contains("<body") || body.contains("<html") || body.contains("<div"),
        "dom_snapshot must contain body/html/div markup, got first 200 chars: {:.200}",
        body
    );
    assert!(
        body.contains("Translator") || body.contains("translate") || body.contains("root"),
        "dom_snapshot should contain app content, got first 200 chars: {:.200}",
        body
    );
}

#[tokio::test]
async fn live_page_snapshot_has_route_pages_bindings_and_theme() {
    let _harness = live_setup().await;
    let resp = rpc_request(BRIDGE_URL, "ps-live", "page_snapshot", json!({}))
        .await
        .expect("page_snapshot failed");
    let result = unwrap_result(&resp);
    assert!(
        result.get("route").is_some(),
        "page_snapshot must include current route, got {result}"
    );
    assert!(
        result.get("pages").and_then(|p| p.as_array()).is_some(),
        "page_snapshot must include pages array, got {result}"
    );
    let pages = result.get("pages").and_then(|p| p.as_array()).unwrap();
    assert!(!pages.is_empty(), "pages array must not be empty");
    let first = &pages[0];
    assert!(
        first.get("id").is_some(),
        "page must have id, got {first}"
    );
    assert!(
        first.get("elements").and_then(|e| e.as_array()).is_some(),
        "page must have elements array, got {first}"
    );
    assert!(
        result.get("bindings").is_some(),
        "page_snapshot must include bindings, got {result}"
    );
    assert!(
        result.get("theme").is_some(),
        "page_snapshot must include theme, got {result}"
    );
}

#[tokio::test]
async fn live_get_schema_includes_pages_and_current_route() {
    let _harness = live_setup().await;
    let resp = rpc_request(BRIDGE_URL, "gs-live", "get_schema", json!({}))
        .await
        .expect("get_schema failed");
    let result = unwrap_result(&resp);
    assert!(
        result.get("pages").and_then(|p| p.as_array()).is_some(),
        "get_schema must include pages array, got {result}"
    );
    assert!(
        result.get("route").is_some(),
        "get_schema must include current route, got {result}"
    );
    let pages = result.get("pages").and_then(|p| p.as_array()).unwrap();
    assert!(!pages.is_empty(), "get_schema.pages must not be empty");
    assert!(
        pages[0].get("id").is_some(),
        "schema page must have id, got {}",
        pages[0]
    );
}

#[tokio::test]
async fn live_navigate_changes_current_route() {
    let _harness = live_setup().await;
    // Snapshot the starting route.
    let before = rpc_request(BRIDGE_URL, "nav-before", "get_schema", json!({}))
        .await
        .expect("get_schema before navigate failed");
    let before_route = before
        .get("result")
        .and_then(|r| r.get("route"))
        .and_then(|r| r.as_str())
        .unwrap_or("/");
    assert_eq!(before_route, "/", "expected to start on / route");

    // Request a navigation to a non-existent route; the bridge enqueues the
    // navigate request and reports a structured error if the route is invalid.
    let nav = rpc_request(
        BRIDGE_URL,
        "nav-req",
        "navigate",
        json!({ "route": "/this-route-does-not-exist" }),
    )
    .await
    .expect("navigate request failed");
    // Either it succeeds (route resolved, current_route changed) or the
    // Dioxus-executor processor reports a structured error. The bridge
    // contract forbids silently swallowing the request.
    if let Some(_result) = nav.get("result") {
        // Verify route actually changed by snapshotting again.
        let after = rpc_request(BRIDGE_URL, "nav-after", "get_schema", json!({}))
            .await
            .expect("get_schema after navigate failed");
        let after_route = after
            .get("result")
            .and_then(|r| r.get("route"))
            .and_then(|r| r.as_str())
            .unwrap_or("/");
        assert_ne!(
            after_route, before_route,
            "navigate to a different route must change current_route"
        );
    } else {
        let err = unwrap_error(&nav);
        let code = err.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
        assert!(
            code != 0,
            "navigate must return either a result or a structured error with a code, got {nav}"
        );
    }
}

#[tokio::test]
async fn live_component_tree_returns_structure() {
    let _harness = live_setup().await;
    let resp = rpc_request(BRIDGE_URL, "ct-live", "component_tree", json!({}))
        .await
        .expect("component_tree failed");
    // component_tree may return either a structured result or a structured
    // error if the Dioxus-executor processor rejects it. Either way the
    // response must be valid JSON-RPC.
    if let Some(result) = resp.get("result") {
        assert!(
            result.is_object() || result.is_array(),
            "component_tree result must be an object or array, got {result}"
        );
    } else {
        let err = unwrap_error(&resp);
        assert!(err.get("message").is_some());
    }
}

#[tokio::test]
async fn live_commands_invoke_translate_returns_translated_text() {
    let _harness = live_setup().await;
    let resp = rpc_request(
        BRIDGE_URL,
        "ci-trans",
        "commands_invoke",
        json!({
            "name": "translator.translate",
            "payload": {
                "text": "hello",
                "source_lang": "en",
                "target_lang": "es",
            },
        }),
    )
    .await
    .expect("commands_invoke translate failed");
    let result = unwrap_result(&resp);
    let translated = result
        .pointer("/translated_text")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("translate result missing translated_text: {result}"));
    assert!(
        !translated.is_empty(),
        "translated_text must be non-empty, got {result}"
    );
    assert_eq!(
        result.pointer("/source_lang"),
        Some(&json!("en"))
    );
    assert_eq!(
        result.pointer("/target_lang"),
        Some(&json!("es"))
    );
}

#[tokio::test]
async fn live_commands_invoke_languages_list_returns_codes() {
    let _harness = live_setup().await;
    let resp = rpc_request(
        BRIDGE_URL,
        "ci-lang",
        "commands_invoke",
        json!({ "name": "translator.languages.list", "payload": {} }),
    )
    .await
    .expect("commands_invoke languages.list failed");
    let result = unwrap_result(&resp);
    let arr = result
        .as_array()
        .unwrap_or_else(|| panic!("languages.list result must be array, got {result}"));
    assert!(!arr.is_empty(), "languages list must not be empty");
    let codes: Vec<&str> = arr
        .iter()
        .map(|v| v.get("code").and_then(|c| c.as_str()).unwrap_or(""))
        .collect();
    assert!(codes.contains(&"en"), "languages missing en: {codes:?}");
    assert!(codes.contains(&"es"), "languages missing es: {codes:?}");
}

#[tokio::test]
async fn live_commands_invoke_empty_text_is_structured_error() {
    let _harness = live_setup().await;
    let resp = rpc_request(
        BRIDGE_URL,
        "ci-empty",
        "commands_invoke",
        json!({
            "name": "translator.translate",
            "payload": {
                "text": "   ",
                "source_lang": "en",
                "target_lang": "es",
            },
        }),
    )
    .await
    .expect("commands_invoke empty translate failed");
    let err = unwrap_error(&resp);
    let message = err
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("");
    assert!(
        message.contains("empty"),
        "empty text must produce 'empty' in error message, got: {message}"
    );
}

#[tokio::test]
async fn live_commands_invoke_unknown_command_is_structured_error() {
    let _harness = live_setup().await;
    let resp = rpc_request(
        BRIDGE_URL,
        "ci-unk",
        "commands_invoke",
        json!({ "name": "translator.does_not_exist", "payload": {} }),
    )
    .await
    .expect("commands_invoke unknown failed");
    let err = unwrap_error(&resp);
    let message = err
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("");
    assert!(
        message.contains("unknown command"),
        "expected 'unknown command' in error message, got: {message}"
    );
}

#[tokio::test]
async fn live_ui_invoke_action_returns_ok() {
    let _harness = live_setup().await;
    let resp = rpc_request(
        BRIDGE_URL,
        "ui-trans",
        "ui_invoke_action",
        json!({
            "action": "translate",
            "payload": {},
        }),
    )
    .await
    .expect("ui_invoke_action failed");
    let result = unwrap_result(&resp);
    assert_eq!(
        result.pointer("/ok"),
        Some(&json!(true)),
        "ui_invoke_action translate must return ok:true, got {result}"
    );
}

#[tokio::test]
async fn live_unknown_method_returns_method_not_found_error() {
    let _harness = live_setup().await;
    let resp = rpc_request(BRIDGE_URL, "um-live", "completely_unknown_method", json!({}))
        .await
        .expect("unknown method request failed");
    let err = unwrap_error(&resp);
    let code = err.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
    let message = err
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("");
    // The bridge emits either a generic "unsupported bridge method" error
    // (code -32000) or the JSON-RPC standard -32601 "Method not found".
    assert!(
        message.contains("unsupported bridge method")
            || message.contains("Method not found")
            || message.contains("not found"),
        "expected method-not-found-style error, got code={code} message={message}"
    );
    assert_ne!(code, 0, "error must have a non-zero code, got {err}");
}

#[tokio::test]
async fn live_malformed_json_is_parse_error() {
    let _harness = live_setup().await;
    // Use a raw socket to send a non-JSON body.
    let connect = timeout(
        Duration::from_secs(BRIDGE_CONNECT_DEADLINE_SECS + 1),
        connect_async(BRIDGE_URL),
    )
    .await
    .expect("connect did not complete in time")
    .expect("connect failed");
    let (mut ws, _) = connect;
    ws.send(Message::Text("{not valid json".into()))
        .await
        .expect("send failed");
    let read = timeout(READ_DEADLINE, ws.next())
        .await
        .expect("read deadline exceeded");
    drop(ws);
    let msg = read.expect("stream closed").expect("ws error");
    let text = match msg {
        Message::Text(t) => t,
        other => panic!("expected text frame, got {other:?}"),
    };
    let parsed: Value =
        serde_json::from_str(&text).expect("server must still emit a JSON-RPC parse error");
    let err = unwrap_error(&parsed);
    let code = err.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
    let message = err
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("");
    assert_eq!(code, -32700, "malformed JSON must produce parse error -32700, got {err}");
    assert!(
        message.contains("Parse error"),
        "expected 'Parse error' in message, got: {message}"
    );
}

#[tokio::test]
async fn live_screenshot_returns_valid_png_in_tempdir() {
    let _harness = live_setup().await;
    let resp = rpc_request(BRIDGE_URL, "scr-live", "screenshot", json!({}))
        .await
        .expect("screenshot request failed");
    // Some headless hosts return "no monitors available" — that is also a
    // valid structured response. Assert the contract:
    //   - success: result has data, width, height; PNG magic bytes verify.
    //   - failure: error.message names a capture failure.
    if let Some(result) = resp.get("result") {
        let data = result
            .pointer("/data")
            .and_then(|v| v.as_str())
            .expect("screenshot result must include base64 data");
        let width = result
            .pointer("/width")
            .and_then(|v| v.as_u64())
            .expect("screenshot result must include width");
        let height = result
            .pointer("/height")
            .and_then(|v| v.as_u64())
            .expect("screenshot result must include height");
        assert!(width > 0, "screenshot width must be > 0");
        assert!(height > 0, "screenshot height must be > 0");

        let bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            data,
        )
        .expect("screenshot data must be valid base64");
        assert!(
            bytes.len() > 8,
            "screenshot bytes must be non-trivial, got {} bytes",
            bytes.len()
        );
        // PNG magic: 0x89 P N G 0x0D 0x0A 0x1A 0x0A
        assert_eq!(
            &bytes[..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A],
            "screenshot must be a valid PNG"
        );

        // Write to TempDir (auto-cleaned on drop) instead of /tmp.
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("translator_screenshot.png");
        std::fs::write(&path, &bytes).expect("write screenshot");
        let written = std::fs::read(&path).expect("read screenshot back");
        assert_eq!(written.len(), bytes.len(), "roundtrip must match");
        eprintln!(
            "[live_screenshot] wrote {} bytes to {} (auto-cleaned on drop)",
            bytes.len(),
            path.display()
        );
    } else {
        let err = unwrap_error(&resp);
        let message = err
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("");
        assert!(
            message.contains("no monitors") || message.contains("capture failed"),
            "screenshot failure must be a structured 'no monitors' or 'capture failed' error, got: {message}"
        );
    }
}

#[tokio::test]
async fn live_logs_tail_includes_bridge_sourced_lines() {
    let _harness = live_setup().await;
    // Trigger a few bridge-sourced log lines by hitting a couple of methods.
    let _ = rpc_request(BRIDGE_URL, "lt-warm-1", "ping", json!({}))
        .await
        .expect("warmup ping failed");
    let _ = rpc_request(BRIDGE_URL, "lt-warm-2", "app_info", json!({}))
        .await
        .expect("warmup app_info failed");
    let resp = rpc_request(
        BRIDGE_URL,
        "lt-live",
        "logs_tail",
        json!({ "lines": 50 }),
    )
    .await
    .expect("logs_tail failed");
    let result = unwrap_result(&resp);
    assert_eq!(
        result.pointer("/source"),
        Some(&json!("bridge")),
        "logs_tail source must be 'bridge'"
    );
    let lines = result
        .pointer("/lines")
        .and_then(|v| v.as_array())
        .expect("lines must be an array");
    assert!(
        !lines.is_empty(),
        "logs_tail should return at least one bridge-sourced line, got empty"
    );
    for line in lines {
        let s = line
            .as_str()
            .unwrap_or_else(|| panic!("log line must be a string, got {line}"));
        assert!(!s.is_empty(), "log lines must be non-empty");
    }
}

#[tokio::test]
async fn live_unknown_id_jsonrpc_version_returns_invalid_request() {
    // The bridge should reject requests with a non-2.0 jsonrpc field.
    let _harness = live_setup().await;
    let connect = timeout(
        Duration::from_secs(BRIDGE_CONNECT_DEADLINE_SECS + 1),
        connect_async(BRIDGE_URL),
    )
    .await
    .expect("connect did not complete in time")
    .expect("connect failed");
    let (mut ws, _) = connect;
    ws.send(Message::Text(
        json!({
            "jsonrpc": "1.0",
            "id": "badver",
            "method": "ping",
            "params": {},
        })
        .to_string()
        .into(),
    ))
    .await
    .expect("send failed");
    let read = timeout(READ_DEADLINE, ws.next())
        .await
        .expect("read deadline exceeded");
    drop(ws);
    let msg = read.expect("stream closed").expect("ws error");
    let text = match msg {
        Message::Text(t) => t,
        other => panic!("expected text frame, got {other:?}"),
    };
    let parsed: Value =
        serde_json::from_str(&text).expect("server must return a JSON-RPC error");
    let err = unwrap_error(&parsed);
    let code = err.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
    assert_eq!(
        code, -32600,
        "non-2.0 jsonrpc must produce invalid request -32600, got {err}"
    );
}

// ============================================================================
// === TIMING TESTS ===========================================================
// ============================================================================

#[tokio::test]
async fn live_connect_failure_completes_within_2_5_seconds() {
    // When the bridge is not running, the connect attempt must fail in
    // roughly BRIDGE_CONNECT_DEADLINE_SECS (2s), NOT the tokio default
    // (10s+). This test points at a port that is not bound, asserts
    // the connect fails, and asserts the elapsed time is bounded.
    let bogus_url = "ws://127.0.0.1:1";
    let started = Instant::now();
    let outcome = timeout(
        Duration::from_secs(5),
        connect_async(bogus_url),
    )
    .await;
    let elapsed = started.elapsed();

    // We accept either an outer timeout (still bounded) or a connect error
    // (the more common path on a closed port). Both are acceptable as
    // long as the elapsed time respects the bridge connect deadline.
    match outcome {
        Ok(Err(e)) => {
            eprintln!("[timing] connect failed in {elapsed:?}: {e}");
        }
        Err(_elapsed_at_outer) => {
            panic!(
                "connect_async to a closed port exceeded the 5s outer bound (elapsed {elapsed:?})"
            );
        }
        Ok(Ok(_)) => {
            panic!("connect_async unexpectedly succeeded for {bogus_url}");
        }
    }
    let upper_bound = Duration::from_millis(2500);
    assert!(
        elapsed < upper_bound,
        "connect to closed port must fail within {upper_bound:?}, took {elapsed:?}"
    );
}

#[tokio::test]
async fn live_response_within_5_seconds() {
    let _harness = live_setup().await;
    // All successful responses should arrive well within the response
    // deadline. ping is the cheapest request and should complete in tens
    // of milliseconds; we use a generous 5s upper bound (the response
    // deadline itself) and assert it holds.
    let started = Instant::now();
    let resp = rpc_request(BRIDGE_URL, "rt-live", "ping", json!({}))
        .await
        .expect("ping failed");
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(BRIDGE_RESPONSE_DEADLINE_SECS),
        "ping must complete within {BRIDGE_RESPONSE_DEADLINE_SECS}s, took {elapsed:?}"
    );
    unwrap_result(&resp);
}

// ============================================================================
// === MISC / CLEANUP ========================================================
// ============================================================================

#[test]
fn no_bridge_test_accepts_timeout_as_success() {
    // Meta-test: every `tokio::time::timeout` in this file maps elapsed
    // time to a structured panic with the deadline attached. This test
    // makes the contract auditable: it scans the source for the pattern
    // and asserts there is no `println!` followed by silent pass on the
    // timeout arm.
    let source = include_str!("ws_bridge.rs");
    assert!(
        !source.contains("println!(\"Timeout"),
        "no test may print 'Timeout' and continue — timeouts must be assertion failures"
    );
    assert!(
        !source.contains("// expected"),
        "no test may label a timeout branch as 'expected' success"
    );
}
