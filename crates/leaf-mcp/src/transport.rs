//! Stdio transport for MCP communication
//!
//! This module implements the stdio transport layer for communicating
//! with MCP servers via newline-delimited JSON-RPC messages.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{debug, error, trace, warn};

use crate::error::{McpError, McpResult};
use crate::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId};

/// Message sender type for the transport
type ResponseSender = oneshot::Sender<McpResult<JsonRpcResponse>>;

/// Pending requests waiting for responses
type PendingRequests = Arc<RwLock<HashMap<RequestId, ResponseSender>>>;

/// Stdio transport for MCP server communication
pub struct StdioTransport {
    /// Writer for sending messages to stdin
    writer: Arc<tokio::sync::Mutex<ChildStdin>>,
    /// Pending requests awaiting responses
    pending: PendingRequests,
    /// Request ID counter
    next_id: AtomicI64,
    /// Shutdown signal sender
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl StdioTransport {
    /// Create a new stdio transport from process stdin/stdout
    pub fn new(stdin: ChildStdin, stdout: ChildStdout) -> Self {
        let writer = Arc::new(tokio::sync::Mutex::new(stdin));
        let pending: PendingRequests = Arc::new(RwLock::new(HashMap::new()));
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);

        // Spawn reader task
        let pending_clone = Arc::clone(&pending);
        tokio::spawn(Self::reader_task(stdout, pending_clone, shutdown_rx));

        Self {
            writer,
            pending,
            next_id: AtomicI64::new(1),
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// Background task that reads responses from stdout
    async fn reader_task(
        stdout: ChildStdout,
        pending: PendingRequests,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        loop {
            line.clear();

            tokio::select! {
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            debug!("MCP server stdout closed (EOF)");
                            break;
                        }
                        Ok(_) => {
                            let trimmed = line.trim();
                            if trimmed.is_empty() {
                                continue;
                            }

                            trace!("MCP recv: {}", trimmed);

                            // Try to parse as JSON-RPC response
                            match serde_json::from_str::<JsonRpcResponse>(trimmed) {
                                Ok(response) => {
                                    let mut pending_guard = pending.write().await;
                                    if let Some(sender) = pending_guard.remove(&response.id) {
                                        let _ = sender.send(Ok(response));
                                    } else {
                                        warn!("Received response for unknown request ID: {:?}", response.id);
                                    }
                                }
                                Err(e) => {
                                    // Could be a notification or other message
                                    trace!("Could not parse as response: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error reading from MCP server stdout: {}", e);
                            break;
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    debug!("MCP transport reader shutting down");
                    break;
                }
            }
        }

        // Signal error to any pending requests
        let mut pending_guard = pending.write().await;
        for (id, sender) in pending_guard.drain() {
            let _ = sender.send(Err(McpError::Transport(format!(
                "Transport closed while waiting for response to request {:?}",
                id
            ))));
        }
    }

    /// Send a request and wait for response
    pub async fn send_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> McpResult<JsonRpcResponse> {
        let id = RequestId::Number(self.next_id.fetch_add(1, Ordering::SeqCst));
        let request = JsonRpcRequest::new(id.clone(), method, params);

        // Create response channel
        let (tx, rx) = oneshot::channel();

        // Register pending request
        {
            let mut pending = self.pending.write().await;
            pending.insert(id.clone(), tx);
        }

        // Send request
        let json = serde_json::to_string(&request)?;
        trace!("MCP send: {}", json);

        {
            let mut writer = self.writer.lock().await;
            writer.write_all(json.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }

        // Wait for response
        match rx.await {
            Ok(result) => result,
            Err(_) => Err(McpError::Transport(
                "Response channel closed unexpectedly".to_string(),
            )),
        }
    }

    /// Send a notification (no response expected)
    pub async fn send_notification(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> McpResult<()> {
        let notification = JsonRpcNotification::new(method, params);
        let json = serde_json::to_string(&notification)?;
        trace!("MCP send notification: {}", json);

        let mut writer = self.writer.lock().await;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        Ok(())
    }

    /// Send a request with timeout
    pub async fn send_request_with_timeout(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
        timeout_secs: u64,
    ) -> McpResult<JsonRpcResponse> {
        let timeout = std::time::Duration::from_secs(timeout_secs);

        match tokio::time::timeout(timeout, self.send_request(method, params)).await {
            Ok(result) => result,
            Err(_) => Err(McpError::Timeout(timeout_secs)),
        }
    }

    /// Close the transport
    pub async fn close(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        // Take the shutdown sender to signal shutdown
        // This won't block since we're in Drop
        if let Some(tx) = self.shutdown_tx.take() {
            // Try to send shutdown signal (may fail if receiver already dropped)
            let _ = tx.try_send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_from_i64() {
        let id: RequestId = 42i64.into();
        assert_eq!(id, RequestId::Number(42));
    }

    #[test]
    fn test_request_id_from_string() {
        let id: RequestId = "test-id".to_string().into();
        assert_eq!(id, RequestId::String("test-id".to_string()));
    }
}
