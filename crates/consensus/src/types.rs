//! 共识类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Block {
    pub hash: String,
    pub height: u64,
}

/// TPC (时空预知共识) 核心结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TPCConsensus {
    /// 验证节点管理器
    pub validator_manager: ValidatorManager,
    /// 预测激励引擎
    pub prediction_incentive_engine: PredictionIncentiveEngine,
    /// 准确性评估器
    pub accuracy_evaluator: AccuracyEvaluator,
    /// 权重调整器
    pub weight_adjuster: WeightAdjuster,
}

/// 验证节点管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorManager {
    /// 活跃验证节点
    pub active_validators: HashMap<String, ValidatorProfile>,
    /// 节点权重历史
    pub weight_history: HashMap<String, Vec<WeightRecord>>,
    /// 最小权重阈值
    pub min_weight_threshold: f64,
}

/// 验证节点档案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorProfile {
    /// 节点ID
    pub node_id: String,
    /// 当前权重
    pub current_weight: f64,
    /// 质押金额
    pub stake_amount: u64,
    /// 预测统计
    pub prediction_stats: PredictionStats,
    /// 信誉评分
    pub reputation_score: f64,
    /// 加入时间
    pub joined_at: u64,
    /// 最后活跃时间
    pub last_active: u64,
}

/// 预测统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionStats {
    /// 总预测次数
    pub total_predictions: usize,
    /// 成功预测次数
    pub successful_predictions: usize,
    /// 平均准确度
    pub average_accuracy: f64,
    /// 平均置信度
    pub average_confidence: f64,
    /// 连续成功次数
    pub consecutive_successes: usize,
    /// 最大连续成功记录
    pub max_consecutive_successes: usize,
    /// 预测类型分布
    pub prediction_type_distribution: HashMap<PredictionType, usize>,
}

/// 预测类型
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionType {
    /// 序列模式预测
    SequencePattern,
    /// 用户行为预测
    UserBehavior,
    /// 依赖链预测
    DependencyChain,
    /// 历史模式预测
    HistoricalPattern,
    /// 市场驱动预测
    MarketDriven,
    /// 紧急响应预测
    EmergencyResponse,
}

/// 权重记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightRecord {
    /// 权重值
    pub weight: f64,
    /// 调整原因
    pub adjustment_reason: WeightAdjustmentReason,
    /// 调整时间
    pub adjusted_at: u64,
    /// 调整前准确度
    pub accuracy_before: f64,
    /// 调整后预期准确度
    pub expected_accuracy: f64,
}

/// 权重调整原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeightAdjustmentReason {
    /// 预测准确度提升
    AccuracyImprovement,
    /// 预测准确度下降
    AccuracyDegradation,
    /// 连续成功奖励
    ConsecutiveSuccessBonus,
    /// 失败惩罚
    FailurePenalty,
    /// 新节点初始化
    NewNodeInitialization,
    /// 长期表现调整
    LongTermPerformance,
    /// 恶意行为惩罚
    MaliciousBehaviorPenalty,
}

/// 预测激励引擎
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionIncentiveEngine {
    /// 激励配置
    pub config: IncentiveConfig,
    /// 奖励分配器
    pub reward_distributor: RewardDistributor,
    /// 惩罚执行器
    pub penalty_executor: PenaltyExecutor,
    /// 长期激励策略
    pub long_term_strategy: LongTermIncentiveStrategy,
}

/// 激励配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncentiveConfig {
    /// 基础奖励金额
    pub base_reward_amount: u64,
    /// 准确度奖励系数
    pub accuracy_reward_multiplier: f64,
    /// 置信度奖励系数
    pub confidence_reward_multiplier: f64,
    /// 连续成功奖励递增率
    pub consecutive_bonus_rate: f64,
    /// 失败惩罚系数
    pub failure_penalty_rate: f64,
    /// 权重衰减率
    pub weight_decay_rate: f64,
    /// 最大权重限制
    pub max_weight_cap: f64,
    /// 最小权重保障
    pub min_weight_floor: f64,
}

/// 奖励分配器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDistributor {
    /// 总奖励池
    pub total_reward_pool: u64,
    /// 分配历史
    pub distribution_history: Vec<RewardDistribution>,
    /// 待分配奖励
    pub pending_rewards: HashMap<String, u64>,
}

/// 奖励分配记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDistribution {
    /// 分配ID
    pub distribution_id: String,
    /// 受益节点
    pub beneficiary_nodes: HashMap<String, u64>,
    /// 分配原因
    pub distribution_reason: RewardReason,
    /// 分配时间
    pub distributed_at: u64,
    /// 涉及预测
    pub related_predictions: Vec<String>,
}

/// 奖励原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardReason {
    /// 准确预测奖励
    AccuratePrediction,
    /// 高置信度奖励
    HighConfidencePrediction,
    /// 连续成功奖励
    ConsecutiveSuccessBonus,
    /// 创新预测奖励
    InnovativePrediction,
    /// 早期预测奖励
    EarlyPredictionBonus,
    /// 难度预测奖励
    DifficultPredictionBonus,
}

/// 惩罚执行器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenaltyExecutor {
    /// 惩罚历史
    pub penalty_history: Vec<PenaltyRecord>,
    /// 待执行惩罚
    pub pending_penalties: HashMap<String, PenaltyAction>,
}

/// 惩罚记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenaltyRecord {
    /// 惩罚ID
    pub penalty_id: String,
    /// 被惩罚节点
    pub penalized_node: String,
    /// 惩罚类型
    pub penalty_type: PenaltyType,
    /// 惩罚金额
    pub penalty_amount: u64,
    /// 权重减少
    pub weight_reduction: f64,
    /// 惩罚原因
    pub penalty_reason: String,
    /// 执行时间
    pub executed_at: u64,
}

