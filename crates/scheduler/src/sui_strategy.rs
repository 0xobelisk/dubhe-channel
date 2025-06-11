//! Sui Object-DAG 策略

use async_trait::async_trait;
use anyhow::Result;

use crate::strategy::ExecutionStrategy;
use crate::types::*;
use crate::conflict::ConflictGraph;

pub struct SuiStrategy;

impl SuiStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ExecutionStrategy for SuiStrategy {
    async fn plan_execution(&self, transactions: &[Transaction], _conflict_graph: &ConflictGraph) -> Result<ExecutionPlan> {
        // TODO: 实现 Sui Object-DAG 对象级并行
        Ok(ExecutionPlan {
            parallel_groups: vec![],
            dependency_order: vec![],
        })
    }

    fn name(&self) -> &str {
        "sui_object_dag"
    }

    fn description(&self) -> &str {
        "Sui Object-DAG object-level parallel execution"
    }
} 