//! Unit tests for core LSP types
//!
//! This module tests the JSON-RPC message parsing, LSP type serialization/deserialization,
//! and error handling functionality.

mod common;

use common::*;
use tokio_lsp::{error::ResponseError, types::*};
use serde_json::json;

#[test]
fn test_id_serialization() {
    // Test number ID
    let id = Id::Number(42);
    let serialized = serde_json::to_string(&id).unwrap();
    assert_eq!(serialized, "42");

    let deserialized: Id = serde_json::from_str(&serialized).unwrap();
    assert_eq!(id, deserialized);

    // Test string ID
    let id = Id::String("test-id".to_string());
    let serialized = serde_json::to_string(&id).unwrap();
    assert_eq!(serialized, "\"test-id\"");

    let deserialized: Id = serde_json::from_str(&serialized).unwrap();
    assert_eq!(id, deserialized);
}

#[test]
fn test_id_display() {
    assert_eq!(Id::Number(42).to_string(), "42");
    assert_eq!(Id::String("test".to_string()).to_string(), "test");
}

#[test]
fn test_id_from_conversions() {
    let id: Id = 42i64.into();
    assert_eq!(id, Id::Number(42));

    let id: Id = "test".into();
    assert_eq!(id, Id::String("test".to_string()));

    let id: Id = "test".to_string().into();
    assert_eq!(id, Id::String("test".to_string()));
}

#[test]
fn test_position() {
    let pos = Position::new(5, 10);
    assert_eq!(pos.line, 5);
    assert_eq!(pos.character, 10);

    let start = Position::start();
    assert_eq!(start.line, 0);
    assert_eq!(start.character, 0);

    // Test serialization
    let json = serde_json::to_value(pos).unwrap();
    let expected = json!({
        "line": 5,
        "character": 10
    });
    assert_eq!(json, expected);

    // Test deserialization
    let deserialized: Position = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, pos);
}

#[test]
fn test_range() {
    let range = Range::new(Position::new(1, 5), Position::new(2, 10));

    assert_eq!(range.start.line, 1);
    assert_eq!(range.start.character, 5);
    assert_eq!(range.end.line, 2);
    assert_eq!(range.end.character, 10);

    // Test convenience constructor
    let range2 = Range::from_coords(1, 5, 2, 10);
    assert_eq!(range, range2);

    // Test single character range
    let single = Range::single_char(Position::new(3, 7));
    assert_eq!(single.start, Position::new(3, 7));
    assert_eq!(single.end, Position::new(3, 8));

    // Test contains
    assert!(range.contains(Position::new(1, 7)));
    assert!(range.contains(Position::new(2, 5)));
    assert!(!range.contains(Position::new(0, 5)));
    assert!(!range.contains(Position::new(2, 10))); // End is exclusive

    // Test is_empty
    let empty = Range::new(Position::new(5, 5), Position::new(5, 5));
    assert!(empty.is_empty());
    assert!(!range.is_empty());
}

#[test]
fn test_location() {
    let location = Location::new("file:///test.rs", Range::from_coords(1, 0, 1, 10));

    assert_eq!(location.uri, "file:///test.rs");
    assert_eq!(location.range.start.line, 1);

    // Test serialization
    let json = serde_json::to_value(&location).unwrap();
    let expected = json!({
        "uri": "file:///test.rs",
        "range": {
            "start": {"line": 1, "character": 0},
            "end": {"line": 1, "character": 10}
        }
    });
    assert_eq!(json, expected);
}

#[test]
fn test_diagnostic_severity() {
    assert_eq!(DiagnosticSeverity::Error as u8, 1);
    assert_eq!(DiagnosticSeverity::Warning as u8, 2);
    assert_eq!(DiagnosticSeverity::Information as u8, 3);
    assert_eq!(DiagnosticSeverity::Hint as u8, 4);

    // Test serialization
    let severity = DiagnosticSeverity::Error;
    let json = serde_json::to_value(severity).unwrap();
    assert_eq!(json, json!(1));

    let deserialized: DiagnosticSeverity = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, severity);
}

#[test]
fn test_diagnostic_code() {
    // Test number code
    let code = DiagnosticCode::Number(404);
    let json = serde_json::to_value(&code).unwrap();
    assert_eq!(json, json!(404));

    let deserialized: DiagnosticCode = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, code);

    // Test string code
    let code = DiagnosticCode::String("E0001".to_string());
    let json = serde_json::to_value(&code).unwrap();
    assert_eq!(json, json!("E0001"));

    let deserialized: DiagnosticCode = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, code);
}

#[test]
fn test_diagnostic() {
    let diagnostic = Diagnostic {
        range: Range::from_coords(0, 0, 0, 5),
        severity: Some(DiagnosticSeverity::Error),
        code: Some(DiagnosticCode::String("E001".to_string())),
        code_description: None,
        source: Some("rust-analyzer".to_string()),
        message: "variable not found".to_string(),
        tags: Some(vec![DiagnosticTag::Unnecessary]),
        related_information: None,
        data: Some(json!({"custom": "data"})),
    };

    // Test serialization
    let json = serde_json::to_value(&diagnostic).unwrap();
    assert!(json["range"].is_object());
    assert_eq!(json["severity"], 1);
    assert_eq!(json["code"], "E001");
    assert_eq!(json["source"], "rust-analyzer");
    assert_eq!(json["message"], "variable not found");
    assert_eq!(json["tags"], json!([1]));
    assert_eq!(json["data"]["custom"], "data");

    // Test deserialization
    let deserialized: Diagnostic = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized, diagnostic);
}

