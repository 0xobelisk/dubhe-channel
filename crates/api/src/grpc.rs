//! gRPC 服务器
//! 
//! 高性能内部微服务调用接口

use anyhow::Result;
use tokio::net::TcpListener;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

// TODO: 生成 protobuf 定义后取消注释
// use crate::proto::{
//     execution_service_server::{ExecutionService, ExecutionServiceServer},
//     ExecuteRequest, ExecuteResponse,
// };

/// gRPC 服务器
pub struct GrpcServer {
    // service: ExecutionServiceImpl,
}

impl GrpcServer {
    pub fn new() -> Self {
        Self {
            // service: ExecutionServiceImpl::new(),
        }
    }

    pub async fn start(&self, bind_addr: &str) -> Result<()> {
        info!("gRPC server starting on {}", bind_addr);
        
        // TODO: 实现 gRPC 服务
        // let addr = bind_addr.parse()?;
        // Server::builder()
        //     .add_service(ExecutionServiceServer::new(self.service.clone()))
        //     .serve(addr)
        //     .await?;
        
        // 暂时的占位实现
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

// TODO: 实现 gRPC 服务
// #[derive(Debug, Clone)]
// struct ExecutionServiceImpl {}
// 
// impl ExecutionServiceImpl {
//     fn new() -> Self {
//         Self {}
//     }
// }
// 
// #[tonic::async_trait]
// impl ExecutionService for ExecutionServiceImpl {
//     async fn execute_transaction(
//         &self,
//         request: Request<ExecuteRequest>,
//     ) -> Result<Response<ExecuteResponse>, Status> {
//         let req = request.into_inner();
//         
//         // TODO: 调用 scheduler 执行交易
//         let response = ExecuteResponse {
//             tx_hash: "0x0".to_string(),
//             success: true,
//             gas_used: 21000,
//             output: vec![],
//         };
//         
//         Ok(Response::new(response))
//     }
// } 