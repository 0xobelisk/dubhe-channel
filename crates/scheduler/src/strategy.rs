//! 执行策略模块

use async_trait::async_trait;
use anyhow::Result;

use crate::types::*;
use crate::conflict::ConflictGraph;

/// 执行策略 trait
#[async_trait]
pub trait ExecutionStrategy {
    /// 分析交易并生成执行计划
    async fn plan_execution(
        &self,
        transactions: &[Transaction],
        conflict_graph: &ConflictGraph,
    ) -> Result<ExecutionPlan>;

    /// 获取策略名称
    fn name(&self) -> &str;

    /// 获取策略描述
    fn description(&self) -> &str;
}

/// 默认串行执行策略（用于测试和回退）
pub struct SequentialStrategy;

#[async_trait]
impl ExecutionStrategy for SequentialStrategy {
    async fn plan_execution(
        &self,
        transactions: &[Transaction],
        _conflict_graph: &ConflictGraph,
    ) -> Result<ExecutionPlan> {
        // 串行执行：每个交易单独一组
        let parallel_groups = transactions
            .iter()
            .enumerate()
            .map(|(i, _)| vec![i])
            .collect();

        let dependency_order = (0..transactions.len()).collect();

        Ok(ExecutionPlan {
            parallel_groups,
            dependency_order,
        })
    }

    fn name(&self) -> &str {
        "sequential"
    }

    fn description(&self) -> &str {
        "Sequential execution strategy (fallback)"
    }
} 