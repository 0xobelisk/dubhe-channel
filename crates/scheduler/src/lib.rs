//! Dubhe Channel Scheduler
//! 
//! 并行调度策略内核
//! 
//! 支持的并行调度范式：
//! 1. Solana Sealevel (账号读写集合)
//! 2. Aptos Block-STM (乐观 STM) 
//! 3. Sui Object-DAG (DAG + Fast-path)

pub mod strategy;
pub mod conflict;
pub mod dispatcher;
pub mod types;
pub mod error;

#[cfg(feature = "solana_parallel")]
pub mod solana_strategy;

#[cfg(feature = "aptos_stm")]
pub mod aptos_strategy;

#[cfg(feature = "sui_object")]
pub mod sui_strategy;

pub use strategy::*;
pub use conflict::*;
pub use dispatcher::*;
pub use types::*;
pub use error::*;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

/// 并行调度器主管理器
pub struct ParallelScheduler {
    strategy: Arc<dyn ExecutionStrategy + Send + Sync>,
    dispatcher: TransactionDispatcher,
    config: SchedulerConfig,
}

impl ParallelScheduler {
    pub fn new(strategy_type: StrategyType, config: SchedulerConfig) -> Result<Self> {
        let strategy: Arc<dyn ExecutionStrategy + Send + Sync> = match strategy_type {
            #[cfg(feature = "solana_parallel")]
            StrategyType::SolanaParallel => Arc::new(solana_strategy::SolanaStrategy::new()),
            
            #[cfg(feature = "aptos_stm")]
            StrategyType::AptosSTM => Arc::new(aptos_strategy::AptosStrategy::new()),
            
            #[cfg(feature = "sui_object")]
            StrategyType::SuiObject => Arc::new(sui_strategy::SuiStrategy::new()),
            
            _ => return Err(anyhow::anyhow!("Unsupported strategy type: {:?}", strategy_type)),
        };

        let dispatcher = TransactionDispatcher::new(config.worker_threads)?;

        info!("Parallel scheduler initialized with strategy: {:?}", strategy_type);

        Ok(Self {
            strategy,
            dispatcher,
            config,
        })
    }

    /// 提交交易批次进行并行执行
    pub async fn submit_batch(&self, transactions: Vec<Transaction>) -> Result<BatchResult> {
        info!("Submitting batch of {} transactions", transactions.len());

        // 1. 冲突检测与依赖分析
        let conflict_graph = self.analyze_conflicts(&transactions).await?;
        
        // 2. 生成执行计划
        let execution_plan = self.strategy.plan_execution(&transactions, &conflict_graph).await?;
        
        // 3. 并行执行
        let results = self.dispatcher.execute_parallel(execution_plan).await?;
        
        // 4. 收集结果
        Ok(BatchResult {
            transaction_results: results,
            execution_stats: ExecutionStats::default(), // TODO: 收集实际统计
        })
    }

    /// 获取调度器状态
    pub async fn get_status(&self) -> SchedulerStatus {
        SchedulerStatus {
            strategy_type: self.get_strategy_type(),
            worker_threads: self.config.worker_threads,
            queue_length: self.dispatcher.queue_length().await,
            total_processed: 0, // TODO: 实现统计
            conflicts_detected: 0, // TODO: 实现统计
            parallel_efficiency: 0.95, // TODO: 计算实际效率
        }
    }

    /// 分析交易冲突
    async fn analyze_conflicts(&self, transactions: &[Transaction]) -> Result<ConflictGraph> {
        let mut analyzer = ConflictAnalyzer::new();
        analyzer.analyze(transactions).await
    }

    fn get_strategy_type(&self) -> StrategyType {
        // TODO: 从 strategy 获取类型
        StrategyType::SolanaParallel
    }
} 