#[test]
fn test_request_message() {
    // Test without params
    let request = RequestMessage::new(Id::Number(1), "test/method");
    assert_eq!(request.id, Id::Number(1));
    assert_eq!(request.method, "test/method");
    assert!(request.params.is_none());

    let json = serde_json::to_value(&request).unwrap();
    let expected = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "test/method"
    });
    assert_eq!(json, expected);

    // Test with params
    let request = RequestMessage::with_params(
        Id::String("req1".to_string()),
        "test/method",
        json!({"key": "value"}),
    );
    assert_eq!(request.id, Id::String("req1".to_string()));
    assert!(request.params.is_some());

    let json = serde_json::to_value(&request).unwrap();
    let expected = json!({
        "jsonrpc": "2.0",
        "id": "req1",
        "method": "test/method",
        "params": {"key": "value"}
    });
    assert_eq!(json, expected);
}

#[test]
fn test_response_message() {
    // Test success response
    let response = ResponseMessage::success(Id::Number(1), json!({"result": "ok"}));
    assert_eq!(response.id, Some(Id::Number(1)));
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let json = serde_json::to_value(&response).unwrap();
    let expected = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {"result": "ok"}
    });
    assert_eq!(json, expected);

    // Test error response
    let error = ResponseError {
        code: -32601,
        message: "Method not found".to_string(),
        data: None,
    };
    let response = ResponseMessage::error(Some(Id::Number(2)), error);
    assert_eq!(response.id, Some(Id::Number(2)));
    assert!(response.result.is_none());
    assert!(response.error.is_some());

    let json = serde_json::to_value(&response).unwrap();
    let expected = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "error": {
            "code": -32601,
            "message": "Method not found"
        }
    });
    assert_eq!(json, expected);
}

#[test]
fn test_notification_message() {
    // Test without params
    let notification = NotificationMessage::new("test/notify");
    assert_eq!(notification.method, "test/notify");
    assert!(notification.params.is_none());

    let json = serde_json::to_value(&notification).unwrap();
    let expected = json!({
        "jsonrpc": "2.0",
        "method": "test/notify"
    });
    assert_eq!(json, expected);

    // Test with params
    let notification = NotificationMessage::with_params("test/notify", json!({"data": "test"}));
    assert!(notification.params.is_some());

    let json = serde_json::to_value(&notification).unwrap();
    let expected = json!({
        "jsonrpc": "2.0",
        "method": "test/notify",
        "params": {"data": "test"}
    });
    assert_eq!(json, expected);
}

#[test]
fn test_client_capabilities_default() {
    let caps = ClientCapabilities::default();
    assert!(caps.workspace.is_some());
    assert!(caps.text_document.is_some());
    assert!(caps.window.is_some());
    assert!(caps.general.is_some());
    assert!(caps.notebook_document.is_none());
    assert!(caps.experimental.is_none());

    // Test serialization doesn't fail
    let _json = serde_json::to_value(&caps).unwrap();
}

#[test]
fn test_initialize_params() {
    let params = test_init_params();

    assert_eq!(params.process_id, Some(12345));
    assert!(params.client_info.is_some());
    assert_eq!(params.root_uri, Some("file:///test/project".to_string()));

    // Test serialization round-trip
    let json = serde_json::to_value(&params).unwrap();
    let deserialized: InitializeParams = serde_json::from_value(json).unwrap();

    assert_eq!(deserialized.process_id, params.process_id);
    assert_eq!(deserialized.client_info, params.client_info);
    assert_eq!(deserialized.root_uri, params.root_uri);
}

#[test]
fn test_rpc_message_parsing() {
    // Test request parsing
    let request_json = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "test",
        "params": {"key": "value"}
    });

    let message: RpcMessage = serde_json::from_value(request_json).unwrap();
    match message {
        RpcMessage::Request(req) => {
            assert_eq!(req.id, Id::Number(1));
            assert_eq!(req.method, "test");
            assert!(req.params.is_some());
        }
        _ => panic!("Expected request message"),
    }

    // Test response parsing
    let response_json = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": {"success": true}
    });

    let message: RpcMessage = serde_json::from_value(response_json).unwrap();
    match message {
        RpcMessage::Response(resp) => {
            assert_eq!(resp.id, Some(Id::Number(2)));
            assert!(resp.result.is_some());
            assert!(resp.error.is_none());
        }
        _ => panic!("Expected response message"),
    }

    // Test notification parsing
    let notification_json = json!({
        "jsonrpc": "2.0",
        "method": "notify",
        "params": {"data": "test"}
    });

    let message: RpcMessage = serde_json::from_value(notification_json).unwrap();
    match message {
        RpcMessage::Notification(notif) => {
            assert_eq!(notif.method, "notify");
            assert!(notif.params.is_some());
        }
        _ => panic!("Expected notification message"),
    }
}
