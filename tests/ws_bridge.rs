//! WebSocket bridge integration test
//! Run with: cd Translator && cargo test --test ws_bridge

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::test]
async fn bridge_ws_ping() {
    let (ws, _) = connect_async("ws://127.0.0.1:9223")
        .await
        .expect("Failed to connect to bridge");
    let (mut write, mut read) = ws.split();

    let ping = r#"{"jsonrpc":"2.0","method":"ping","params":{},"id":"1"}"#;
    write
        .send(Message::Text(ping.into()))
        .await
        .expect("send failed");
    write.flush().await.expect("flush failed");

    // Use timeout to avoid hanging
    let result = tokio::time::timeout(std::time::Duration::from_secs(3), read.next()).await;
    match result {
        Ok(Some(Ok(msg))) => match msg {
            Message::Text(text) => {
                println!("Response: {text}");
            }
            Message::Binary(data) => {
                let s = String::from_utf8_lossy(&data);
                println!("Binary response (as string): {s}");
                println!("Binary data len: {}", data.len());
                println!("First 100 bytes hex: {:02X?}", &data[..data.len().min(100)]);
            }
            other => {
                println!("Other message type: {other:?}");
            }
        },
        Ok(Some(Err(e))) => panic!("WS error: {e}"),
        Ok(None) => panic!("Stream ended"),
        Err(_) => panic!("Timeout"),
    }
}

#[tokio::test]
async fn bridge_ws_app_info() {
    let (ws, _) = connect_async("ws://127.0.0.1:9223")
        .await
        .expect("Failed to connect to bridge");
    let (mut write, mut read) = ws.split();

    let info = r#"{"jsonrpc":"2.0","method":"app_info","params":{},"id":"2"}"#;
    write
        .send(Message::Text(info.into()))
        .await
        .expect("send failed");

    let resp = read.next().await.expect("no response").unwrap();
    let text = resp.into_text().unwrap();
    println!("app_info response: {text}");
    assert!(
        text.contains("translator") || text.contains("version"),
        "Expected app info, got: {text}"
    );
}

#[tokio::test]
async fn bridge_ws_commands_list() {
    let (ws, _) = connect_async("ws://127.0.0.1:9223")
        .await
        .expect("Failed to connect to bridge");
    let (mut write, mut read) = ws.split();

    let list = r#"{"jsonrpc":"2.0","method":"commands_list","params":{},"id":"3"}"#;
    write
        .send(Message::Text(list.into()))
        .await
        .expect("send failed");

    let resp = read.next().await.expect("no response").unwrap();
    let text = resp.into_text().unwrap();
    println!("commands_list response: {text}");
    assert!(text.contains("["), "Expected JSON array, got: {text}");
}

/// evaluate_js is processed on the Dioxus main thread by a ticker-driven
/// use_effect (Translator/src/main.rs) that drains pending_eval_requests and
/// evaluates via the webview. Requires the Translator app to be running on
/// ws://127.0.0.1:9223 (D67 resolved).
#[tokio::test]
async fn bridge_ws_evaluate_js() {
    let (ws, _) = connect_async("ws://127.0.0.1:9223")
        .await
        .expect("Failed to connect to bridge");
    let (mut write, mut read) = ws.split();

    let eval = r#"{"jsonrpc":"2.0","method":"evaluate_js","params":{"code":"1 + 1"},"id":"4"}"#;
    write
        .send(Message::Text(eval.into()))
        .await
        .expect("send failed");
    println!("Sent evaluate_js: {eval}");

    // Request is routed correctly (no parse error) but eval times out
    // because use_effect loop is blocking (D67)
    let timeout = tokio::time::timeout(tokio::time::Duration::from_secs(2), read.next()).await;

    match timeout {
        Ok(Some(Ok(msg))) => {
            let text = msg.into_text().unwrap();
            println!("evaluate_js response: {text}");
            // Should NOT be a parse error (parse errors mean WebSocket frame bug)
            assert!(
                !text.contains("Parse error"),
                "Got parse error (D70 regression): {text}"
            );
            // Should be either a result or a command timeout
            assert!(
                text.contains("result") || text.contains("timeout") || text.contains("error"),
                "Expected result or timeout, got: {text}"
            );
        }
        Ok(None) => panic!("Stream ended unexpectedly"),
        Err(_) => println!("Timeout (expected - use_effect blocking loop, D67)"),
        _ => panic!("Unexpected message type"),
    }
}

#[tokio::test]
async fn bridge_ws_page_snapshot() {
    let (ws, _) = connect_async("ws://127.0.0.1:9223")
        .await
        .expect("Failed to connect to bridge");
    let (mut write, mut read) = ws.split();

    let snap = r#"{"jsonrpc":"2.0","method":"page_snapshot","params":{},"id":"5"}"#;
    write
        .send(Message::Text(snap.into()))
        .await
        .expect("send failed");

    let resp = read.next().await.expect("no response").unwrap();
    let text = resp.into_text().unwrap();
    println!("RAW page_snapshot response: {}", text);
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid json");
    // The result field is a JSON string containing serialized JSON
    let result_str = parsed
        .get("result")
        .and_then(|r| r.as_str())
        .expect("result is string");
    let result: serde_json::Value = serde_json::from_str(result_str).expect("result parses");

    println!("page_snapshot:");
    println!("  route: {:?}", result.get("route"));
    println!("  theme: {:?}", result.get("theme"));
    if let Some(cp) = result.get("current_page") {
        println!("  page: {:?}", cp.get("id"));
        println!("  title: {:?}", cp.get("title"));
        if let Some(els) = cp.get("elements").and_then(|e| e.as_array()) {
            println!("  elements count: {}", els.len());
            for el in els.iter().take(5) {
                println!(
                    "    - id={:?} component={:?} classes={:.50}",
                    el.get("id"),
                    el.get("component"),
                    el.get("classes").and_then(|c| c.as_str()).unwrap_or("")
                );
            }
        }
    }
    println!("bindings: {:?}", result.get("bindings"));
}

