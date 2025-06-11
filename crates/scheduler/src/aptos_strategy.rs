//! Aptos Block-STM 策略

use async_trait::async_trait;
use anyhow::Result;

use crate::strategy::ExecutionStrategy;
use crate::types::*;
use crate::conflict::ConflictGraph;

pub struct AptosStrategy;

impl AptosStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ExecutionStrategy for AptosStrategy {
    async fn plan_execution(&self, transactions: &[Transaction], _conflict_graph: &ConflictGraph) -> Result<ExecutionPlan> {
        // TODO: 实现 Aptos Block-STM 乐观并发控制
        Ok(ExecutionPlan {
            parallel_groups: vec![],
            dependency_order: vec![],
        })
    }

    fn name(&self) -> &str {
        "aptos_block_stm"
    }

    fn description(&self) -> &str {
        "Aptos Block-STM optimistic concurrent execution"
    }
} 