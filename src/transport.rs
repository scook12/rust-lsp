//! Transport layer for the Language Server Protocol.
//!
//! This module implements the base protocol as defined by the LSP specification,
//! including header parsing, content handling, and message framing.

use crate::error::{LspError, Result};
use crate::types::RpcMessage;
use std::collections::HashMap;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// The default content type for LSP messages.
pub const DEFAULT_CONTENT_TYPE: &str = "application/vscode-jsonrpc; charset=utf-8";

/// Header fields for LSP messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageHeaders {
    /// The length of the content part in bytes.
    pub content_length: usize,
    /// The MIME type of the content part.
    pub content_type: String,
    /// Additional header fields.
    pub additional: HashMap<String, String>,
}

impl MessageHeaders {
    /// Create new headers with the given content length.
    pub fn new(content_length: usize) -> Self {
        Self {
            content_length,
            content_type: DEFAULT_CONTENT_TYPE.to_string(),
            additional: HashMap::new(),
        }
    }

    /// Create headers with custom content type.
    pub fn with_content_type(content_length: usize, content_type: impl Into<String>) -> Self {
        Self {
            content_length,
            content_type: content_type.into(),
            additional: HashMap::new(),
        }
    }

    /// Add an additional header field.
    pub fn add_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional.insert(name.into(), value.into());
        self
    }

    /// Get the character encoding from the content type.
    /// Returns "utf-8" by default, and also accepts "utf8" for backwards compatibility.
    pub fn get_encoding(&self) -> &str {
        if self.content_type.contains("charset=") {
            if let Some(charset_part) = self.content_type.split("charset=").nth(1) {
                let charset = charset_part.split(';').next().unwrap_or("utf-8").trim();
                // Handle backwards compatibility with "utf8"
                if charset == "utf8" {
                    return "utf-8";
                }
                return charset;
            }
        }
        "utf-8"
    }
}

/// A complete LSP message with headers and content.
#[derive(Debug, Clone)]
pub struct Message {
    pub headers: MessageHeaders,
    pub content: String,
}

impl Message {
    /// Create a new message with the given content.
    pub fn new(content: impl Into<String>) -> Self {
        let content = content.into();
        let content_bytes = content.len();

        Self {
            headers: MessageHeaders::new(content_bytes),
            content,
        }
    }

    /// Create a message from an RPC message by serializing it to JSON.
    pub fn from_rpc_message(rpc_message: &RpcMessage) -> Result<Self> {
        let content = serde_json::to_string(rpc_message)?;
        Ok(Self::new(content))
    }

    /// Parse the content as an RPC message.
    pub fn parse_rpc_message(&self) -> Result<RpcMessage> {
        Ok(serde_json::from_str(&self.content)?)
    }

    /// Serialize this message to bytes for transmission.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        // Add Content-Length header
        result.extend_from_slice(
            format!("Content-Length: {}\r\n", self.headers.content_length).as_bytes(),
        );

        // Add Content-Type header if not default
        if self.headers.content_type != DEFAULT_CONTENT_TYPE {
            result.extend_from_slice(
                format!("Content-Type: {}\r\n", self.headers.content_type).as_bytes(),
            );
        }

        // Add additional headers
        for (name, value) in &self.headers.additional {
            result.extend_from_slice(format!("{}: {}\r\n", name, value).as_bytes());
        }

        // Add separator between headers and content
        result.extend_from_slice(b"\r\n");

        // Add content
        result.extend_from_slice(self.content.as_bytes());

        result
    }
}

/// Transport for reading and writing LSP messages.
pub struct Transport<R, W> {
    reader: R,
    writer: W,
}

impl<R: AsyncRead + Unpin, W: AsyncWrite + Unpin> Transport<R, W> {
    /// Create a new transport with the given reader and writer.
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }

    /// Read a complete message from the transport.
    pub async fn read_message(&mut self) -> Result<Message> {
        let headers = self.read_headers().await?;
        let content = self.read_content(&headers).await?;

        Ok(Message { headers, content })
    }

    /// Write a message to the transport.
    pub async fn write_message(&mut self, message: &Message) -> Result<()> {
        let bytes = message.to_bytes();
        self.writer.write_all(&bytes).await?;
        self.writer.flush().await?;
        Ok(())
    }

    /// Write an RPC message to the transport.
    pub async fn write_rpc_message(&mut self, rpc_message: &RpcMessage) -> Result<()> {
        let message = Message::from_rpc_message(rpc_message)?;
        self.write_message(&message).await
    }

    /// Read message headers from the transport.
    async fn read_headers(&mut self) -> Result<MessageHeaders> {
        let mut headers = HashMap::new();
        let mut content_length = None;
        let mut content_type = DEFAULT_CONTENT_TYPE.to_string();

        loop {
            let line = self.read_line().await?;

            // Empty line indicates end of headers
            if line.is_empty() {
                break;
            }

            // Parse header field
            if let Some((name, value)) = parse_header_field(&line)? {
                match name.to_lowercase().as_str() {
                    "content-length" => {
                        content_length = Some(value.parse::<usize>().map_err(|_| {
                            LspError::Transport(format!("Invalid Content-Length: {}", value))
                        })?);
                    }
                    "content-type" => {
                        content_type = value;
                    }
                    _ => {
                        headers.insert(name, value);
                    }
                }
            }
        }

        let content_length = content_length
            .ok_or_else(|| LspError::Transport("Missing Content-Length header".to_string()))?;

        Ok(MessageHeaders {
            content_length,
            content_type,
            additional: headers,
        })
    }

    /// Read message content based on the headers.
    async fn read_content(&mut self, headers: &MessageHeaders) -> Result<String> {
        let mut buffer = vec![0; headers.content_length];
        self.reader.read_exact(&mut buffer).await?;

        // Validate encoding
        let encoding = headers.get_encoding();
        if encoding != "utf-8" {
            return Err(LspError::Transport(format!(
                "Unsupported encoding: {}",
                encoding
            )));
        }

        // Convert to string
        String::from_utf8(buffer)
            .map_err(|e| LspError::Transport(format!("Invalid UTF-8 content: {}", e)))
    }

    /// Read a single line (ending with \r\n) from the transport.
    async fn read_line(&mut self) -> Result<String> {
        let mut line = Vec::new();
        let mut prev_byte = 0u8;

        loop {
            let mut byte = [0u8; 1];
            self.reader.read_exact(&mut byte).await?;
            let byte = byte[0];

            if byte == b'\n' && prev_byte == b'\r' {
                // Remove the \r\n
                line.pop();
                break;
            }

            line.push(byte);
            prev_byte = byte;
        }

        String::from_utf8(line)
            .map_err(|e| LspError::Transport(format!("Invalid UTF-8 in header: {}", e)))
    }
}

