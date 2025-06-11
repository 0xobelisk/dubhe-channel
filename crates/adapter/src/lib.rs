//! Dubhe Channel Adapter
//!
//! 各 L1 轻节点 & ABI 提取模块
//! 支持: Ethereum, Solana, Aptos, Sui, Bitcoin

pub mod aptos;
pub mod btc;
pub mod eth;
pub mod solana;
pub mod sui;
pub mod sui_types;
pub mod traits;
pub mod types;

pub use traits::*;
pub use types::*;

// 重新导出 SuiNetworkType 以便其他模块使用
pub use types::SuiNetworkType;

use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::info;

/// 多链适配器管理器
pub struct AdapterManager {
    adapters: RwLock<HashMap<ChainType, Box<dyn ChainAdapter + Send + Sync>>>,
}

impl AdapterManager {
    pub fn new() -> Self {
        Self {
            adapters: RwLock::new(HashMap::new()),
        }
    }

    /// 注册链适配器
    pub async fn register_adapter(
        &self,
        chain_type: ChainType,
        adapter: Box<dyn ChainAdapter + Send + Sync>,
    ) {
        info!("Registering adapter for {:?}", chain_type);
        self.adapters.write().await.insert(chain_type, adapter);
    }

    /// 获取合约元数据
    pub async fn get_contract_meta(
        &self,
        chain_type: ChainType,
        address: &str,
    ) -> Result<ContractMeta> {
        let adapters = self.adapters.read().await;
        match adapters.get(&chain_type) {
            Some(adapter) => adapter.get_contract_meta(address).await,
            None => Err(anyhow::anyhow!(
                "No adapter found for chain type: {:?}",
                chain_type
            )),
        }
    }

    /// 获取交易回执
    pub async fn get_transaction_receipt(
        &self,
        chain_type: ChainType,
        tx_hash: &str,
    ) -> Result<TransactionReceipt> {
        let adapters = self.adapters.read().await;
        match adapters.get(&chain_type) {
            Some(adapter) => adapter.get_transaction_receipt(tx_hash).await,
            None => Err(anyhow::anyhow!(
                "No adapter found for chain type: {:?}",
                chain_type
            )),
        }
    }

    /// 启动所有适配器的后台任务
    pub async fn start_background_tasks(&self) -> Result<()> {
        info!("Starting adapter background tasks...");

        // TODO: 启动各个适配器的监听任务
        // 比如监听新区块、新交易等

        Ok(())
    }
}
