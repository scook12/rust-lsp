//! Core types for the Language Server Protocol.
//!
//! This module contains all the basic types used in LSP communication,
//! including JSON-RPC message types and LSP-specific data structures.

use serde::{Deserialize, Serialize};

pub mod initialization;
pub mod jsonrpc;
pub mod lsp;

pub use initialization::*;
pub use jsonrpc::*;
pub use lsp::*;

/// Type alias for request/notification IDs.
/// Can be either a number or a string as per JSON-RPC spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Id {
    Number(i64),
    String(String),
}

impl From<i64> for Id {
    fn from(value: i64) -> Self {
        Id::Number(value)
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Id::String(value)
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Id::String(value.to_string())
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Id::Number(n) => write!(f, "{}", n),
            Id::String(s) => write!(f, "{}", s),
        }
    }
}

/// Progress token as defined by LSP.
pub type ProgressToken = Id;

/// URI type as defined by LSP.
/// Over the wire, it's transferred as a string but represents a valid URI.
pub type Uri = String;

/// Document URI type as defined by LSP.
/// Guaranteed to be a valid document URI.
pub type DocumentUri = Uri;