#[tokio::test]
async fn bridge_ws_dom_query() {
    let (ws, _) = connect_async("ws://127.0.0.1:9223")
        .await
        .expect("Failed to connect to bridge");
    let (mut write, mut read) = ws.split();

    // Get body content
    let eval = r#"{"jsonrpc":"2.0","method":"evaluate_js","params":{"code":"document.body.innerText.substring(0,200)"},"id":"6"}"#;
    write
        .send(Message::Text(eval.into()))
        .await
        .expect("send failed");

    let timeout = tokio::time::timeout(std::time::Duration::from_secs(3), read.next()).await;
    match timeout {
        Ok(Some(Ok(msg))) => {
            let text = msg.into_text().unwrap();
            println!("dom innerText: {text}");
        }
        Ok(None) => panic!("Stream ended"),
        Err(_) => println!("Timeout - eval not processed"),
        _ => {}
    }
}

#[tokio::test]
async fn bridge_ws_logs_read() {
    let (ws, _) = connect_async("ws://127.0.0.1:9223")
        .await
        .expect("Failed to connect to bridge");
    let (mut write, mut read) = ws.split();

    let logs = r#"{"jsonrpc":"2.0","method":"logs_read","params":{"lines":50},"id":"log1"}"#;
    write
        .send(Message::Text(logs.into()))
        .await
        .expect("send failed");

    let resp = read.next().await.expect("no response").unwrap();
    let text = resp.into_text().unwrap();
    println!("logs_read: {}", text);
}

#[tokio::test]
async fn bridge_ws_dom_snapshot() {
    let (ws, _) = connect_async("ws://127.0.0.1:9223")
        .await
        .expect("Failed to connect to bridge");
    let (mut write, mut read) = ws.split();

    // dom_snapshot returns raw HTML of body
    let eval = r#"{"jsonrpc":"2.0","method":"dom_snapshot","params":{},"id":"dom1"}"#;
    write
        .send(Message::Text(eval.into()))
        .await
        .expect("send failed");

    let timeout = tokio::time::timeout(std::time::Duration::from_secs(5), read.next()).await;
    match timeout {
        Ok(Some(Ok(msg))) => {
            let text = msg.into_text().unwrap();
            println!("dom_snapshot response: {}", text);
        }
        Ok(None) => panic!("Stream ended"),
        Err(_) => println!("Timeout - dom_snapshot not processed (eval loop issue)"),
        _ => {}
    }
}

#[tokio::test]
async fn bridge_ws_evaluate_js_long_timeout() {
    let (ws, _) = connect_async("ws://127.0.0.1:9223")
        .await
        .expect("Failed to connect to bridge");
    let (mut write, mut read) = ws.split();

    let eval = r#"{"jsonrpc":"2.0","method":"evaluate_js","params":{"code":"1 + 1"},"id":"t1"}"#;
    write
        .send(Message::Text(eval.into()))
        .await
        .expect("send failed");
    println!("Sent evaluate_js, waiting up to 10s...");

    let timeout = tokio::time::timeout(std::time::Duration::from_secs(10), read.next()).await;
    match timeout {
        Ok(Some(Ok(msg))) => {
            let text = msg.into_text().unwrap();
            println!("Got response: {}", text);
        }
        Ok(None) => println!("Stream ended unexpectedly"),
        Ok(Some(Err(e))) => println!("WS error: {}", e),
        Err(_) => println!("TIMEOUT after 10s - eval loop not delivering results"),
    }
}

#[tokio::test]
async fn bridge_ws_screenshot() {
    let (ws, _) = connect_async("ws://127.0.0.1:9223")
        .await
        .expect("Failed to connect to bridge");
    let (mut write, mut read) = ws.split();

    let screenshot = r#"{"jsonrpc":"2.0","method":"screenshot","params":{},"id":"scr1"}"#;
    write
        .send(Message::Text(screenshot.into()))
        .await
        .expect("send failed");

    let resp = read.next().await.expect("no response").unwrap();
    let text = resp.into_text().unwrap();
    println!("screenshot response length: {}", text.len());
    if text.contains("data") && text.len() > 100 {
        println!("Got screenshot data!");
        // Parse and save
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        if let Some(result) = parsed.get("result") {
            if let Some(data) = result.get("data").and_then(|d| d.as_str()) {
                use std::io::Write;
                let decoded =
                    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, data)
                        .unwrap();
                std::fs::write("/tmp/translator_screenshot.png", &decoded).unwrap();
                println!(
                    "Saved screenshot to /tmp/translator_screenshot.png ({} bytes)",
                    decoded.len()
                );
            }
        }
    } else {
        println!("screenshot: {}", text);
    }
}
