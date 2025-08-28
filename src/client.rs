//! The main LSP client implementation.
//!
//! This module provides the core `Client` struct that handles communication
//! with language servers according to the LSP specification.

use crate::error::{LspError, Result};
use crate::transport::Transport;
use crate::types::{
    ClientCapabilities, ClientInfo, Id, InitializeParams, InitializeResult, NotificationMessage,
    RequestMessage, ResponseMessage, RpcMessage,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{mpsc, oneshot, RwLock};

/// Pending request information.
struct PendingRequest {
    sender: oneshot::Sender<ResponseMessage>,
}

/// The main LSP client for communicating with language servers.
pub struct Client<R, W> {
    /// The underlying transport for reading and writing messages.
    transport: Arc<RwLock<Transport<R, W>>>,
    /// Counter for generating unique request IDs.
    request_id_counter: AtomicI64,
    /// Pending requests waiting for responses.
    pending_requests: Arc<RwLock<HashMap<Id, PendingRequest>>>,
    /// Channel for receiving incoming messages.
    message_receiver: Option<mpsc::UnboundedReceiver<RpcMessage>>,
    /// Channel for sending outgoing messages.
    #[allow(dead_code)]
    message_sender: mpsc::UnboundedSender<RpcMessage>,
    /// Handle for the message processing task.
    _message_task: tokio::task::JoinHandle<()>,
}

impl<R, W> Client<R, W>
where
    R: AsyncRead + Unpin + Send + Sync + 'static,
    W: AsyncWrite + Unpin + Send + Sync + 'static,
{
    /// Create a new LSP client with the given transport.
    pub fn new(reader: R, writer: W) -> Self {
        let transport = Arc::new(RwLock::new(Transport::new(reader, writer)));
        let pending_requests: Arc<RwLock<HashMap<Id, PendingRequest>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let (message_sender, message_receiver) = mpsc::unbounded_channel::<RpcMessage>();
        let message_sender_clone = message_sender.clone();

        // Spawn task to handle incoming messages
        let transport_clone = Arc::clone(&transport);
        let pending_requests_clone = Arc::clone(&pending_requests);
        let message_task = tokio::spawn(async move {
            loop {
                let message = {
                    let mut transport = transport_clone.write().await;
                    match transport.read_message().await {
                        Ok(msg) => msg,
                        Err(e) => {
                            log::error!("Failed to read message: {}", e);
                            break;
                        }
                    }
                };

                let rpc_message = match message.parse_rpc_message() {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::error!("Failed to parse RPC message: {}", e);
                        continue;
                    }
                };

                match &rpc_message {
                    RpcMessage::Response(response) => {
                        if let Some(id) = &response.id {
                            let mut pending = pending_requests_clone.write().await;
                            if let Some(pending_request) = pending.remove(id) {
                                if let Err(e) = pending_request.sender.send(response.clone()) {
                                    log::warn!(
                                        "Failed to send response to pending request: {:?}",
                                        e
                                    );
                                }
                            } else {
                                log::warn!("Received response for unknown request ID: {}", id);
                            }
                        }
                    }
                    RpcMessage::Request(_) | RpcMessage::Notification(_) => {
                        // Forward to client for handling
                        if message_sender_clone.send(rpc_message).is_err() {
                            log::error!("Message receiver dropped, stopping message loop");
                            break;
                        }
                    }
                }
            }
        });

        Self {
            transport,
            request_id_counter: AtomicI64::new(1),
            pending_requests,
            message_receiver: Some(message_receiver),
            message_sender,
            _message_task: message_task,
        }
    }

    /// Generate a new unique request ID.
    fn next_request_id(&self) -> Id {
        Id::Number(self.request_id_counter.fetch_add(1, Ordering::SeqCst))
    }

    /// Send a request and wait for the response.
    pub async fn send_request(
        &self,
        method: impl Into<String>,
        params: Option<serde_json::Value>,
    ) -> Result<ResponseMessage> {
        let id = self.next_request_id();
        let request = match params {
            Some(params) => RequestMessage::with_params(id.clone(), method, params),
            None => RequestMessage::new(id.clone(), method),
        };

        let (response_sender, response_receiver) = oneshot::channel();

        // Register the pending request
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(
                id.clone(),
                PendingRequest {
                    sender: response_sender,
                },
            );
        }

        // Send the request
        {
            let mut transport = self.transport.write().await;
            transport
                .write_rpc_message(&RpcMessage::Request(request))
                .await?;
        }

        // Wait for the response
        match response_receiver.await {
            Ok(response) => Ok(response),
            Err(_) => {
                // Clean up the pending request if it wasn't already removed
                let mut pending = self.pending_requests.write().await;
                pending.remove(&id);
                Err(LspError::Other("Response receiver dropped".to_string()))
            }
        }
    }

    /// Send a notification (no response expected).
    pub async fn send_notification(
        &self,
        method: impl Into<String>,
        params: Option<serde_json::Value>,
    ) -> Result<()> {
        let notification = match params {
            Some(params) => NotificationMessage::with_params(method, params),
            None => NotificationMessage::new(method),
        };

        let mut transport = self.transport.write().await;
        transport
            .write_rpc_message(&RpcMessage::Notification(notification))
            .await
    }

    /// Receive the next incoming message (request or notification from server).
    /// This method should be called in a loop to handle all incoming messages.
    pub async fn receive_message(&mut self) -> Option<RpcMessage> {
        if let Some(ref mut receiver) = self.message_receiver {
            receiver.recv().await
        } else {
            None
        }
    }

    /// Send a response to a request from the server.
    pub async fn send_response(
        &self,
        id: Id,
        result: Option<serde_json::Value>,
        error: Option<crate::error::ResponseError>,
    ) -> Result<()> {
        let response = if let Some(error) = error {
            ResponseMessage::error(Some(id), error)
        } else {
            ResponseMessage::success(id, result.unwrap_or(serde_json::Value::Null))
        };

        let mut transport = self.transport.write().await;
        transport
            .write_rpc_message(&RpcMessage::Response(response))
            .await
    }

    /// Check if there are any pending requests.
    pub async fn has_pending_requests(&self) -> bool {
        !self.pending_requests.read().await.is_empty()
    }

    /// Get the number of pending requests.
    pub async fn pending_request_count(&self) -> usize {
        self.pending_requests.read().await.len()
    }

    /// Cancel all pending requests.
    pub async fn cancel_all_requests(&self) {
        let mut pending = self.pending_requests.write().await;
        pending.clear();
    }

    /// Initialize the LSP server with the given parameters.
    /// This is typically the first method called after creating the client.
    pub async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let response = self
            .send_request("initialize", Some(serde_json::to_value(params)?))
            .await?;

        if let Some(error) = response.error {
            return Err(LspError::InitializationFailed(format!(
                "Initialize request failed: {}",
                error.message
            )));
        }

        if let Some(result) = response.result {
            Ok(serde_json::from_value(result)?)
        } else {
            Err(LspError::InitializationFailed(
                "Initialize response missing result".to_string(),
            ))
        }
    }

    /// Send the 'initialized' notification to the server.
    /// This should be called after a successful 'initialize' request.
    pub async fn initialized(&self) -> Result<()> {
        self.send_notification("initialized", Some(serde_json::json!({})))
            .await
    }

    /// Complete the initialization handshake with default parameters.
    /// This is a convenience method that creates default initialization parameters
    /// and sends both the initialize request and initialized notification.
    pub async fn initialize_default(
        &self,
        client_name: impl Into<String>,
        client_version: Option<String>,
        root_uri: Option<String>,
    ) -> Result<InitializeResult> {
        let params = InitializeParams {
            process_id: Some(std::process::id()),
            client_info: Some(ClientInfo {
                name: client_name.into(),
                version: client_version,
            }),
            locale: None,
            root_path: None,
            root_uri,
            initialization_options: None,
            capabilities: ClientCapabilities::default(),
            trace: None,
            workspace_folders: None,
        };

        let result = self.initialize(params).await?;
        self.initialized().await?;
        Ok(result)
    }
}

