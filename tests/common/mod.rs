//! Common test utilities and infrastructure for rust-lsp tests
//!
//! This module provides shared testing utilities, mock implementations,
//! and test data for integration and unit tests.
#![allow(dead_code)]
use rust_lsp::{types::*, Client};
use std::io::{Cursor, Result as IoResult};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// Mock transport that can be configured with predefined responses
pub struct MockTransport {
    pub read_data: Cursor<Vec<u8>>,
    pub write_data: Cursor<Vec<u8>>,
    pub read_error: Option<std::io::Error>,
    pub write_error: Option<std::io::Error>,
}

impl MockTransport {
    /// Create a new mock transport with predefined read data
    pub fn new(read_data: Vec<u8>) -> Self {
        Self {
            read_data: Cursor::new(read_data),
            write_data: Cursor::new(Vec::new()),
            read_error: None,
            write_error: None,
        }
    }

    /// Create a mock transport that will return an error on read
    pub fn with_read_error(error: std::io::Error) -> Self {
        Self {
            read_data: Cursor::new(Vec::new()),
            write_data: Cursor::new(Vec::new()),
            read_error: Some(error),
            write_error: None,
        }
    }

    /// Create a mock transport that will return an error on write
    pub fn with_write_error(error: std::io::Error) -> Self {
        Self {
            read_data: Cursor::new(Vec::new()),
            write_data: Cursor::new(Vec::new()),
            read_error: None,
            write_error: Some(error),
        }
    }

    /// Get the data that was written to this transport
    pub fn written_data(&self) -> &[u8] {
        self.write_data.get_ref()
    }
}

impl AsyncRead for MockTransport {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        if let Some(ref error) = self.read_error {
            return Poll::Ready(Err(std::io::Error::new(error.kind(), "Mock read error")));
        }
        Pin::new(&mut self.read_data).poll_read(cx, buf)
    }
}

impl AsyncWrite for MockTransport {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<IoResult<usize>> {
        if let Some(ref error) = self.write_error {
            return Poll::Ready(Err(std::io::Error::new(error.kind(), "Mock write error")));
        }
        Pin::new(&mut self.write_data).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.write_data).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.write_data).poll_shutdown(cx)
    }
}

/// Sample LSP messages for testing
pub struct TestMessages;

impl TestMessages {
    /// A valid initialize request
    pub fn initialize_request() -> String {
        r#"Content-Length: 246

{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":12345,"clientInfo":{"name":"Test Client","version":"1.0.0"},"rootUri":"file:///test/project","capabilities":{"textDocument":{"hover":{"dynamicRegistration":false}}}}}"#.to_string()
    }

    /// A valid server request (asking client to do something)
    pub fn server_request() -> String {
        "Content-Length: 91\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"client/registerCapability\",\"params\":{\"registrations\":[]}}".to_string()
    }

    /// A diagnostic notification
    pub fn diagnostic_notification() -> String {
        "Content-Length: 244\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/publishDiagnostics\",\"params\":{\"uri\":\"file:///test.rs\",\"diagnostics\":[{\"range\":{\"start\":{\"line\":0,\"character\":0},\"end\":{\"line\":0,\"character\":5}},\"message\":\"test diagnostic\",\"severity\":1,\"source\":\"test\"}]}}".to_string()
    }

    /// Another notification
    pub fn progress_notification() -> String {
        "Content-Length: 115\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"$/progress\",\"params\":{\"token\":\"workDone\",\"value\":{\"kind\":\"begin\",\"title\":\"Processing\"}}}".to_string()
    }

    /// Multiple messages concatenated
    pub fn multiple_messages() -> String {
        format!(
            "{}{}{}",
            Self::server_request(),
            Self::diagnostic_notification(),
            Self::progress_notification()
        )
    }
}

/// Create a client with mock transport for testing
pub fn create_test_client(read_data: Vec<u8>) -> Client<MockTransport, MockTransport> {
    let reader = MockTransport::new(read_data);
    let writer = MockTransport::new(Vec::new());
    Client::new(reader, writer)
}

/// Create test initialization parameters
pub fn test_init_params() -> InitializeParams {
    InitializeParams {
        process_id: Some(12345),
        client_info: Some(ClientInfo {
            name: "Test Client".to_string(),
            version: Some("1.0.0".to_string()),
        }),
        root_uri: Some("file:///test/project".to_string()),
        capabilities: ClientCapabilities::default(),
        root_path: None,
        locale: None,
        initialization_options: None,
        trace: None,
        workspace_folders: None,
    }
}

/// Assert that two JSON values are equivalent, ignoring field order
pub fn assert_json_eq(expected: &str, actual: &str) {
    let expected: serde_json::Value =
        serde_json::from_str(expected).expect("Expected JSON should be valid");
    let actual: serde_json::Value =
        serde_json::from_str(actual).expect("Actual JSON should be valid");
    assert_eq!(expected, actual, "JSON values should be equal");
}

/// Create a range for testing
pub fn test_range() -> Range {
    Range::new(Position::new(0, 0), Position::new(0, 5))
}

/// Create a diagnostic for testing
pub fn test_diagnostic() -> Diagnostic {
    Diagnostic {
        range: test_range(),
        severity: Some(DiagnosticSeverity::Error),
        code: Some(DiagnosticCode::String("E001".to_string())),
        code_description: None,
        source: Some("test".to_string()),
        message: "Test diagnostic message".to_string(),
        tags: None,
        related_information: None,
        data: None,
    }
}
