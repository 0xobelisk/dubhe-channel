//! Solana Sealevel 并行策略

use async_trait::async_trait;
use anyhow::Result;

use crate::strategy::ExecutionStrategy;
use crate::types::*;
use crate::conflict::ConflictGraph;

/// Solana 并行执行策略
pub struct SolanaStrategy;

impl SolanaStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ExecutionStrategy for SolanaStrategy {
    async fn plan_execution(
        &self,
        transactions: &[Transaction],
        conflict_graph: &ConflictGraph,
    ) -> Result<ExecutionPlan> {
        // TODO: 实现 Solana 账号读写集合并行算法
        let parallel_groups = vec![transactions.iter().enumerate().map(|(i, _)| i).collect()];
        let dependency_order = (0..transactions.len()).collect();

        Ok(ExecutionPlan {
            parallel_groups,
            dependency_order,
        })
    }

    fn name(&self) -> &str {
        "solana_sealevel"
    }

    fn description(&self) -> &str {
        "Solana Sealevel account read/write set parallel execution"
    }
} 