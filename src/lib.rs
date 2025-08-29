//! # tokio-lsp
//!
//! A Language Server Protocol (LSP) client implementation in Rust.
//!
//! This crate provides a lightweight, async-first LSP client that can be integrated
//! into text editors and IDEs. It implements the LSP 3.16 specification and focuses
//! on providing a clean, safe API for communicating with language servers.
//!
//! ## Features
//!
//! - Full LSP 3.16 specification support
//! - Async/await interface using tokio
//! - Type-safe message handling with serde
//! - Comprehensive error handling
//! - Transport layer abstraction
//!
//! ## Example
//!
//! ```rust,no_run
//! use tokio_lsp::Client;
//! use tokio::process::{ChildStdin, ChildStdout};
//! use std::io::Cursor;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Example using mock streams
//!     let reader = Cursor::new(vec![]);
//!     let writer = Cursor::new(Vec::new());
//!     let client = Client::new(reader, writer);
//!     // In real usage, you'd connect to a language server process
//!     // and use its stdin/stdout for reader/writer
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod transport;
pub mod types;

pub use client::Client;
pub use error::{LspError, Result};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::client::Client;
    pub use crate::error::{LspError, Result};
    pub use crate::types::*;
}
