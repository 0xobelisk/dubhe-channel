//! 适配器类型定义

use serde::{Deserialize, Serialize};

/// 支持的区块链类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainType {
    Ethereum,
    Solana,
    Aptos,
    Sui,
    Bitcoin,
}

/// 合约类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractType {
    EVM,    // Ethereum Virtual Machine
    Move,   // Aptos/Sui Move
    BPF,    // Solana Berkeley Packet Filter
    Script, // Bitcoin Script
}

/// 统一的合约元数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMeta {
    pub address: String,
    pub chain_type: ChainType,
    pub contract_type: ContractType,
    pub bytecode: Vec<u8>,
    pub abi: Option<String>, // JSON 格式的 ABI
    pub source_code: Option<String>,
    pub compiler_version: Option<String>,
    pub created_at: u64,         // 创建时间戳
    pub creator: Option<String>, // 创建者地址
}

/// 交易回执
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub tx_hash: String,
    pub block_hash: String,
    pub block_number: u64,
    pub transaction_index: u32,
    pub from: String,
    pub to: Option<String>,
    pub gas_used: u64,
    pub status: TransactionStatus,
    pub logs: Vec<EventLog>,
    pub contract_address: Option<String>, // 如果是合约创建交易
}

/// 交易状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Success,
    Failed,
    Pending,
}

/// 事件日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLog {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
}

/// 适配器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    pub ethereum: Option<EthereumConfig>,
    pub solana: Option<SolanaConfig>,
    pub aptos: Option<AptosConfig>,
    pub sui: Option<SuiConfig>,
    pub bitcoin: Option<BitcoinConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumConfig {
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub chain_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub commitment: String, // finalized, confirmed, processed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosConfig {
    pub rpc_url: String,
    pub faucet_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiConfig {
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub network_type: SuiNetworkType,
    pub package_ids: Vec<String>, // 用户配置的包ID列表
}

/// Sui 网络类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuiNetworkType {
    Mainnet,
    Testnet,
    Devnet,
    Localnet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinConfig {
    pub rpc_url: String,
    pub rpc_user: String,
    pub rpc_password: String,
}
