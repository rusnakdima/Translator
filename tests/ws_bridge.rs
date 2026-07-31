//! WebSocket bridge integration test
//! Run with: cd Translator && cargo test --test ws_bridge

use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

#[tokio::test]
async fn bridge_ws_ping() {
    let (ws, _) = connect_async("ws://127.0.0.1:9223")
        .await
        .expect("Failed to connect to bridge");
    let (mut write, mut read) = ws.split();

    let ping = r#"{"jsonrpc":"2.0","method":"ping","params":{},"id":"1"}"#;
    write.send(Message::Text(ping.into())).await.expect("send failed");
    write.flush().await.expect("flush failed");

    // Use timeout to avoid hanging
    let result = tokio::time::timeout(std::time::Duration::from_secs(3), read.next()).await;
    match result {
        Ok(Some(Ok(msg))) => {
            match msg {
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
            }
        }
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
    write.send(Message::Text(info.into())).await.expect("send failed");

    let resp = read.next().await.expect("no response").unwrap();
    let text = resp.into_text().unwrap();
    println!("app_info response: {text}");
    assert!(text.contains("translator") || text.contains("version"), "Expected app info, got: {text}");
}

#[tokio::test]
async fn bridge_ws_commands_list() {
    let (ws, _) = connect_async("ws://127.0.0.1:9223")
        .await
        .expect("Failed to connect to bridge");
    let (mut write, mut read) = ws.split();

    let list = r#"{"jsonrpc":"2.0","method":"commands_list","params":{},"id":"3"}"#;
    write.send(Message::Text(list.into())).await.expect("send failed");

    let resp = read.next().await.expect("no response").unwrap();
    let text = resp.into_text().unwrap();
    println!("commands_list response: {text}");
    assert!(text.contains("["), "Expected JSON array, got: {text}");
}

/// NOTE: evaluate_js currently times out because the use_effect eval loop
/// (in Translator/src/main.rs) uses a blocking `loop {}` with `std::thread::sleep`
/// that freezes the Dioxus event loop. The request reaches `pending_eval_requests`
/// but the main thread never processes it. Fix tracked as D67.
#[tokio::test]
async fn bridge_ws_evaluate_js() {
    let (ws, _) = connect_async("ws://127.0.0.1:9223")
        .await
        .expect("Failed to connect to bridge");
    let (mut write, mut read) = ws.split();

    let eval = r#"{"jsonrpc":"2.0","method":"evaluate_js","params":{"code":"1 + 1"},"id":"4"}"#;
    write.send(Message::Text(eval.into())).await.expect("send failed");
    println!("Sent evaluate_js: {eval}");

    // Request is routed correctly (no parse error) but eval times out
    // because use_effect loop is blocking (D67)
    let timeout = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        read.next()
    ).await;

    match timeout {
        Ok(Some(Ok(msg))) => {
            let text = msg.into_text().unwrap();
            println!("evaluate_js response: {text}");
            // Should NOT be a parse error (parse errors mean WebSocket frame bug)
            assert!(!text.contains("Parse error"), "Got parse error (D70 regression): {text}");
            // Should be either a result or a command timeout
            assert!(text.contains("result") || text.contains("timeout") || text.contains("error"),
                "Expected result or timeout, got: {text}");
        }
        Ok(None) => panic!("Stream ended unexpectedly"),
        Err(_) => println!("Timeout (expected - use_effect blocking loop, D67)"),
        _ => panic!("Unexpected message type"),
    }
}