impl<R, W> Drop for Client<R, W> {
    fn drop(&mut self) {
        // The join handle will be cancelled when dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_client_creation() {
        let reader = Cursor::new(vec![]);
        let writer = Cursor::new(Vec::new());

        let client = Client::new(reader, writer);
        assert_eq!(client.pending_request_count().await, 0);
        assert!(!client.has_pending_requests().await);
    }

    #[tokio::test]
    async fn test_request_id_generation() {
        let reader = Cursor::new(vec![]);
        let writer = Cursor::new(Vec::new());

        let client = Client::new(reader, writer);

        let id1 = client.next_request_id();
        let id2 = client.next_request_id();

        assert_ne!(id1, id2);
        // IDs should be increasing
        if let (Id::Number(n1), Id::Number(n2)) = (id1, id2) {
            assert!(n1 < n2);
        }
    }

    #[tokio::test]
    async fn test_send_notification() {
        let reader = Cursor::new(vec![]);
        let writer = Cursor::new(Vec::new());

        let client = Client::new(reader, writer);

        // Should not fail to send notification
        let result = client
            .send_notification("test/notification", Some(json!({"key": "value"})))
            .await;
        assert!(result.is_ok());

        let result2 = client.send_notification("test/simple", None).await;
        assert!(result2.is_ok());
    }

    #[test]
    fn test_pending_request_struct() {
        let (sender, _receiver) = oneshot::channel();
        let _pending = PendingRequest { sender };
        // Just test that the struct can be created
    }
}
