//! Dubhe 节点核心实现

use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};

use dubhe_adapter::AdapterManager;
use dubhe_api::ApiServer;
use dubhe_loader::CodeLoader;
use dubhe_scheduler::ParallelScheduler;
use dubhe_vm_runtime::VmManager;

use crate::config::NodeConfig;

pub use crate::offchain_execution::{
    ExecutionRequest, ExecutionStats, OffchainExecutionManager, OffchainExecutionResult,
};

/// Dubhe Channel 节点
pub struct DubheNode {
    config: NodeConfig,
    api_server: ApiServer,
    adapter_manager: Arc<AdapterManager>,
    code_loader: Arc<CodeLoader>,
    scheduler: Arc<ParallelScheduler>,
    vm_manager: Arc<VmManager>,
    offchain_manager: Arc<OffchainExecutionManager>,
}

impl DubheNode {
    /// 创建新的节点实例
    pub async fn new(config: NodeConfig) -> Result<Self> {
        info!("🔧 Initializing Dubhe Channel components...");

        // 初始化各个组件
        let api_server = ApiServer::new(config.api.clone());
        let adapter_manager = Arc::new(AdapterManager::new());
        let code_loader = Arc::new(CodeLoader::new()?);
        let scheduler = Arc::new(ParallelScheduler::new(
            config.node.strategy,
            config.scheduler.clone(),
        )?);
        let vm_manager = Arc::new(VmManager::new(config.vm.default_vm));

        // 注册适配器
        if let Some(eth_config) = &config.adapters.ethereum {
            let eth_adapter = dubhe_adapter::eth::EthereumAdapter::new(eth_config.clone()).await?;
            adapter_manager
                .register_adapter(dubhe_adapter::ChainType::Ethereum, Box::new(eth_adapter))
                .await;
            info!("✅ Ethereum adapter registered");
        }

        if let Some(sui_config) = &config.adapters.sui {
            let sui_adapter = dubhe_adapter::sui::SuiAdapter::new(sui_config.clone()).await?;
            adapter_manager
                .register_adapter(dubhe_adapter::ChainType::Sui, Box::new(sui_adapter))
                .await;
            info!("✅ Sui adapter registered");
        }

        // TODO: 注册其他链的适配器（Solana, Aptos, Bitcoin）

        // 初始化链下执行管理器
        let sui_adapter = if let Some(sui_config) = &config.adapters.sui {
            Arc::new(dubhe_adapter::sui::SuiAdapter::new(sui_config.clone()).await?)
        } else {
            return Err(anyhow::anyhow!(
                "Sui adapter is required for offchain execution"
            ));
        };

        let offchain_manager = Arc::new(
            OffchainExecutionManager::new(sui_adapter, vm_manager.clone(), code_loader.clone())
                .await?,
        );

        info!("✅ All components initialized successfully");

        Ok(Self {
            config,
            api_server,
            adapter_manager,
            code_loader,
            scheduler,
            vm_manager,
            offchain_manager,
        })
    }

    /// 启动节点
    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 Starting Dubhe Channel Node services...");

        // 创建数据目录
        std::fs::create_dir_all(&self.config.node.data_dir)?;
        info!("📁 Data directory: {}", self.config.node.data_dir);

        // 启动适配器后台任务
        self.adapter_manager.start_background_tasks().await?;
        info!("🔗 Adapter background tasks started");

        // 启动 API 服务器
        let api_config = self.config.api.clone();
        tokio::spawn(async move {
            let api_server = ApiServer::new(api_config);
            if let Err(e) = api_server.start().await {
                error!("❌ API server failed: {}", e);
            }
        });

        info!("🌐 API servers started");
        Ok(())
    }

    /// 获取节点状态
    pub async fn get_status(&self) -> NodeStatus {
        NodeStatus {
            running: true,
            scheduler_status: self.scheduler.get_status().await,
            adapter_count: 1,    // TODO: 从 adapter_manager 获取实际数量
            loaded_contracts: 0, // TODO: 从 code_loader 获取实际数量
        }
    }

    /// 执行链下交易（Phase 1 功能）
    pub async fn execute_offchain(
        &self,
        request: ExecutionRequest,
    ) -> Result<OffchainExecutionResult> {
        info!(
            "🎯 Processing offchain execution request: {}",
            request.session_id
        );
        self.offchain_manager.execute_offchain(request).await
    }

    /// 获取链下执行统计
    pub async fn get_offchain_stats(&self) -> ExecutionStats {
        self.offchain_manager.get_execution_stats().await
    }
}

/// 节点状态信息
#[derive(Debug)]
pub struct NodeStatus {
    pub running: bool,
    pub scheduler_status: dubhe_scheduler::SchedulerStatus,
    pub adapter_count: usize,
    pub loaded_contracts: usize,
}
