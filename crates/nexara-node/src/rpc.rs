//! JSON-RPC server for the NEXARA node.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// RPC request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: u64,
}

/// RPC response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<RpcError>,
    pub id: u64,
}

/// RPC error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

/// Simple RPC handler registry.
pub struct RpcServer {
    port: u16,
    handlers: HashMap<String, Box<dyn Fn(serde_json::Value) -> serde_json::Value + Send + Sync>>,
    running: Arc<RwLock<bool>>,
}

impl RpcServer {
    pub fn new(port: u16) -> Self {
        let mut server = RpcServer {
            port,
            handlers: HashMap::new(),
            running: Arc::new(RwLock::new(false)),
        };
        server.register_defaults();
        server
    }

    /// Register default RPC methods.
    fn register_defaults(&mut self) {
        self.handlers.insert("nxr_chainId".to_string(), Box::new(|_| {
            serde_json::json!(20240101_u64)
        }));

        self.handlers.insert("nxr_blockNumber".to_string(), Box::new(|_| {
            serde_json::json!(0_u64) // Placeholder
        }));

        self.handlers.insert("nxr_getBalance".to_string(), Box::new(|params| {
            let _address = params.get(0).and_then(|v| v.as_str()).unwrap_or("");
            serde_json::json!("0x0")
        }));

        self.handlers.insert("nxr_version".to_string(), Box::new(|_| {
            serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
                "chain": "nexara-mainnet"
            })
        }));

        self.handlers.insert("nxr_shardCount".to_string(), Box::new(|_| {
            serde_json::json!(100_u32)
        }));

        self.handlers.insert("nxr_health".to_string(), Box::new(|_| {
            serde_json::json!({ "status": "ok" })
        }));
    }

    /// Register a custom RPC handler.
    pub fn register_handler<F>(&mut self, method: &str, handler: F)
    where
        F: Fn(serde_json::Value) -> serde_json::Value + Send + Sync + 'static,
    {
        self.handlers.insert(method.to_string(), Box::new(handler));
    }

    /// Process a single RPC request.
    pub fn handle_request(&self, request: &RpcRequest) -> RpcResponse {
        match self.handlers.get(&request.method) {
            Some(handler) => RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(handler(request.params.clone())),
                error: None,
                id: request.id,
            },
            None => RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(RpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                }),
                id: request.id,
            },
        }
    }

    /// Get the port this server is configured for.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// List registered methods.
    pub fn methods(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_req(method: &str, id: u64) -> RpcRequest {
        RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: serde_json::json!([]),
            id,
        }
    }

    #[test]
    fn test_chain_id() {
        let server = RpcServer::new(9944);
        let resp = server.handle_request(&make_req("nxr_chainId", 1));
        assert_eq!(resp.result.unwrap(), serde_json::json!(20240101_u64));
    }

    #[test]
    fn test_health() {
        let server = RpcServer::new(9944);
        let resp = server.handle_request(&make_req("nxr_health", 2));
        let result = resp.result.unwrap();
        assert_eq!(result["status"], "ok");
    }

    #[test]
    fn test_method_not_found() {
        let server = RpcServer::new(9944);
        let resp = server.handle_request(&make_req("nonexistent", 3));
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[test]
    fn test_shard_count() {
        let server = RpcServer::new(9944);
        let resp = server.handle_request(&make_req("nxr_shardCount", 4));
        assert_eq!(resp.result.unwrap(), serde_json::json!(100_u32));
    }

    #[test]
    fn test_custom_handler() {
        let mut server = RpcServer::new(9944);
        server.register_handler("custom_method", |_| serde_json::json!("hello"));
        let resp = server.handle_request(&make_req("custom_method", 5));
        assert_eq!(resp.result.unwrap(), serde_json::json!("hello"));
    }

    #[test]
    fn test_methods_list() {
        let server = RpcServer::new(9944);
        let methods = server.methods();
        assert!(methods.contains(&"nxr_chainId".to_string()));
        assert!(methods.contains(&"nxr_health".to_string()));
    }
}
