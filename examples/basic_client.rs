//! Basic LSP client initialization example
//!
//! This example shows how to create and initialize an LSP client
//! with the most basic configuration.

use rust_lsp::{types::*, Client};
use std::io::Cursor;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Basic LSP Client Example");
    println!("==========================");

    // Create a mock server response for the initialize request
    let mock_server_response = r#"Content-Length: 187

{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"textDocumentSync":1,"hoverProvider":true,"completionProvider":{"triggerCharacters":["."]}},"serverInfo":{"name":"Mock Server","version":"1.0.0"}}}"#;

    // Create transport using in-memory buffers (for demo purposes)
    let reader = Cursor::new(mock_server_response.as_bytes());
    let writer = Cursor::new(Vec::new());

    // Create the LSP client
    let mut client = Client::new(reader, writer);
    println!("✅ Client created successfully");

    // Create initialization parameters (use defaults to match crate types)
    let init_params = InitializeParams {
        process_id: Some(std::process::id()),
        client_info: Some(ClientInfo {
            name: "Basic Example Client".to_string(),
            version: Some("0.1.0".to_string()),
        }),
        root_uri: Some("file:///example/project".to_string()),
        capabilities: ClientCapabilities::default(),
        root_path: None,
        locale: None,
        initialization_options: None,
        trace: None,
        workspace_folders: None,
    };

    println!("📤 Sending initialize request...");

    // Simulate receiving the initialize response
    // In a real scenario, you would call client.initialize(init_params)
    // but since we're using mock data, we'll just read the response
    match timeout(Duration::from_millis(100), client.receive_message()).await {
        Ok(Some(message)) => {
            match message {
                RpcMessage::Response(response) => {
                    if let Some(result) = response.result {
                        match serde_json::from_value::<InitializeResult>(result) {
                            Ok(init_result) => {
                                println!("✅ Initialize response received!");
                                println!("🔧 Server capabilities:");

                                if let Some(sync) = init_result.capabilities.text_document_sync {
                                    println!("  📄 Text Document Sync: {:?}", sync);
                                }

                                // Print a simple summary of a couple of known capabilities if present
                                if init_result.capabilities.hover_provider.is_some() {
                                    println!("  🔍 Hover: Supported");
                                }
                                if init_result.capabilities.completion_provider.is_some() {
                                    println!("  💡 Completion: Supported");
                                }

                                if let Some(server_info) = init_result.server_info {
                                    println!(
                                        "  🖥️  Server: {} v{}",
                                        server_info.name,
                                        server_info
                                            .version
                                            .unwrap_or_else(|| "unknown".to_string())
                                    );
                                }
                            }
                            Err(e) => {
                                println!("❌ Failed to parse initialize result: {}", e);
                            }
                        }
                    }
                }
                _ => {
                    println!("⚠️  Received unexpected message type");
                }
            }
        }
        Ok(None) => {
            println!("📭 No message received");
        }
        Err(_) => {
            println!("⏰ Timeout waiting for response");
        }
    }

    println!("\n🏁 Basic client example completed!");
    println!("\n💡 Next steps:");
    println!("   - Try the file_operations example for document sync");
    println!("   - Try the diagnostics example for error handling");
    println!("   - Connect to a real language server by replacing the mock transport");

    Ok(())
}
