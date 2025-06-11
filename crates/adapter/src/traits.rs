//! 适配器通用 trait

use async_trait::async_trait;
use anyhow::Result;
use crate::types::*;

/// 链适配器通用接口
#[async_trait]
pub trait ChainAdapter {
    /// 获取合约元数据 (bytecode + ABI)
    async fn get_contract_meta(&self, address: &str) -> Result<ContractMeta>;
    
    /// 获取交易回执
    async fn get_transaction_receipt(&self, tx_hash: &str) -> Result<TransactionReceipt>;
    
    /// 获取账户余额
    async fn get_balance(&self, address: &str) -> Result<u64>;
    
    /// 获取账户 nonce
    async fn get_nonce(&self, address: &str) -> Result<u64>;
    
    /// 获取当前区块高度
    async fn get_block_number(&self) -> Result<u64>;
    
    /// 监听新区块（返回区块哈希）
    async fn subscribe_new_blocks(&self) -> Result<tokio::sync::mpsc::Receiver<String>>;
    
    /// 监听新交易（返回交易哈希）
    async fn subscribe_new_transactions(&self) -> Result<tokio::sync::mpsc::Receiver<String>>;
} 