/// 惩罚类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PenaltyType {
    /// 轻微警告
    MinorWarning,
    /// 权重减少
    WeightReduction,
    /// 质押扣除
    StakeSlashing,
    /// 临时禁用
    TemporarySuspension,
    /// 永久封禁
    PermanentBan,
}

/// 惩罚行动
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenaltyAction {
    /// 行动类型
    pub action_type: PenaltyType,
    /// 执行参数
    pub execution_params: PenaltyParams,
    /// 预定执行时间
    pub scheduled_at: u64,
}

/// 惩罚参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenaltyParams {
    /// 权重调整
    pub weight_adjustment: f64,
    /// 质押扣除金额
    pub stake_slash_amount: u64,
    /// 禁用持续时间
    pub suspension_duration: u64,
}

/// 长期激励策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTermIncentiveStrategy {
    /// 策略类型
    pub strategy_type: LongTermStrategyType,
    /// 评估周期
    pub evaluation_period: u64,
    /// 渐进奖励计划
    pub progressive_reward_plan: ProgressiveRewardPlan,
}

/// 长期策略类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LongTermStrategyType {
    /// 线性增长
    LinearGrowth,
    /// 指数增长
    ExponentialGrowth,
    /// 阶梯式增长
    StepwiseGrowth,
    /// 自适应增长
    AdaptiveGrowth,
}

/// 渐进奖励计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressiveRewardPlan {
    /// 里程碑定义
    pub milestones: Vec<RewardMilestone>,
    /// 当前进度
    pub current_progress: HashMap<String, usize>,
}

/// 奖励里程碑
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardMilestone {
    /// 里程碑名称
    pub name: String,
    /// 达成条件
    pub achievement_condition: AchievementCondition,
    /// 奖励内容
    pub reward_content: MilestoneReward,
}

/// 达成条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementCondition {
    /// 连续成功次数
    ConsecutiveSuccesses(usize),
    /// 总准确度
    OverallAccuracy(f64),
    /// 服务时长
    ServiceDuration(u64),
    /// 预测数量
    PredictionVolume(usize),
    /// 创新贡献
    InnovationContribution(f64),
}

/// 里程碑奖励
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneReward {
    /// 奖励金额
    pub reward_amount: u64,
    /// 权重提升
    pub weight_boost: f64,
    /// 特殊权限
    pub special_privileges: Vec<SpecialPrivilege>,
}

/// 特殊权限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecialPrivilege {
    /// 优先预测权
    PriorityPrediction,
    /// 高级预测权限
    AdvancedPredictionAccess,
    /// 治理投票权
    GovernanceVoting,
    /// 协议升级参与权
    ProtocolUpgradeParticipation,
}

/// 准确性评估器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyEvaluator {
    /// 评估配置
    pub config: AccuracyConfig,
    /// 评估历史
    pub evaluation_history: Vec<AccuracyEvaluation>,
}

/// 准确性配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyConfig {
    /// 评估窗口大小
    pub evaluation_window_size: usize,
    /// 准确度计算方法
    pub accuracy_calculation_method: AccuracyMethod,
    /// 权重算法
    pub weighting_algorithm: WeightingAlgorithm,
    /// 时间衰减因子
    pub time_decay_factor: f64,
}

/// 准确度计算方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccuracyMethod {
    /// 简单匹配
    SimpleMatch,
    /// 加权匹配
    WeightedMatch,
    /// 模糊匹配
    FuzzyMatch,
    /// 语义匹配
    SemanticMatch,
}

/// 权重算法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeightingAlgorithm {
    /// 平均权重
    EqualWeight,
    /// 时间加权
    TimeWeighted,
    /// 重要性加权
    ImportanceWeighted,
    /// 动态加权
    DynamicWeight,
}

/// 准确性评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyEvaluation {
    /// 评估ID
    pub evaluation_id: String,
    /// 被评估节点
    pub evaluated_node: String,
    /// 预测ID
    pub prediction_id: String,
    /// 准确度分数
    pub accuracy_score: f64,
    /// 置信度分数
    pub confidence_score: f64,
    /// 时效性分数
    pub timeliness_score: f64,
    /// 综合评分
    pub overall_score: f64,
    /// 评估时间
    pub evaluated_at: u64,
}

/// 权重调整器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightAdjuster {
    /// 调整配置
    pub config: WeightAdjustmentConfig,
    /// 调整历史
    pub adjustment_history: Vec<WeightAdjustment>,
}

/// 权重调整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightAdjustmentConfig {
    /// 调整频率
    pub adjustment_frequency: u64,
    /// 最大单次调整幅度
    pub max_single_adjustment: f64,
    /// 调整平滑因子
    pub smoothing_factor: f64,
    /// 异常检测阈值
    pub anomaly_detection_threshold: f64,
}

/// 权重调整记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightAdjustment {
    /// 调整ID
    pub adjustment_id: String,
    /// 被调整节点
    pub adjusted_node: String,
    /// 调整前权重
    pub weight_before: f64,
    /// 调整后权重
    pub weight_after: f64,
    /// 调整幅度
    pub adjustment_magnitude: f64,
    /// 调整依据
    pub adjustment_basis: AdjustmentBasis,
    /// 调整时间
    pub adjusted_at: u64,
}

/// 调整依据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustmentBasis {
    /// 近期准确度
    pub recent_accuracy: f64,
    /// 历史表现
    pub historical_performance: f64,
    /// 一致性指标
    pub consistency_metric: f64,
    /// 创新度指标
    pub innovation_metric: f64,
}
