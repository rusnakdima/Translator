//! WebSocket test client for MCP Bridge
//! Run with: cargo run --example ws_test

use std::net::TcpStream;
use std::io::{Read, Write};

fn main() {
    println!("Connecting to MCP Bridge at ws://127.0.0.1:9223...");
    
    let mut stream = TcpStream::connect("127.0.0.1:9223").expect("Failed to connect");
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).ok();

    // Test 1: ping
    println!("\n=== Test 1: Ping ===");
    let ping = r#"{"jsonrpc":"2.0","method":"ping","params":{},"id":"1"}"#;
    stream.write_all(ping.as_bytes()).expect("Failed to send");
    let mut response = [0u8; 1024];
    if let Ok(n) = stream.read(&mut response) {
        println!("Response: {}", String::from_utf8_lossy(&response[..n]));
    }

    // Test 2: app_info
    println!("\n=== Test 2: App Info ===");
    stream = TcpStream::connect("127.0.0.1:9223").expect("Failed to connect");
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).ok();
    let app_info = r#"{"jsonrpc":"2.0","method":"app_info","params":{},"id":"2"}"#;
    stream.write_all(app_info.as_bytes()).expect("Failed to send");
    let mut response = [0u8; 2048];
    if let Ok(n) = stream.read(&mut response) {
        println!("Response: {}", String::from_utf8_lossy(&response[..n]));
    }

    // Test 3: dom_snapshot
    println!("\n=== Test 3: DOM Snapshot ===");
    stream = TcpStream::connect("127.0.0.1:9223").expect("Failed to connect");
    stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).ok();
    let dom = r#"{"jsonrpc":"2.0","method":"dom_snapshot","params":{"selector":"body"},"id":"3"}"#;
    stream.write_all(dom.as_bytes()).expect("Failed to send");
    let mut response = [0u8; 8192];
    if let Ok(n) = stream.read(&mut response) {
        let resp_str = String::from_utf8_lossy(&response[..n]);
        if resp_str.len() > 500 {
            println!("Response (truncated): {}...", &resp_str[..500]);
        } else {
            println!("Response: {}", resp_str);
        }
    }

    println!("\n=== Tests Complete ===");
}
