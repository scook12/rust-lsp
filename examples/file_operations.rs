//! File operations and text document synchronization example
//!
//! This example demonstrates the concepts of text document synchronization
//! and shows how you would handle different file operations with the client.

use rust_lsp::{types::*, Client};
use std::io::Cursor;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 File Operations Example");
    println!("==========================");

    // Create mock server responses for various file operations
    let mock_responses = r#"Content-Length: 150

{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"textDocumentSync":{"openClose":true,"change":2,"save":{"includeText":true}},"hoverProvider":true}}}Content-Length: 85

{"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":"file:///example.rs","diagnostics":[]}}Content-Length: 120

{"jsonrpc":"2.0","id":2,"result":{"contents":{"kind":"markdown","value":"**Function**: `main`\n\nEntry point of the program"},"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":4}}}}"#;

    let reader = Cursor::new(mock_responses.as_bytes());
    let writer = Cursor::new(Vec::new());
    let mut client = Client::new(reader, writer);

    println!("✅ Client created for file operations");

    // Step 1: Simulate opening a document
    println!("\n📂 Step 1: Opening a text document");

    let document_uri = "file:///example.rs";
    let document_text = r#"fn main() {
    println!("Hello, world!");
    let x = 42;
    println!("The answer is {}", x);
}"#;

    // In a real scenario, you'd create and send a textDocument/didOpen notification
    println!("📤 Would send textDocument/didOpen notification");
    println!("   URI: {}", document_uri);
    println!("   Language: rust");
    println!("   Version: 1");
    println!("   Text length: {} characters", document_text.len());

    // Simulate receiving diagnostics response
    match timeout(Duration::from_millis(50), client.receive_message()).await {
        Ok(Some(RpcMessage::Response(_response))) => {
            println!("✅ Received initialize response (capabilities)");
        }
        _ => {}
    }

    // Step 2: Simulate receiving diagnostics
    println!("\n🔍 Step 2: Receiving diagnostics");

    match timeout(Duration::from_millis(50), client.receive_message()).await {
        Ok(Some(RpcMessage::Notification(notif))) => {
            if notif.method == "textDocument/publishDiagnostics" {
                println!("📊 Received diagnostics notification");
                if let Some(_params) = notif.params {
                    println!("   📄 Would parse diagnostics here");
                    println!("   ✅ No issues found in this demo!");
                }
            }
        }
        _ => println!("⚠️  No diagnostics received"),
    }

    // Step 3: Simulate document changes
    println!("\n✏️  Step 3: Making document changes");

    let new_text = r#"fn main() {
    println!("Hello, LSP world!");
    let x = 42;
    let y = x * 2;
    println!("The answer is {} and double is {}", x, y);
}"#;

    println!("📝 Would send textDocument/didChange notification");
    println!("   Version: 2");
    println!("   Change type: Full document replacement");
    println!("   New text length: {} characters", new_text.len());

    // Step 4: Simulate hover request
    println!("\n🔍 Step 4: Requesting hover information");

    let hover_position = Position {
        line: 0,
        character: 3, // Position at "main"
    };

    println!("📤 Would send textDocument/hover request");
    println!(
        "   Position: line {}, character {} (at 'main')",
        hover_position.line, hover_position.character
    );

    // Simulate receiving hover response
    match timeout(Duration::from_millis(50), client.receive_message()).await {
        Ok(Some(RpcMessage::Response(response))) => {
            if let Some(_result) = response.result {
                println!("✅ Received hover response");
                println!("   📋 Would contain hover information here");
            }
        }
        _ => println!("⚠️  No hover response received"),
    }

    // Step 5: Simulate saving the document
    println!("\n💾 Step 5: Saving the document");

    println!("📤 Would send textDocument/didSave notification");
    println!("   Including text content in save notification");

    // Step 6: Simulate closing the document
    println!("\n📁 Step 6: Closing the document");

    println!("📤 Would send textDocument/didClose notification");
    println!("   Document will be removed from server memory");

    println!("\n🏁 File operations example completed!");
    println!("\n📚 What we demonstrated:");
    println!("   ✅ Document lifecycle management");
    println!("   ✅ Text synchronization");
    println!("   ✅ Change notifications");
    println!("   ✅ Language features (hover)");
    println!("   ✅ Save/close operations");

    println!("\n💡 In a real application:");
    println!("   - Connect to an actual language server");
    println!("   - Handle responses asynchronously");
    println!("   - Process diagnostics for error highlighting");
    println!("   - Use hover data for tooltips and documentation");

    Ok(())
}
