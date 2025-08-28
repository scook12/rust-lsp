//! JSON-RPC 2.0 message types as defined by the specification.
//!
//! This module implements the base protocol message types that LSP builds upon.

use crate::error::ResponseError;
use crate::types::Id;
use serde::{Deserialize, Serialize};

/// Base message trait for all JSON-RPC messages.
/// All messages must have a jsonrpc field set to "2.0".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The JSON-RPC version. Always "2.0" for LSP.
    pub jsonrpc: String,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
        }
    }
}

/// Request message to describe a request between client and server.
/// Every processed request must send a response back to the sender.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMessage {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// The request ID
    pub id: Id,
    /// The method to be invoked
    pub method: String,
    /// The method's parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl RequestMessage {
    /// Create a new request message.
    pub fn new(id: impl Into<Id>, method: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            method: method.into(),
            params: None,
        }
    }

    /// Create a new request message with parameters.
    pub fn with_params(
        id: impl Into<Id>,
        method: impl Into<String>,
        params: serde_json::Value,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            method: method.into(),
            params: Some(params),
        }
    }
}

/// Response message sent as a result of a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// The request ID (same as the request, or null for parse errors)
    pub id: Option<Id>,
    /// The result of a successful request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// The error object in case of failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

impl ResponseMessage {
    /// Create a successful response.
    pub fn success(id: impl Into<Id>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id.into()),
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: Option<Id>, error: ResponseError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }

    /// Check if this response represents an error.
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// Get the error if present.
    pub fn get_error(&self) -> Option<&ResponseError> {
        self.error.as_ref()
    }
}

/// Notification message.
/// A processed notification message must not send a response back.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// The method to be invoked
    pub method: String,
    /// The notification's parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl NotificationMessage {
    /// Create a new notification message.
    pub fn new(method: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params: None,
        }
    }

    /// Create a new notification message with parameters.
    pub fn with_params(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params: Some(params),
        }
    }
}

/// Enum representing any type of JSON-RPC message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcMessage {
    // Order matters for untagged deserialization!
    // Most specific variants should come first
    Request(RequestMessage),     // Has both id and method
    Notification(NotificationMessage), // Has method, no id
    Response(ResponseMessage),   // May have only id, or no required fields
}

impl RpcMessage {
    /// Check if this is a request message.
    pub fn is_request(&self) -> bool {
        matches!(self, RpcMessage::Request(_))
    }

    /// Check if this is a response message.
    pub fn is_response(&self) -> bool {
        matches!(self, RpcMessage::Response(_))
    }

    /// Check if this is a notification message.
    pub fn is_notification(&self) -> bool {
        matches!(self, RpcMessage::Notification(_))
    }

    /// Get the method name if this is a request or notification.
    pub fn method(&self) -> Option<&str> {
        match self {
            RpcMessage::Request(req) => Some(&req.method),
            RpcMessage::Notification(notif) => Some(&notif.method),
            RpcMessage::Response(_) => None,
        }
    }

    /// Get the ID if this is a request or response.
    pub fn id(&self) -> Option<&Id> {
        match self {
            RpcMessage::Request(req) => Some(&req.id),
            RpcMessage::Response(resp) => resp.id.as_ref(),
            RpcMessage::Notification(_) => None,
        }
    }
}

/// Parameters for the $/cancelRequest notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelParams {
    /// The request ID to cancel.
    pub id: Id,
}

/// Parameters for the $/progress notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressParams<T> {
    /// The progress token provided by the client or server.
    pub token: crate::types::ProgressToken,
    /// The progress data.
    pub value: T,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_message_serialization() {
        let request = RequestMessage::with_params(1, "test/method", json!({"key": "value"}));
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: RequestMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, Id::Number(1));
        assert_eq!(request.method, "test/method");
        assert_eq!(deserialized.jsonrpc, request.jsonrpc);
    }

    #[test]
    fn test_response_message_success() {
        let response = ResponseMessage::success(1, json!({"result": "success"}));
        assert!(!response.is_error());
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_response_message_error() {
        let error = ResponseError::new(-32600, "Invalid Request");
        let response = ResponseMessage::error(Some(Id::Number(1)), error);
        assert!(response.is_error());
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_notification_message() {
        let notification = NotificationMessage::with_params("test/notify", json!({"data": "test"}));
        assert_eq!(notification.jsonrpc, "2.0");
        assert_eq!(notification.method, "test/notify");
        assert!(notification.params.is_some());
    }

    #[test]
    fn test_rpc_message_enum() {
        let request = RpcMessage::Request(RequestMessage::new(1, "test"));
        assert!(request.is_request());
        assert!(!request.is_response());
        assert!(!request.is_notification());
        assert_eq!(request.method(), Some("test"));
        assert_eq!(request.id(), Some(&Id::Number(1)));
    }
}
