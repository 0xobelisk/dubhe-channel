//! Dubhe Channel Consensus
//!
//! 内部轻量 BFT / DAG 共识 (可选)
//! 新增：TPC (时空预知共识) - 基于预测准确性的激励机制

pub mod bft;
pub mod dag;
pub mod tpc;
pub mod types; // 新增 TPC 模块

pub use types::*;

use anyhow::Result;

/// 共识管理器
pub struct ConsensusManager {
    /// TPC 共识引擎
    pub tpc_engine: Option<tpc::TPCEngine>,
    /// 传统 BFT 共识
    pub bft_consensus: Option<bft::BftConsensus>,
    /// DAG 共识
    pub dag_consensus: Option<dag::DagConsensus>,
}

impl ConsensusManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            tpc_engine: None,
            bft_consensus: None,
            dag_consensus: None,
        })
    }

    /// 启用 TPC 共识
    pub async fn enable_tpc_consensus(&mut self, config: tpc::TPCConfig) -> Result<()> {
        self.tpc_engine = Some(tpc::TPCEngine::new(config)?);
        Ok(())
    }

    /// 处理预测提交
    pub async fn handle_prediction_submission(
        &self,
        prediction: tpc::PredictionSubmission,
    ) -> Result<tpc::PredictionReceipt> {
        if let Some(engine) = &self.tpc_engine {
            engine.process_prediction(prediction).await
        } else {
            Err(anyhow::anyhow!("TPC consensus not enabled"))
        }
    }

    /// 验证预测结果并分配奖励
    pub async fn validate_and_reward(
        &self,
        actual_transaction: tpc::ActualTransaction,
    ) -> Result<tpc::ValidationResult> {
        if let Some(engine) = &self.tpc_engine {
            engine
                .validate_prediction_and_distribute_rewards(actual_transaction)
                .await
        } else {
            Err(anyhow::anyhow!("TPC consensus not enabled"))
        }
    }
}
