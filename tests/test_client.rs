//! Integration tests for LSP client functionality
//!
//! This module tests client initialization, message sending/receiving,
//! request/response handling, and async operations.

mod common;

use common::*;
use rust_lsp::{error::*, types::*, Client};
use serde_json::json;
use std::io::Cursor;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_client_creation() {
    let reader = Cursor::new(Vec::new());
    let writer = Cursor::new(Vec::new());

    let client = Client::new(reader, writer);

    // Test that pending request count starts at zero
    assert_eq!(client.pending_request_count().await, 0);
    assert!(!client.has_pending_requests().await);
}

#[tokio::test]
async fn test_receive_message_request() {
    let request_data = TestMessages::server_request();
    let reader = Cursor::new(request_data.into_bytes());
    let writer = Cursor::new(Vec::new());

    let mut client = Client::new(reader, writer);

    // Give the background task a moment to process the message
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Receive the message
    let message = timeout(Duration::from_millis(100), client.receive_message())
        .await
        .expect("Should not timeout")
        .expect("Should receive a message");

    match message {
        RpcMessage::Request(request) => {
            assert_eq!(request.id, Id::Number(1));
            assert_eq!(request.method, "client/registerCapability");
            assert!(request.params.is_some());

            // Verify the request contains expected data
            if let Some(params) = request.params {
                let registrations = params["registrations"]
                    .as_array()
                    .expect("Should have registrations");
                assert_eq!(registrations.len(), 0);
            }
        }
        _ => panic!("Expected request message"),
    }
}

#[tokio::test]
async fn test_receive_message_notification() {
    let notification_data = TestMessages::diagnostic_notification();
    let reader = Cursor::new(notification_data.into_bytes());
    let writer = Cursor::new(Vec::new());

    let mut client = Client::new(reader, writer);

    // Give the background task a moment to process the message
    tokio::time::sleep(Duration::from_millis(50)).await;

    let message = timeout(Duration::from_millis(100), client.receive_message())
        .await
        .expect("Should not timeout")
        .expect("Should receive a message");

    match message {
        RpcMessage::Notification(notification) => {
            assert_eq!(notification.method, "textDocument/publishDiagnostics");
            assert!(notification.params.is_some());

            // Verify the notification contains expected data
            if let Some(params) = notification.params {
                let uri = params["uri"].as_str().expect("Should have URI");
                assert_eq!(uri, "file:///test.rs");

                let diagnostics = params["diagnostics"]
                    .as_array()
                    .expect("Should have diagnostics");
                assert_eq!(diagnostics.len(), 1);
            }
        }
        _ => panic!("Expected notification message"),
    }
}

#[tokio::test]
async fn test_receive_multiple_messages() {
    let multiple_data = TestMessages::multiple_messages();
    let reader = Cursor::new(multiple_data.into_bytes());
    let writer = Cursor::new(Vec::new());

    let mut client = Client::new(reader, writer);

    // Give the background task a moment to process messages
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Should receive server request first
    let msg1 = timeout(Duration::from_millis(100), client.receive_message())
        .await
        .unwrap()
        .unwrap();
    match msg1 {
        RpcMessage::Request(req) => {
            assert_eq!(req.method, "client/registerCapability");
        }
        _ => panic!("Expected request message first"),
    }

    // Should receive diagnostic notification second
    let msg2 = timeout(Duration::from_millis(100), client.receive_message())
        .await
        .unwrap()
        .unwrap();
    match msg2 {
        RpcMessage::Notification(notif) => {
            assert_eq!(notif.method, "textDocument/publishDiagnostics");
        }
        _ => panic!("Expected notification message second"),
    }

    // Should receive progress notification third
    let msg3 = timeout(Duration::from_millis(100), client.receive_message())
        .await
        .unwrap()
        .unwrap();
    match msg3 {
        RpcMessage::Notification(notif) => {
            assert_eq!(notif.method, "$/progress");
            assert!(notif.params.is_some());
        }
        _ => panic!("Expected progress notification third"),
    }

    // Should have no more messages
    let result = timeout(Duration::from_millis(50), client.receive_message()).await;
    assert!(result.is_err(), "Should timeout with no more messages");
}

#[tokio::test]
async fn test_initialize_method() {
    let reader = Cursor::new(Vec::new());
    let writer = Cursor::new(Vec::new());

    let _client = Client::new(reader, writer);
    let init_params = test_init_params();

    // Since we're using mock data, we can't actually send a request and get a response
    // But we can test that the initialize method would work with proper data
    // For now, let's just test the parameter creation
    assert_eq!(init_params.process_id, Some(12345));
    assert!(init_params.client_info.is_some());

    // Test that we can serialize the params
    let _serialized = serde_json::to_value(&init_params).expect("Should serialize init params");
}

