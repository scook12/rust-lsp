# rust-lsp

A Language Server Protocol (LSP) client implementation in Rust.

This crate provides a lightweight, async-first LSP client that can be integrated into text editors and IDEs. It implements the LSP 3.16 specification and focuses on providing a clean, safe API for communicating with language servers.

## Features

- **Full LSP 3.16 specification support** - Comprehensive types and message handling
- **Async/await interface** - Built on tokio for non-blocking I/O
- **Type-safe message handling** - Uses serde for JSON serialization/deserialization
- **Comprehensive error handling** - Detailed error types for different failure modes
- **Transport layer abstraction** - Clean separation between protocol and transport
- **Safe Rust** - Uses only safe Rust code with no `unsafe` blocks
- **Well-tested** - Extensive unit tests for all components

## Architecture

The crate is organized into several key modules:

- **`types`** - LSP and JSON-RPC type definitions
- **`transport`** - Low-level message framing and I/O
- **`client`** - High-level client interface
- **`error`** - Comprehensive error handling

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
rust-lsp = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use rust_lsp::Client;
use std::process::Stdio;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start a language server process
    let mut server = Command::new("rust-analyzer")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    
    let stdin = server.stdin.take().unwrap();
    let stdout = server.stdout.take().unwrap();
    
    // Create LSP client
    let client = Client::new(stdout, stdin);
    
    // Initialize the server
    let init_result = client.initialize_default(
        "My Editor", 
        Some("1.0.0".to_string()), 
        Some("file:///path/to/workspace".to_string())
    ).await?;
    
    println!("Server capabilities: {:?}", init_result.capabilities);
    
    // Send requests and notifications
    // client.send_notification("textDocument/didOpen", Some(params)).await?;
    
    Ok(())
}
```

### Manual Initialization

For more control over the initialization process:

```rust
use rust_lsp::{Client, InitializeParams, ClientCapabilities, ClientInfo};

// Create initialize parameters
let params = InitializeParams {
    process_id: Some(std::process::id()),
    client_info: Some(ClientInfo {
        name: "My Editor".to_string(),
        version: Some("1.0.0".to_string()),
    }),
    root_uri: Some("file:///path/to/workspace".to_string()),
    capabilities: ClientCapabilities::default(),
    // ... other fields
    ..Default::default()
};

let result = client.initialize(params).await?;
client.initialized().await?;
```

### Handling Server Messages

```rust
// Listen for messages from the server
while let Some(message) = client.receive_message().await {
    match message {
        RpcMessage::Request(req) => {
            // Handle server requests
            println!("Server request: {}", req.method);
            // Send response back
            client.send_response(req.id, Some(serde_json::json!({})), None).await?;
        }
        RpcMessage::Notification(notif) => {
            // Handle server notifications
            println!("Server notification: {}", notif.method);
        }
        _ => {} // Responses are handled internally
    }
}
```

## Examples

The repository includes several examples:

- **`basic_client.rs`** - Simple client initialization
- **`file_operations.rs`** - Text document synchronization
- **`diagnostics.rs`** - Handling diagnostic messages

Run examples with:

```bash
cargo run --example basic_client
```

## LSP Message Types

The crate provides comprehensive type definitions for all LSP messages:

### Core Types
- `Position`, `Range`, `Location` - Text document positioning
- `TextEdit`, `WorkspaceEdit` - Document modifications  
- `Diagnostic` - Error/warning information
- `Command` - Executable commands

### Request/Response Types
- `InitializeParams` / `InitializeResult`
- `CompletionParams` / `CompletionList`
- `HoverParams` / `Hover`
- And many more...

### Notification Types
- `DidOpenTextDocumentParams`
- `DidChangeTextDocumentParams`
- `PublishDiagnosticsParams`
- And more...

## Transport Layer

The transport layer handles the LSP base protocol:

- **Header parsing** - Content-Length and Content-Type headers
- **Message framing** - Proper JSON-RPC message boundaries
- **Encoding handling** - UTF-8 content with backwards compatibility
- **Error recovery** - Graceful handling of malformed messages

## Error Handling

The crate provides detailed error types:

```rust
use rust_lsp::{LspError, Result};

match client.send_request("textDocument/hover", params).await {
    Ok(response) => { /* handle success */ },
    Err(LspError::Timeout) => { /* request timed out */ },
    Err(LspError::Protocol(err)) => { /* LSP protocol error */ },
    Err(LspError::Transport(msg)) => { /* transport layer error */ },
    // ... other error types
}
```

## Testing

Run the full test suite:

```bash
cargo test
```

Run with coverage:

```bash
cargo test --all-features
```

## Contributing

Contributions are welcome! Please:

1. Follow the existing code style
2. Add tests for new functionality
3. Update documentation as needed
4. Use conventional commit messages

## License

This project is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## LSP Specification

This implementation is based on the [Language Server Protocol Specification v3.16](https://microsoft.github.io/language-server-protocol/specifications/specification-3-16/).
