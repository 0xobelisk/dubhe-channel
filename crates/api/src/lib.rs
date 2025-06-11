//! Dubhe Channel API Layer
//!
//! 多协议接口层，支持：
//! - HTTP JSON-RPC (EIP-1474 兼容，支持 Metamask)
//! - gRPC (高性能内部微服务调用)
//! - WebSocket PubSub (事件推送)

pub mod error;
pub mod grpc;
pub mod rpc;
pub mod types;
pub mod ws;

pub use error::ApiError;
pub use grpc::GrpcServer;
pub use rpc::RpcServer;
pub use types::*;
pub use ws::WsServer;

use anyhow::Result;
use tokio::net::TcpListener;
use tracing::info;

/// API 服务器配置
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ApiConfig {
    pub rpc_bind: String,
    pub grpc_bind: String,
    pub ws_bind: String,
    pub max_connections: usize,
    pub request_timeout_ms: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            rpc_bind: "127.0.0.1:8545".to_string(),
            grpc_bind: "127.0.0.1:9090".to_string(),
            ws_bind: "127.0.0.1:8546".to_string(),
            max_connections: 1000,
            request_timeout_ms: 30000,
        }
    }
}

/// API 服务器组合体
pub struct ApiServer {
    config: ApiConfig,
    rpc_server: RpcServer,
    grpc_server: GrpcServer,
    ws_server: WsServer,
}

impl ApiServer {
    pub fn new(config: ApiConfig) -> Self {
        Self {
            rpc_server: RpcServer::new(),
            grpc_server: GrpcServer::new(),
            ws_server: WsServer::new(),
            config,
        }
    }

    /// 启动所有 API 服务
    pub async fn start(&self) -> Result<()> {
        info!("Starting Dubhe Channel API servers...");

        // 并行启动三个服务
        let rpc_task = self.start_rpc();
        let grpc_task = self.start_grpc();
        let ws_task = self.start_ws();

        tokio::try_join!(rpc_task, grpc_task, ws_task)?;

        Ok(())
    }

    async fn start_rpc(&self) -> Result<()> {
        info!("Starting JSON-RPC server on {}", self.config.rpc_bind);
        self.rpc_server.start(&self.config.rpc_bind).await
    }

    async fn start_grpc(&self) -> Result<()> {
        info!("Starting gRPC server on {}", self.config.grpc_bind);
        self.grpc_server.start(&self.config.grpc_bind).await
    }

    async fn start_ws(&self) -> Result<()> {
        info!("Starting WebSocket server on {}", self.config.ws_bind);
        self.ws_server.start(&self.config.ws_bind).await
    }
}