/// Parse a header field line into name and value.
fn parse_header_field(line: &str) -> Result<Option<(String, String)>> {
    if line.is_empty() {
        return Ok(None);
    }

    if let Some((name, value)) = line.split_once(": ") {
        Ok(Some((name.trim().to_string(), value.trim().to_string())))
    } else {
        Err(LspError::Transport(format!(
            "Invalid header field: {}",
            line
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::jsonrpc::RequestMessage;
    use std::io::Cursor;

    #[test]
    fn test_message_headers() {
        let headers = MessageHeaders::new(42).add_header("Custom-Header", "custom-value");

        assert_eq!(headers.content_length, 42);
        assert_eq!(headers.content_type, DEFAULT_CONTENT_TYPE);
        assert_eq!(
            headers.additional.get("Custom-Header"),
            Some(&"custom-value".to_string())
        );
    }

    #[test]
    fn test_encoding_detection() {
        let headers = MessageHeaders::with_content_type(10, "application/json; charset=utf-8");
        assert_eq!(headers.get_encoding(), "utf-8");

        let headers_utf8 = MessageHeaders::with_content_type(10, "application/json; charset=utf8");
        assert_eq!(headers_utf8.get_encoding(), "utf-8"); // backwards compatibility

        let headers_default = MessageHeaders::new(10);
        assert_eq!(headers_default.get_encoding(), "utf-8");
    }

    #[test]
    fn test_message_serialization() {
        let content = r#"{"jsonrpc":"2.0","id":1,"method":"test"}"#;
        let message = Message::new(content);
        let bytes = message.to_bytes();
        let expected = format!("Content-Length: {}\r\n\r\n{}", content.len(), content);
        assert_eq!(String::from_utf8(bytes).unwrap(), expected);
    }

    #[test]
    fn test_message_from_rpc() {
        let request = RequestMessage::new(1, "test/method");
        let rpc_message = RpcMessage::Request(request);
        let message = Message::from_rpc_message(&rpc_message).unwrap();

        // Should be valid JSON
        let parsed = message.parse_rpc_message().unwrap();
        assert_eq!(parsed.method(), Some("test/method"));
    }

    #[tokio::test]
    async fn test_transport_round_trip() {
        let request = RequestMessage::new(1, "test/method");
        let rpc_message = RpcMessage::Request(request);

        // Serialize message
        let message = Message::from_rpc_message(&rpc_message).unwrap();
        let bytes = message.to_bytes();

        // Create transport with cursor
        let reader = Cursor::new(bytes);
        let writer = Cursor::new(Vec::new());
        let mut transport = Transport::new(reader, writer);

        // Read back the message
        let read_message = transport.read_message().await.unwrap();
        assert_eq!(read_message.content, message.content);
        assert_eq!(
            read_message.headers.content_length,
            message.headers.content_length
        );

        // Parse RPC message
        let parsed_rpc = read_message.parse_rpc_message().unwrap();
        assert_eq!(parsed_rpc.method(), Some("test/method"));
    }

    #[tokio::test]
    async fn test_transport_write_read() {
        let writer = Cursor::new(Vec::new());
        let mut transport = Transport::new(Cursor::new(vec![]), writer);

        // Write a message
        let request = RequestMessage::new(42, "initialize");
        let rpc_message = RpcMessage::Request(request);
        transport.write_rpc_message(&rpc_message).await.unwrap();

        // Get the written bytes
        let written_bytes = transport.writer.into_inner();

        // Create new transport to read it back
        let reader = Cursor::new(written_bytes);
        let mut read_transport = Transport::new(reader, Cursor::new(Vec::new()));

        let read_message = read_transport.read_message().await.unwrap();
        let parsed = read_message.parse_rpc_message().unwrap();

        assert_eq!(parsed.method(), Some("initialize"));
    }

    #[test]
    fn test_header_parsing() {
        assert_eq!(
            parse_header_field("Content-Length: 123").unwrap(),
            Some(("Content-Length".to_string(), "123".to_string()))
        );

        assert_eq!(
            parse_header_field("Custom-Header: value with spaces").unwrap(),
            Some(("Custom-Header".to_string(), "value with spaces".to_string()))
        );

        assert_eq!(parse_header_field("").unwrap(), None);

        assert!(parse_header_field("InvalidHeader").is_err());
    }
}