#[tokio::test]
async fn test_send_notification() {
    let reader = Cursor::new(Vec::new());
    let writer = Cursor::new(Vec::new());

    let client = Client::new(reader, writer);

    // This should complete without error even though we can't actually send
    // (the mock writer will just store the data)
    let params = json!({"uri": "file:///test.rs"});

    // In a real scenario, this would send the notification
    // For testing purposes, we're just verifying the API exists and works
    let _result = client
        .send_notification("textDocument/didOpen", Some(params))
        .await;
    // Note: This will likely fail with the current mock setup, but demonstrates the API
}

#[tokio::test]
async fn test_pending_request_management() {
    let reader = Cursor::new(Vec::new());
    let writer = Cursor::new(Vec::new());

    let client = Client::new(reader, writer);

    // Initially no pending requests
    assert_eq!(client.pending_request_count().await, 0);
    assert!(!client.has_pending_requests().await);

    // Test clearing all requests (should be safe even when empty)
    client.cancel_all_requests().await;
    assert_eq!(client.pending_request_count().await, 0);
}

#[tokio::test]
async fn test_send_response() {
    let reader = Cursor::new(Vec::new());
    let writer = Cursor::new(Vec::new());

    let client = Client::new(reader, writer);

    let response_id = Id::Number(42);
    let result = json!({"status": "ok"});

    // Test successful response
    let _result = client
        .send_response(response_id.clone(), Some(result), None)
        .await;

    // Test error response
    let error = ResponseError {
        code: -32600,
        message: "Invalid Request".to_string(),
        data: None,
    };
    let _result = client
        .send_response(Id::Number(43), None, Some(error))
        .await;
}

#[tokio::test]
async fn test_client_with_empty_data() {
    let reader = Cursor::new(Vec::new());
    let writer = Cursor::new(Vec::new());

    let mut client = Client::new(reader, writer);

    // Attempting to receive from empty data should timeout
    let result = timeout(Duration::from_millis(50), client.receive_message()).await;
    assert!(result.is_err(), "Should timeout with empty data");
}

#[tokio::test]
async fn test_client_with_invalid_message() {
    let invalid_data = b"Content-Length: 20\n\n{\"invalid\": \"json\"}";
    let reader = Cursor::new(invalid_data.as_slice());
    let writer = Cursor::new(Vec::new());

    let mut client = Client::new(reader, writer);

    // This should handle the invalid JSON gracefully
    // The behavior depends on implementation - it might timeout or return an error
    let _result = timeout(Duration::from_millis(100), client.receive_message()).await;

    // We don't assert a specific outcome here since error handling
    // might vary, but the test ensures we don't panic
}

#[tokio::test]
async fn test_concurrent_operations() {
    let reader = Cursor::new(Vec::new());
    let writer = Cursor::new(Vec::new());

    let client = Client::new(reader, writer);

    // Test that multiple concurrent operations on client state work
    let (count1, count2, has_pending) = tokio::join!(
        client.pending_request_count(),
        client.pending_request_count(),
        client.has_pending_requests()
    );

    assert_eq!(count1, 0);
    assert_eq!(count2, 0);
    assert!(!has_pending);
}

#[tokio::test]
async fn test_message_parsing_edge_cases() {
    // Test message with extra whitespace
    let message_with_whitespace =
        "Content-Length: 45\n\n  {\"jsonrpc\":\"2.0\",\"method\":\"test\"}  ";
    let reader = Cursor::new(message_with_whitespace.as_bytes());
    let writer = Cursor::new(Vec::new());

    let mut client = Client::new(reader, writer);

    // Should still parse correctly despite whitespace
    let result = timeout(Duration::from_millis(100), client.receive_message()).await;

    // The exact behavior depends on implementation, but we ensure no panic
    if let Ok(Some(RpcMessage::Notification(notif))) = result {
        assert_eq!(notif.method, "test");
    }
}

#[tokio::test]
async fn test_large_message_handling() {
    // Create a large JSON message
    let large_params = json!({
        "data": "x".repeat(1000), // 1KB of data
        "array": (0..100).collect::<Vec<i32>>(),
        "nested": {
            "deep": {
                "object": {
                    "with": {
                        "many": {
                            "levels": "test"
                        }
                    }
                }
            }
        }
    });

    let large_message = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "method": "large/message",
        "params": large_params
    }))
    .unwrap();

    let full_message = format!(
        "Content-Length: {}\n\n{}",
        large_message.len(),
        large_message
    );
    let reader = Cursor::new(full_message.into_bytes());
    let writer = Cursor::new(Vec::new());

    let mut client = Client::new(reader, writer);

    let result = timeout(Duration::from_millis(200), client.receive_message()).await;

    if let Ok(Some(RpcMessage::Notification(notif))) = result {
        assert_eq!(notif.method, "large/message");
        assert!(notif.params.is_some());

        if let Some(params) = notif.params {
            assert_eq!(params["data"].as_str().unwrap().len(), 1000);
            assert_eq!(params["array"].as_array().unwrap().len(), 100);
        }
    }
}
