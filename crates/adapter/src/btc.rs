//! Bitcoin 适配器

use async_trait::async_trait;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::traits::ChainAdapter;
use crate::types::*;

pub struct BitcoinAdapter {
    config: BitcoinConfig,
}

impl BitcoinAdapter {
    pub async fn new(config: BitcoinConfig) -> Result<Self> {
        Ok(Self { config })
    }
}

#[async_trait]
impl ChainAdapter for BitcoinAdapter {
    async fn get_contract_meta(&self, address: &str) -> Result<ContractMeta> {
        todo!("Implement Bitcoin script extraction")
    }

    async fn get_transaction_receipt(&self, tx_hash: &str) -> Result<TransactionReceipt> {
        todo!("Implement Bitcoin transaction info")
    }

    async fn get_balance(&self, address: &str) -> Result<u64> {
        todo!("Implement Bitcoin UTXO balance query")
    }

    async fn get_nonce(&self, address: &str) -> Result<u64> {
        todo!("Bitcoin doesn't use nonce")
    }

    async fn get_block_number(&self) -> Result<u64> {
        todo!("Implement Bitcoin block height query")
    }

    async fn subscribe_new_blocks(&self) -> Result<mpsc::Receiver<String>> {
        todo!("Implement Bitcoin new block subscription")
    }

    async fn subscribe_new_transactions(&self) -> Result<mpsc::Receiver<String>> {
        todo!("Implement Bitcoin new transaction subscription")
    }
} 