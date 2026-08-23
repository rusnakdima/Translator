//! WebSocket regression test - verifies evaluate_js, dom_snapshot, screenshot
//! return within 500ms instead of 5000ms timeout.

use futures_util::{SinkExt, StreamExt};
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    println!("Connecting to ws://127.0.0.1:9223...");
    let (ws, _) = connect_async("ws://127.0.0.1:9223")
        .await
        .expect("Failed to connect to bridge");
    let (mut write, mut read) = ws.split();

    let mut results = Vec::new();

    // Test 1: ping (should be ~5ms)
    let result = send_and_receive(&mut write, &mut read, "1", "ping", r#"{"code":"","selector":""}"#, 500).await;
    results.push(result);

    // Test 2: app_info (should be ~25ms)
    let result = send_and_receive(&mut write, &mut read, "2", "app_info", r#"{"code":"","selector":""}"#, 500).await;
    results.push(result);

    // Test 3: evaluate_js with code "1 + 1" (critical test - was 5000ms timeout, should be <500ms now)
    let result = send_and_receive(&mut write, &mut read, "3", "evaluate_js", r#"{"code":"1 + 1","selector":""}"#, 500).await;
    results.push(result);

    // Test 4: dom_snapshot with selector "body" (critical test - was 5000ms timeout, should be <500ms now)
    let result = send_and_receive(&mut write, &mut read, "4", "dom_snapshot", r#"{"code":"","selector":"body"}"#, 500).await;
    results.push(result);

    // Test 5: screenshot (should be <500ms now)
    let result = send_and_receive(&mut write, &mut read, "5", "screenshot", r#"{"code":"","selector":""}"#, 500).await;
    results.push(result);

    // Test 6: commands_list
    let result = send_and_receive(&mut write, &mut read, "6", "commands_list", r#"{"code":"","selector":""}"#, 500).await;
    results.push(result);

    // Print results
    println!("\n=== RESULTS ===");
    let mut ok_count = 0;
    let mut error_count = 0;
    let mut timeout_count = 0;

    for r in &results {
        print!("Test: {:30s} | ", r.test_name);
        if r.timed_out {
            println!("TIMEOUT ({}ms) ❌", r.elapsed_ms);
            timeout_count += 1;
        } else if r.passed {
            println!("PASSED ({}ms) ✅", r.elapsed_ms);
            ok_count += 1;
        } else {
            println!("ERROR: {} ❌", r.error_msg);
            error_count += 1;
        }
    }

    println!("\nSummary: {} ok, {} error, {} timeout", ok_count, error_count, timeout_count);

    // Output JSON for evidence
    let json = serde_json::json!({
        "test": "translator-mcp-bridge-fix Todo 19 live regression v2",
        "target": "ws://127.0.0.1:9223",
        "timestamp": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "summary": {
            "ok": ok_count,
            "error": error_count,
            "timeout": timeout_count
        },
        "critical_checks": {
            "evaluate_js_under_500ms": results[2].passed && results[2].elapsed_ms < 500.0,
            "evaluate_js_time_ms": results[2].elapsed_ms,
            "dom_snapshot_under_500ms": results[3].passed && results[3].elapsed_ms < 500.0,
            "dom_snapshot_time_ms": results[3].elapsed_ms,
            "screenshot_under_500ms": results[4].passed && results[4].elapsed_ms < 500.0,
            "screenshot_time_ms": results[4].elapsed_ms,
            "ping_under_500ms": results[0].passed && results[0].elapsed_ms < 500.0,
            "ping_time_ms": results[0].elapsed_ms,
            "app_info_under_500ms": results[1].passed && results[1].elapsed_ms < 500.0,
            "app_info_time_ms": results[1].elapsed_ms,
        },
        "results": results,
    });

    let json_str = serde_json::to_string_pretty(&json).unwrap();
    println!("\n=== JSON EVIDENCE ===\n{}", json_str);

    std::fs::write("/home/dmitriy/Projects/.omo/evidence/translator-mcp-bridge-fix/task-19-live-regression-v2.json", &json_str).unwrap();
    println!("\nSaved to /home/dmitriy/Projects/.omo/evidence/translator-mcp-bridge-fix/task-19-live-regression-v2.json");
}

struct TestResult {
    test_name: String,
    method: String,
    passed: bool,
    elapsed_ms: f64,
    timed_out: bool,
    error_msg: String,
    response: String,
}

async fn send_and_receive(
    write: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, Message>,
    read: &mut futures_util::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>,
    id: &str,
    method: &str,
    params: &str,
    timeout_ms: u64,
) -> TestResult {
    let request = format!(
        r#"{{"jsonrpc":"2.0","method":"{}","params":{},"id":"{}"}}"#,
        method, id, params
    );

    let start = Instant::now();
    write.send(Message::Text(request.into())).await.expect("send failed");
    write.flush().await.expect("flush failed");

    let timeout = tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), read.next()).await;

    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    match timeout {
        Ok(Some(Ok(msg))) => {
            let text = match msg {
                Message::Text(t) => t.to_string(),
                Message::Binary(data) => String::from_utf8_lossy(&data).to_string(),
                other => format!("{:?}", other),
            };

            let passed = text.contains("result") && !text.contains("Parse error");
            let error_msg = if text.contains("\"error\"") {
                text.clone()
            } else {
                String::new()
            };

            TestResult {
                test_name: format!("{} ({})", method, id),
                method: method.to_string(),
                passed,
                elapsed_ms,
                timed_out: false,
                error_msg: if error_msg.is_empty() && !passed {
                    "Response missing result or has error".to_string()
                } else {
                    error_msg
                },
                response: text,
            }
        }
        Ok(None) => TestResult {
            test_name: format!("{} ({})", method, id),
            method: method.to_string(),
            passed: false,
            elapsed_ms,
            timed_out: false,
            error_msg: "Stream ended".to_string(),
            response: String::new(),
        },
        Ok(Some(Err(e))) => TestResult {
            test_name: format!("{} ({})", method, id),
            method: method.to_string(),
            passed: false,
            elapsed_ms,
            timed_out: false,
            error_msg: format!("WS error: {}", e),
            response: String::new(),
        },
        Err(_) => TestResult {
            test_name: format!("{} ({})", method, id),
            method: method.to_string(),
            passed: false,
            elapsed_ms,
            timed_out: true,
            error_msg: format!("Timeout after {}ms", timeout_ms),
            response: String::new(),
        },
    }
}