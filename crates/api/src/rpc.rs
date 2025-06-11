//! JSON-RPC 服务器
//!
//! 兼容 EIP-1474 标准，支持 Metamask 等钱包直接连接

use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};
use jsonrpc_core::{IoHandler, Params, Value};
use serde_json::json;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

use crate::error::ApiError;
use crate::types::*;

/// JSON-RPC 服务器
pub struct RpcServer {
    handler: IoHandler,
}

impl RpcServer {
    pub fn new() -> Self {
        let mut handler = IoHandler::new();

        // EIP-1474 标准方法
        handler.add_method("eth_chainId", Self::eth_chain_id);
        handler.add_method("eth_blockNumber", Self::eth_block_number);
        handler.add_method("eth_getBalance", Self::eth_get_balance);
        handler.add_method("eth_getTransactionCount", Self::eth_get_transaction_count);
        handler.add_method("eth_sendRawTransaction", Self::eth_send_raw_transaction);
        handler.add_method("eth_call", Self::eth_call);
        handler.add_method("eth_estimateGas", Self::eth_estimate_gas);
        handler.add_method(
            "eth_getTransactionReceipt",
            Self::eth_get_transaction_receipt,
        );
        handler.add_method("eth_getLogs", Self::eth_get_logs);

        // 自定义扩展方法
        handler.add_method("dubhe_getChannelStatus", Self::dubhe_get_channel_status);
        handler.add_method("dubhe_loadContract", Self::dubhe_load_contract);
        handler.add_method("dubhe_getParallelStats", Self::dubhe_get_parallel_stats);

        // Phase 1 链下执行方法
        handler.add_method("dubhe_executeOffchain", Self::dubhe_execute_offchain);
        handler.add_method("dubhe_getOffchainStats", Self::dubhe_get_offchain_stats);

        Self { handler }
    }

    pub async fn start(&self, bind_addr: &str) -> Result<()> {
        let app = Router::new()
            .route("/", post(Self::handle_request))
            .layer(CorsLayer::permissive())
            .with_state(Arc::new(self.handler.clone()));

        let listener = TcpListener::bind(bind_addr).await?;
        info!("JSON-RPC server listening on {}", bind_addr);

        // 使用 hyper 直接服务，避免版本兼容性问题
        let make_service = app.into_make_service();
        let server = hyper::Server::from_tcp(listener.into_std()?)?.serve(make_service);

        server.await?;
        Ok(())
    }

    async fn handle_request(
        State(handler): State<Arc<IoHandler>>,
        Json(request): Json<JsonRpcRequest>,
    ) -> Result<Json<JsonRpcResponse>, StatusCode> {
        let request_str = serde_json::to_string(&request).map_err(|_| StatusCode::BAD_REQUEST)?;
        let response = match handler.handle_request(&request_str).await {
            Some(resp) => resp,
            None => {
                error!("Failed to handle RPC request: {:?}", request);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let parsed_response: JsonRpcResponse =
            serde_json::from_str(&response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(parsed_response))
    }

    // EIP-1474 标准方法实现
    async fn eth_chain_id(_params: Params) -> Result<Value, jsonrpc_core::Error> {
        // Dubhe Channel 使用自定义 Chain ID: 0x44554248 (DUBH)
        Ok(json!("0x44554248"))
    }

    async fn eth_block_number(_params: Params) -> Result<Value, jsonrpc_core::Error> {
        // TODO: 从 state 模块获取当前块高度
        Ok(json!("0x1"))
    }

    async fn eth_get_balance(_params: Params) -> Result<Value, jsonrpc_core::Error> {
        // TODO: 从 state 模块查询账户余额
        Ok(json!("0x0"))
    }

    async fn eth_get_transaction_count(_params: Params) -> Result<Value, jsonrpc_core::Error> {
        // TODO: 从 state 模块查询账户 nonce
        Ok(json!("0x0"))
    }

    async fn eth_send_raw_transaction(_params: Params) -> Result<Value, jsonrpc_core::Error> {
        // TODO: 将原始交易提交到 scheduler 模块
        Ok(json!("0x0"))
    }

    async fn eth_call(_params: Params) -> Result<Value, jsonrpc_core::Error> {
        // TODO: 执行只读合约调用
        Ok(json!("0x"))
    }

    async fn eth_estimate_gas(_params: Params) -> Result<Value, jsonrpc_core::Error> {
        // TODO: 估算 gas 消耗
        Ok(json!("0x5208"))
    }

    async fn eth_get_transaction_receipt(_params: Params) -> Result<Value, jsonrpc_core::Error> {
        // TODO: 从 state 模块获取交易回执
        Ok(Value::Null)
    }

    async fn eth_get_logs(_params: Params) -> Result<Value, jsonrpc_core::Error> {
        // TODO: 从 state 模块查询事件日志
        Ok(json!([]))
    }

    // Dubhe 自定义方法
    async fn dubhe_get_channel_status(_params: Params) -> Result<Value, jsonrpc_core::Error> {
        // TODO: 返回 Channel 运行状态
        Ok(json!({
            "status": "running",
            "parallel_workers": 8,
            "loaded_contracts": 0,
            "tps": 0
        }))
    }

    async fn dubhe_load_contract(_params: Params) -> Result<Value, jsonrpc_core::Error> {
        // TODO: 动态加载合约
        Ok(json!({
            "success": true,
            "contract_id": "0x0",
            "loaded_at": 0
        }))
    }

    async fn dubhe_get_parallel_stats(_params: Params) -> Result<Value, jsonrpc_core::Error> {
        // TODO: 返回并行执行统计
        Ok(json!({
            "parallel_efficiency": 0.95,
            "conflict_rate": 0.05,
            "avg_execution_time_ms": 10
        }))
    }

    // Phase 1 链下执行方法
    async fn dubhe_execute_offchain(_params: Params) -> Result<Value, jsonrpc_core::Error> {
        // TODO: 执行链下交易
        Ok(json!({
            "session_id": "session_123",
            "success": true,
            "gas_used": 5000,
            "execution_time_ms": 50,
            "modified_objects": [],
            "new_objects": []
        }))
    }

    async fn dubhe_get_offchain_stats(_params: Params) -> Result<Value, jsonrpc_core::Error> {
        // TODO: 返回链下执行统计
        Ok(json!({
            "active_sessions": 0,
            "locked_objects": 0,
            "pending_executions": 0,
            "total_gas_saved": 0
        }))
    }
}
