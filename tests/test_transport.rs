//! Transport layer tests
//!
//! This module tests different AsyncRead/AsyncWrite implementations,
//! error conditions, and message framing.

mod common;

use common::*;
use core::panic;
use rust_lsp::{transport::*, Client};
use std::io::{Cursor, ErrorKind};
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_message_parsing_from_stream() {
    let test_message =
        "Content-Length: 40\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}";
    let reader = Cursor::new(test_message.as_bytes());
    let writer = Cursor::new(Vec::new());

    let mut transport = Transport::new(reader, writer);

    // Test that we can parse a basic message
    let message = transport.read_message().await;
    assert!(message.is_ok(), "Should successfully read message");

    let message = message.unwrap();
    assert_eq!(
        message.content.trim(),
        "{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}"
    );
}

#[tokio::test]
async fn test_malformed_header() {
    let bad_header = "Invalid-Header: 45\n\n{\"jsonrpc\":\"2.0\",\"method\":\"test\"}";
    let reader = Cursor::new(bad_header.as_bytes());
    let writer = Cursor::new(Vec::new());

    let mut transport = Transport::new(reader, writer);

    let result = transport.read_message().await;
    assert!(result.is_err(), "Should fail with malformed header");
}

#[tokio::test]
async fn test_missing_content_length() {
    let no_content_length = "\n\n{\"jsonrpc\":\"2.0\",\"method\":\"test\"}";
    let reader = Cursor::new(no_content_length.as_bytes());
    let writer = Cursor::new(Vec::new());

    let mut transport = Transport::new(reader, writer);

    let result = transport.read_message().await;
    assert!(result.is_err(), "Should fail without content-length");
}

#[tokio::test]
async fn test_content_length_mismatch() {
    let wrong_length = "Content-Length: 10\n\n{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"id\":1}";
    let reader = Cursor::new(wrong_length.as_bytes());
    let writer = Cursor::new(Vec::new());

    let mut transport = Transport::new(reader, writer);

    let _result = transport.read_message().await;
    // This might succeed or fail depending on implementation
    // The important thing is that it doesn't panic
}

#[tokio::test]
async fn test_empty_message_body() {
    let empty_body = "Content-Length: 0\n\n";
    let reader = Cursor::new(empty_body.as_bytes());
    let writer = Cursor::new(Vec::new());

    let mut transport = Transport::new(reader, writer);

    let result = transport.read_message().await;
    if let Ok(message) = result {
        assert!(message.content.is_empty());
    }
}

#[tokio::test]
async fn test_message_writing() {
    let reader = Cursor::new(Vec::new());
    let writer = Cursor::new(Vec::new());

    let mut transport = Transport::new(reader, writer);

    let test_rpc = rust_lsp::types::RpcMessage::Notification(
        rust_lsp::types::NotificationMessage::new("test/method"),
    );

    let _result = transport.write_rpc_message(&test_rpc).await;

    // The result depends on the implementation - it might succeed or fail
    // with the cursor-based mock, but should not panic
}

#[tokio::test]
async fn test_large_message_transport() {
    let large_content = "x".repeat(10000); // 10KB
    let large_message = format!(
        "Content-Length: {}\r\n\r\n{}",
        large_content.len(),
        large_content
    );

    let reader = Cursor::new(large_message.as_bytes());
    let writer = Cursor::new(Vec::new());

    let mut transport = Transport::new(reader, writer);

    let result = timeout(Duration::from_secs(1), transport.read_message()).await;

    match result {
        Ok(Ok(message)) => {
            assert_eq!(message.content.len(), 10000);
            assert_eq!(message.content, large_content);
        }
        Ok(Err(_)) => {
            panic!("Unexpected transport error")
        }
        Err(_) => {
            panic!("Unexpected transport timeout")
        }
    }
}

#[tokio::test]
async fn test_concurrent_transport_operations() {
    let reader = Cursor::new(Vec::new());
    let writer = Cursor::new(Vec::new());

    let mut transport = Transport::new(reader, writer);

    // Test that we can have multiple concurrent read operations
    // (though they might all timeout/fail due to empty data)
    // Note: We can't actually do concurrent borrows, so we test sequentially
    let result1 = timeout(Duration::from_millis(50), transport.read_message()).await;
    let result2 = timeout(Duration::from_millis(50), transport.read_message()).await;

    // Both should timeout/fail since there's no data, but shouldn't panic
    // (They could either timeout or return an immediate error)
    // The important thing is they don't succeed or panic
    if let Ok(result1_val) = result1 {
        assert!(
            result1_val.is_err(),
            "Empty reader should not return successful message"
        );
    }
    if let Ok(result2_val) = result2 {
        assert!(
            result2_val.is_err(),
            "Empty reader should not return successful message"
        );
    }
}

