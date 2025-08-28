//! Error types for the LSP client.
//!
//! This module defines all error types that can occur during LSP communication,
//! from transport-level errors to protocol-level failures.

use std::fmt;
use thiserror::Error;

/// A specialized Result type for LSP operations.
pub type Result<T> = std::result::Result<T, LspError>;

/// The main error type for LSP operations.
#[derive(Error, Debug)]
pub enum LspError {
    /// IO errors from the transport layer
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Protocol-level errors as defined by the LSP specification
    #[error("LSP protocol error: {0}")]
    Protocol(#[from] ResponseError),

    /// Transport protocol errors (malformed headers, etc.)
    #[error("Transport error: {0}")]
    Transport(String),

    /// Connection errors
    #[error("Connection error: {0}")]
    Connection(String),

    /// Request timeout
    #[error("Request timeout")]
    Timeout,

    /// Server initialization failed
    #[error("Server initialization failed: {0}")]
    InitializationFailed(String),

    /// Generic error for other cases
    #[error("LSP error: {0}")]
    Other(String),
}

/// LSP ResponseError as defined by the JSON-RPC specification.
/// This corresponds to the error object in LSP response messages.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseError {
    /// A number indicating the error type that occurred.
    pub code: i32,
    /// A string providing a short description of the error.
    pub message: String,
    /// A primitive or structured value that contains additional information about the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for ResponseError {}

/// Error codes as defined by the LSP specification.
pub mod error_codes {
    // JSON RPC error codes
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    // JSON RPC reserved error range
    pub const JSONRPC_RESERVED_ERROR_RANGE_START: i32 = -32099;
    pub const SERVER_NOT_INITIALIZED: i32 = -32002;
    pub const UNKNOWN_ERROR_CODE: i32 = -32001;
    pub const JSONRPC_RESERVED_ERROR_RANGE_END: i32 = -32000;

    // LSP reserved error range
    pub const LSP_RESERVED_ERROR_RANGE_START: i32 = -32899;
    pub const CONTENT_MODIFIED: i32 = -32801;
    pub const REQUEST_CANCELLED: i32 = -32800;
    pub const LSP_RESERVED_ERROR_RANGE_END: i32 = -32800;
}

impl ResponseError {
    /// Create a new ResponseError with the given code and message.
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create a new ResponseError with additional data.
    pub fn with_data(code: i32, message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Create a parse error.
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(error_codes::PARSE_ERROR, message)
    }

    /// Create an invalid request error.
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(error_codes::INVALID_REQUEST, message)
    }

    /// Create a method not found error.
    pub fn method_not_found(message: impl Into<String>) -> Self {
        Self::new(error_codes::METHOD_NOT_FOUND, message)
    }

    /// Create an invalid params error.
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(error_codes::INVALID_PARAMS, message)
    }

    /// Create an internal error.
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(error_codes::INTERNAL_ERROR, message)
    }

    /// Create a server not initialized error.
    pub fn server_not_initialized(message: impl Into<String>) -> Self {
        Self::new(error_codes::SERVER_NOT_INITIALIZED, message)
    }

    /// Create a request cancelled error.
    pub fn request_cancelled(message: impl Into<String>) -> Self {
        Self::new(error_codes::REQUEST_CANCELLED, message)
    }

    /// Create a content modified error.
    pub fn content_modified(message: impl Into<String>) -> Self {
        Self::new(error_codes::CONTENT_MODIFIED, message)
    }
}
