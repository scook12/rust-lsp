//! Diagnostic message handling example
//!
//! This example demonstrates how to handle diagnostic messages from
//! language servers, including errors, warnings, and informational messages.

use rust_lsp::{types::*, Client};
use std::io::Cursor;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Diagnostics Handling Example");
    println!("===============================");

    // Create mock diagnostic notifications with various severity levels
    let mock_diagnostics = r#"Content-Length: 280

{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":"file:///example.rs","diagnostics":[{"range":{"start":{"line":2,"character":8},"end":{"line":2,"character":9}},"message":"unused variable: `x`","severity":2,"code":"unused_variables","source":"rustc"}]}}Content-Length: 320

{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":"file:///example.rs","diagnostics":[{"range":{"start":{"line":5,"character":4},"end":{"line":5,"character":20}},"message":"cannot find function `undefined_func` in this scope","severity":1,"code":"E0425","source":"rustc","relatedInformation":[{"location":{"uri":"file:///example.rs","range":{"start":{"line":1,"character":0},"end":{"line":1,"character":10}}},"message":"consider importing this function"}]}]}}Content-Length: 250

{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":"file:///helper.rs","diagnostics":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"message":"this function could be marked as `const`","severity":3,"code":"clippy::missing_const_for_fn","source":"clippy"}]}}Content-Length: 180

{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":"file:///example.rs","diagnostics":[{"range":{"start":{"line":10,"character":0},"end":{"line":10,"character":12}},"message":"TODO: implement error handling","severity":4,"source":"todo-comments"}]}}"#;

    let reader = Cursor::new(mock_diagnostics.as_bytes());
    let writer = Cursor::new(Vec::new());
    let mut client = Client::new(reader, writer);

    println!("✅ Client created for diagnostics handling");

    // Process multiple diagnostic notifications
    println!("\n📊 Processing diagnostic notifications...\n");

    for i in 1..=4 {
        match timeout(Duration::from_millis(50), client.receive_message()).await {
            Ok(Some(RpcMessage::Notification(notif))) => {
                if notif.method == "textDocument/publishDiagnostics" {
                    println!("📋 Diagnostic Set {}", i);
                    if let Some(params) = notif.params {
                        println!("   📄 Received diagnostics notification");
                        println!("   🔍 Raw params: {}", params);

                        // In a real implementation, you would parse the diagnostics here
                        // and extract information like:
                        // - File URI
                        // - Error/Warning/Info messages
                        // - Line/character positions
                        // - Diagnostic codes and sources

                        // Simulate some diagnostic analysis
                        if params.to_string().contains("unused variable") {
                            println!("   ⚠️  Found: Unused variable warning");
                        }
                        if params.to_string().contains("cannot find function") {
                            println!("   ❌ Found: Function not found error");
                        }
                        if params.to_string().contains("clippy") {
                            println!("   ℹ️  Found: Clippy suggestion");
                        }
                        if params.to_string().contains("TODO") {
                            println!("   💡 Found: TODO comment");
                        }

                        println!(); // Extra spacing between diagnostic sets
                    }
                } else {
                    println!("⚠️  Received non-diagnostic notification: {}", notif.method);
                }
            }
            Ok(Some(_)) => {
                println!("⚠️  Received non-notification message");
            }
            Ok(None) => {
                println!("📭 No more messages");
                break;
            }
            Err(_) => {
                println!("⏰ Timeout waiting for message {}", i);
            }
        }
    }

    // Demonstrate diagnostic analysis
    println!("🎯 Diagnostic Analysis Summary");
    println!("=============================");

    let sample_diagnostics = vec![
        (
            "file:///example.rs",
            "Error",
            "E0425",
            "Cannot find function",
        ),
        (
            "file:///example.rs",
            "Warning",
            "unused_variables",
            "Unused variable",
        ),
        (
            "file:///helper.rs",
            "Info",
            "clippy::missing_const_for_fn",
            "Could be const",
        ),
        ("file:///example.rs", "Hint", "", "TODO comment"),
    ];

    let mut error_count = 0;
    let mut warning_count = 0;
    let mut info_count = 0;
    let mut hint_count = 0;

    for (file, severity, code, message) in &sample_diagnostics {
        match *severity {
            "Error" => error_count += 1,
            "Warning" => warning_count += 1,
            "Info" => info_count += 1,
            "Hint" => hint_count += 1,
            _ => {}
        }
    }

    println!("📊 Summary:");
    println!("   ❌ Errors: {}", error_count);
    println!("   ⚠️  Warnings: {}", warning_count);
    println!("   ℹ️  Information: {}", info_count);
    println!("   💡 Hints: {}", hint_count);
    println!("   📁 Total files: {}", 2);

    println!("\n🔧 How to handle diagnostics in your application:");
    println!("   1. ✅ Subscribe to publishDiagnostics notifications");
    println!("   2. 🎨 Display errors/warnings with appropriate styling");
    println!("   3. 🔗 Handle related information for better context");
    println!("   4. 🏷️  Process diagnostic tags for special handling");
    println!("   5. 📍 Update UI based on line/character positions");
    println!("   6. 🔄 Clear previous diagnostics when new ones arrive");

    println!("\n💡 Pro tips:");
    println!("   - Group diagnostics by file for efficient UI updates");
    println!("   - Use severity levels to prioritize display");
    println!("   - Cache diagnostic codes for quick reference");
    println!("   - Implement filtering by severity or source");

    println!("\n🏁 Diagnostics example completed!");

    Ok(())
}
