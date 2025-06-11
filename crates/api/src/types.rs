//! API 类型定义

use serde::{Deserialize, Serialize};

/// JSON-RPC 请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: serde_json::Value,
}

/// JSON-RPC 响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
    pub id: serde_json::Value,
}

/// JSON-RPC 错误
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// 交易哈希
pub type TxHash = String;

/// 区块哈希
pub type BlockHash = String;

/// 地址
pub type Address = String;

/// WebSocket 事件类型
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum WsEvent {
    NewTransaction { tx_hash: TxHash },
    NewBlock { block_hash: BlockHash, number: u64 },
    ContractLoaded { address: Address, name: String },
    ParallelStats { efficiency: f64, conflicts: u64 },
}
