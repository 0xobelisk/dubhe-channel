//! Scheduler 类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 调度策略类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyType {
    SolanaParallel, // Solana Sealevel 账号读写集合并行
    AptosSTM,       // Aptos Block-STM 乐观并发控制
    SuiObject,      // Sui Object-DAG 对象级并行
}

/// 交易表示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub data: Vec<u8>,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub nonce: u64,
    pub read_set: Vec<String>,  // 读取的状态地址
    pub write_set: Vec<String>, // 写入的状态地址
}

/// 执行计划
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub parallel_groups: Vec<Vec<usize>>, // 可并行执行的交易组
    pub dependency_order: Vec<usize>,     // 依赖顺序
}

/// 交易执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub tx_hash: String,
    pub success: bool,
    pub gas_used: u64,
    pub output: Vec<u8>,
    pub logs: Vec<String>,
    pub error: Option<String>,
}

/// 批次执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub transaction_results: Vec<TransactionResult>,
    pub execution_stats: ExecutionStats,
}

/// 执行统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionStats {
    pub total_transactions: usize,
    pub successful_transactions: usize,
    pub failed_transactions: usize,
    pub total_gas_used: u64,
    pub execution_time_ms: u64,
    pub parallel_efficiency: f64,
    pub conflicts_detected: usize,
}

/// 调度器配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchedulerConfig {
    pub worker_threads: usize,
    pub batch_size: usize,
    pub max_queue_size: usize,
    pub timeout_ms: u64,
    pub enable_optimistic_execution: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            batch_size: 100,
            max_queue_size: 10000,
            timeout_ms: 30000,
            enable_optimistic_execution: true,
        }
    }
}

/// 调度器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatus {
    pub strategy_type: StrategyType,
    pub worker_threads: usize,
    pub queue_length: usize,
    pub total_processed: u64,
    pub conflicts_detected: u64,
    pub parallel_efficiency: f64,
}