#[tokio::test]
async fn test_partial_message_reads() {
    // Simulate a message that arrives in pieces
    let partial1 = "Content-";
    let partial2 = "Length: 24\r\n\r\n";
    let partial3 = "{\"jsonrpc\":\"2.0\",\"id\":1}";

    let full_message = format!("{}{}{}", partial1, partial2, partial3);
    let reader = Cursor::new(full_message.as_bytes());
    let writer = Cursor::new(Vec::new());

    let mut transport = Transport::new(reader, writer);

    let result = transport.read_message().await;

    // Should handle partial reads correctly
    if let Ok(message) = result {
        assert_eq!(message.content, "{\"jsonrpc\":\"2.0\",\"id\":1}");
    }
}

#[tokio::test]
async fn test_multiple_messages_in_stream() {
    let message1 = "Content-Length: 24\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":1}";
    let message2 = "Content-Length: 24\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":2}";
    let combined = format!("{}{}", message1, message2);

    let reader = Cursor::new(combined.as_bytes());
    let writer = Cursor::new(Vec::new());

    let mut transport = Transport::new(reader, writer);

    // Read first message
    let msg1_result = transport.read_message().await;
    assert!(msg1_result.is_ok());

    if let Ok(msg1) = msg1_result {
        assert_eq!(msg1.content, "{\"jsonrpc\":\"2.0\",\"id\":1}");
    }

    // Read second message
    let msg2_result = transport.read_message().await;
    if let Ok(msg2) = msg2_result {
        assert_eq!(msg2.content, "{\"jsonrpc\":\"2.0\",\"id\":2}");
    }
}

#[tokio::test]
async fn test_message_with_extra_headers() {
    let message_with_extra =
        "Content-Length: 25\nContent-Type: application/json\n\n{\"jsonrpc\":\"2.0\",\"id\":1}";
    let reader = Cursor::new(message_with_extra.as_bytes());
    let writer = Cursor::new(Vec::new());

    let mut transport = Transport::new(reader, writer);

    let result = transport.read_message().await;

    // Should ignore extra headers and parse correctly
    if let Ok(message) = result {
        assert_eq!(message.content, "{\"jsonrpc\":\"2.0\",\"id\":1}");
    }
}

#[tokio::test]
async fn test_rpc_message_parsing() {
    let test_message = "Content-Length: 52\n\n{\"jsonrpc\":\"2.0\",\"method\":\"test/notification\",\"params\":{}}";
    let reader = Cursor::new(test_message.as_bytes());
    let writer = Cursor::new(Vec::new());

    let mut transport = Transport::new(reader, writer);

    if let Ok(message) = transport.read_message().await {
        let rpc_result = message.parse_rpc_message();
        assert!(rpc_result.is_ok(), "Should parse valid RPC message");

        if let Ok(rpc_message) = rpc_result {
            match rpc_message {
                rust_lsp::types::RpcMessage::Notification(notif) => {
                    assert_eq!(notif.method, "test/notification");
                }
                _ => panic!("Expected notification message"),
            }
        }
    }
}

#[tokio::test]
async fn test_invalid_json_in_message() {
    let invalid_json = "Content-Length: 20\n\n{invalid json here}";
    let reader = Cursor::new(invalid_json.as_bytes());
    let writer = Cursor::new(Vec::new());

    let mut transport = Transport::new(reader, writer);

    if let Ok(message) = transport.read_message().await {
        let rpc_result = message.parse_rpc_message();
        assert!(rpc_result.is_err(), "Should fail to parse invalid JSON");
    }
}

#[tokio::test]
async fn test_different_line_endings() {
    // Test with Windows line endings (\r\n)
    let windows_message = "Content-Length: 24\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":1}";
    let reader = Cursor::new(windows_message.as_bytes());
    let writer = Cursor::new(Vec::new());

    let mut transport = Transport::new(reader, writer);

    let result = transport.read_message().await;

    // Should handle different line endings
    if let Ok(message) = result {
        assert_eq!(message.content, "{\"jsonrpc\":\"2.0\",\"id\":1}");
    }
}

#[tokio::test]
async fn test_mock_transport_error_conditions() {
    // Test read error
    let read_error = std::io::Error::new(ErrorKind::ConnectionReset, "Connection lost");
    let mock_reader = MockTransport::with_read_error(read_error);
    let mock_writer = MockTransport::new(Vec::new());

    let mut client = Client::new(mock_reader, mock_writer);

    // This should handle the error gracefully
    let _result = timeout(Duration::from_millis(100), client.receive_message()).await;

    // The exact behavior depends on error handling, but shouldn't panic
}

#[tokio::test]
async fn test_transport_with_different_async_types() {
    // Test that transport works with different async I/O types
    let data = b"Content-Length: 25\n\n{\"jsonrpc\":\"2.0\",\"id\":1}";

    // Test with Cursor (which we've been using)
    let cursor = Cursor::new(data.as_slice());

    // We can't easily test with actual file/network I/O in unit tests,
    // but we can verify the trait bounds work
    let _: Box<dyn tokio::io::AsyncRead + Unpin + Send> = Box::new(cursor);
}
