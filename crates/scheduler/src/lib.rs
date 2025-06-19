//! Dubhe Channel Scheduler
//!
//! 并行调度策略内核
//!
//! 支持的并行调度范式：
//! 1. Solana Sealevel (账号读写集合)
//! 2. Aptos Block-STM (乐观 STM)
//! 3. Sui Object-DAG (DAG + Fast-path)
//! 4. 自适应调度 (根据工作负载动态选择策略)

pub mod adaptive;
pub mod conflict;
pub mod dispatcher;
pub mod error;
pub mod strategy;
pub mod types;

#[cfg(feature = "solana_parallel")]
pub mod solana_strategy;

#[cfg(feature = "aptos_stm")]
pub mod aptos_strategy;

#[cfg(feature = "sui_object")]
pub mod sui_strategy;

pub use adaptive::*;
pub use conflict::*;
pub use dispatcher::*;
pub use error::*;
pub use strategy::*;
pub use types::*;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

/// 并行调度器主管理器
pub struct ParallelScheduler {
    strategy: Arc<dyn ExecutionStrategy + Send + Sync>,
    adaptive_scheduler: Option<AdaptiveScheduler>,
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

            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported strategy type: {:?}",
                    strategy_type
                ))
            }
        };

        let dispatcher = TransactionDispatcher::new(config.worker_threads)?;

        // 如果启用了自适应调度，创建自适应调度器
        let adaptive_scheduler = if config.enable_adaptive_scheduling {
            Some(AdaptiveScheduler::new()?)
        } else {
            None
        };

        info!(
            "Parallel scheduler initialized with strategy: {:?}",
            strategy_type
        );
        if adaptive_scheduler.is_some() {
            info!("Adaptive scheduling enabled");
        }

        Ok(Self {
            strategy,
            adaptive_scheduler,
            dispatcher,
            config,
        })
    }

    /// 提交交易批次进行并行执行
    pub async fn submit_batch(&self, transactions: Vec<Transaction>) -> Result<BatchResult> {
        info!("Submitting batch of {} transactions", transactions.len());

        // 如果启用了自适应调度，选择最优策略
        let selected_strategy = if let Some(ref adaptive) = self.adaptive_scheduler {
            adaptive.select_optimal_strategy(&transactions).await?
        } else {
            self.get_strategy_type()
        };

        info!("Using strategy: {:?}", selected_strategy);

        // 1. 冲突检测与依赖分析
        let conflict_graph = self.analyze_conflicts(&transactions).await?;

        // 2. 生成执行计划
        let execution_plan = self
            .strategy
            .plan_execution(&transactions, &conflict_graph)
            .await?;

        // 3. 并行执行
        let start_time = std::time::Instant::now();
        let results = self.dispatcher.execute_parallel(execution_plan).await?;
        let execution_time = start_time.elapsed();

        // 4. 收集结果和性能数据
        let stats = ExecutionStats {
            total_transactions: transactions.len(),
            successful_transactions: results.iter().filter(|r| r.success).count(),
            failed_transactions: results.iter().filter(|r| !r.success).count(),
            total_gas_used: results.iter().map(|r| r.gas_used).sum(),
            execution_time_ms: execution_time.as_millis() as u64,
            parallel_efficiency: self.calculate_parallel_efficiency(&results).await,
            conflicts_detected: conflict_graph.edges.len(),
        };

        // 5. 如果使用自适应调度，记录性能数据
        if let Some(ref adaptive) = self.adaptive_scheduler {
            let workload_features = adaptive
                .workload_analyzer
                .extract_features(&transactions)
                .await?;
            let actual_performance = ActualPerformance {
                tps: transactions.len() as f64 / execution_time.as_secs_f64(),
                latency: execution_time.as_millis() as f64,
                efficiency: stats.parallel_efficiency,
                resource_utilization: ResourceUtilization {
                    cpu_usage: 0.8, // TODO: 实际测量
                    memory_usage: 0.6,
                    io_usage: 0.4,
                },
            };

            adaptive
                .record_performance(selected_strategy, workload_features, actual_performance)
                .await?;
        }

        Ok(BatchResult {
            transaction_results: results,
            execution_stats: stats,
        })
    }

    /// 获取调度器状态
    pub async fn get_status(&self) -> SchedulerStatus {
        let base_status = SchedulerStatus {
            strategy_type: self.get_strategy_type(),
            worker_threads: self.config.worker_threads,
            queue_length: self.dispatcher.queue_length().await,
            total_processed: 0,        // TODO: 实现统计
            conflicts_detected: 0,     // TODO: 实现统计
            parallel_efficiency: 0.95, // TODO: 计算实际效率
        };

        // 如果有自适应调度器，获取额外信息
        if let Some(ref adaptive) = self.adaptive_scheduler {
            let adaptive_stats = adaptive.get_statistics().await;
            info!("Adaptive scheduler stats: {:?}", adaptive_stats);
        }

        base_status
    }

    /// 分析交易冲突
    async fn analyze_conflicts(&self, transactions: &[Transaction]) -> Result<ConflictGraph> {
        let mut analyzer = ConflictAnalyzer::new();
        analyzer.analyze(transactions).await
    }

    /// 计算并行效率
    async fn calculate_parallel_efficiency(&self, results: &[TransactionResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }

        let successful_count = results.iter().filter(|r| r.success).count();
        successful_count as f64 / results.len() as f64
    }

    fn get_strategy_type(&self) -> StrategyType {
        // TODO: 从 strategy 获取类型
        StrategyType::SolanaParallel
    }
}

/// 扩展的调度器配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SchedulerConfig {
    pub worker_threads: usize,
    pub batch_size: usize,
    pub max_queue_size: usize,
    pub timeout_ms: u64,
    pub enable_optimistic_execution: bool,

    // 新增：自适应调度配置
    pub enable_adaptive_scheduling: bool,
    pub adaptation_window: usize,
    pub min_samples_for_adaptation: usize,
    pub exploration_rate: f64,
}
