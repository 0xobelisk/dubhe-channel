//! 自适应并行调度器
//!
//! 根据工作负载特征动态选择最优的并行执行策略
//! 这是 Dubhe Channel 的核心创新之一

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::strategy::ExecutionStrategy;
use crate::types::*;

/// 自适应调度器
pub struct AdaptiveScheduler {
    /// 策略选择器
    strategy_selector: StrategySelector,
    /// 工作负载分析器
    workload_analyzer: WorkloadAnalyzer,
    /// 性能预测器
    performance_predictor: RwLock<PerformancePredictor>,
    /// 历史性能数据
    performance_history: RwLock<PerformanceHistory>,
    /// 当前活跃策略
    current_strategy: RwLock<StrategyType>,
}

/// 策略选择器
pub struct StrategySelector {
    available_strategies: Vec<StrategyType>,
    selection_algorithm: SelectionAlgorithm,
}

/// 工作负载分析器
pub struct WorkloadAnalyzer {
    feature_extractor: FeatureExtractor,
    conflict_analyzer: ConflictPatternAnalyzer,
}

/// 性能预测器
pub struct PerformancePredictor {
    models: HashMap<StrategyType, PredictionModel>,
    training_data: Vec<TrainingExample>,
    model_accuracy: HashMap<StrategyType, f64>,
}

/// 性能历史记录
#[derive(Debug, Clone)]
pub struct PerformanceHistory {
    records: Vec<PerformanceRecord>,
    strategy_effectiveness: HashMap<StrategyType, StrategyMetrics>,
}

/// 工作负载特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadFeatures {
    /// 交易数量
    pub transaction_count: usize,
    /// 冲突密度 (0.0 - 1.0)
    pub conflict_density: f64,
    /// 读写比例
    pub read_write_ratio: f64,
    /// 地址空间分布熵
    pub address_entropy: f64,
    /// 交易大小分布
    pub transaction_size_distribution: SizeDistribution,
    /// 时间局部性
    pub temporal_locality: f64,
    /// 空间局部性  
    pub spatial_locality: f64,
    /// Gas 使用模式
    pub gas_usage_pattern: GasPattern,
}

/// 预测结果
#[derive(Debug, Clone)]
pub struct PerformancePrediction {
    pub strategy: StrategyType,
    pub predicted_tps: f64,
    pub predicted_latency: f64,
    pub predicted_efficiency: f64,
    pub confidence: f64,
}

/// 性能记录
#[derive(Debug, Clone)]
pub struct PerformanceRecord {
    pub timestamp: u64,
    pub strategy: StrategyType,
    pub workload_features: WorkloadFeatures,
    pub actual_performance: ActualPerformance,
}

/// 实际性能
#[derive(Debug, Clone)]
pub struct ActualPerformance {
    pub tps: f64,
    pub latency: f64,
    pub efficiency: f64,
    pub resource_utilization: ResourceUtilization,
}

impl AdaptiveScheduler {
    pub fn new() -> Result<Self> {
        info!("Initializing adaptive scheduler");

        Ok(Self {
            strategy_selector: StrategySelector::new(),
            workload_analyzer: WorkloadAnalyzer::new(),
            performance_predictor: RwLock::new(PerformancePredictor::new()),
            performance_history: RwLock::new(PerformanceHistory::new()),
            current_strategy: RwLock::new(StrategyType::SolanaParallel),
        })
    }

    /// 核心方法：选择最优策略
    pub async fn select_optimal_strategy(
        &self,
        transactions: &[Transaction],
    ) -> Result<StrategyType> {
        info!(
            "Selecting optimal strategy for {} transactions",
            transactions.len()
        );

        // 1. 提取工作负载特征
        let features = self
            .workload_analyzer
            .extract_features(transactions)
            .await?;
        debug!("Extracted workload features: {:?}", features);

        // 2. 为每个策略预测性能
        let predictions = self.predict_all_strategies(&features).await?;
        debug!("Performance predictions: {:?}", predictions);

        // 3. 选择最优策略
        let optimal_strategy = self.strategy_selector.select_best(&predictions)?;
        info!("Selected optimal strategy: {:?}", optimal_strategy);

        // 4. 更新当前策略
        *self.current_strategy.write().await = optimal_strategy;

        Ok(optimal_strategy)
    }

    /// 预测所有策略的性能
    async fn predict_all_strategies(
        &self,
        features: &WorkloadFeatures,
    ) -> Result<Vec<PerformancePrediction>> {
        let predictor = self.performance_predictor.read().await;
        let mut predictions = Vec::new();

        for &strategy in &[
            StrategyType::SolanaParallel,
            StrategyType::AptosSTM,
            StrategyType::SuiObject,
        ] {
            if let Some(prediction) = predictor.predict(strategy, features)? {
                predictions.push(prediction);
            }
        }

        Ok(predictions)
    }

    /// 记录实际性能并更新模型
    pub async fn record_performance(
        &self,
        strategy: StrategyType,
        workload_features: WorkloadFeatures,
        actual_performance: ActualPerformance,
    ) -> Result<()> {
        let record = PerformanceRecord {
            timestamp: chrono::Utc::now().timestamp() as u64,
            strategy,
            workload_features: workload_features.clone(),
            actual_performance: actual_performance.clone(),
        };

        // 更新历史记录
        self.performance_history
            .write()
            .await
            .add_record(record.clone());

        // 更新预测模型
        self.performance_predictor.write().await.update_model(
            strategy,
            &workload_features,
            &actual_performance,
        )?;

        info!(
            "Recorded performance for strategy {:?}: TPS={:.2}, Latency={:.2}ms",
            strategy, actual_performance.tps, actual_performance.latency
        );

        Ok(())
    }

    /// 获取调度器统计信息
    pub async fn get_statistics(&self) -> AdaptiveSchedulerStats {
        let history = self.performance_history.read().await;
        let current_strategy = *self.current_strategy.read().await;

        AdaptiveSchedulerStats {
            current_strategy,
            total_decisions: history.records.len(),
            strategy_distribution: history.get_strategy_distribution(),
            average_performance: history.get_average_performance(),
            adaptation_efficiency: self.calculate_adaptation_efficiency(&history).await,
        }
    }

    /// 计算自适应效率
    async fn calculate_adaptation_efficiency(&self, history: &PerformanceHistory) -> f64 {
        if history.records.is_empty() {
            return 0.0;
        }

        // 计算自适应调度相比固定策略的性能提升
        let adaptive_avg = history.get_average_performance();
        let fixed_strategy_avg =
            history.get_average_performance_for_strategy(StrategyType::SolanaParallel);

        if fixed_strategy_avg > 0.0 {
            (adaptive_avg - fixed_strategy_avg) / fixed_strategy_avg * 100.0
        } else {
            0.0
        }
    }
}

impl WorkloadAnalyzer {
    pub fn new() -> Self {
        Self {
            feature_extractor: FeatureExtractor::new(),
            conflict_analyzer: ConflictPatternAnalyzer::new(),
        }
    }

    /// 提取工作负载特征
    pub async fn extract_features(&self, transactions: &[Transaction]) -> Result<WorkloadFeatures> {
        info!(
            "Extracting features from {} transactions",
            transactions.len()
        );

        // 并行计算各种特征
        let (
            conflict_density,
            read_write_ratio,
            address_entropy,
            size_distribution,
            temporal_locality,
            spatial_locality,
            gas_pattern,
        ) = tokio::join!(
            self.calculate_conflict_density(transactions),
            self.calculate_read_write_ratio(transactions),
            self.calculate_address_entropy(transactions),
            self.calculate_size_distribution(transactions),
            self.calculate_temporal_locality(transactions),
            self.calculate_spatial_locality(transactions),
            self.calculate_gas_pattern(transactions)
        );

        Ok(WorkloadFeatures {
            transaction_count: transactions.len(),
            conflict_density: conflict_density?,
            read_write_ratio: read_write_ratio?,
            address_entropy: address_entropy?,
            transaction_size_distribution: size_distribution?,
            temporal_locality: temporal_locality?,
            spatial_locality: spatial_locality?,
            gas_usage_pattern: gas_pattern?,
        })
    }

    /// 计算冲突密度
    async fn calculate_conflict_density(&self, transactions: &[Transaction]) -> Result<f64> {
        let total_pairs = transactions.len() * (transactions.len() - 1) / 2;
        if total_pairs == 0 {
            return Ok(0.0);
        }

        let mut conflicts = 0;
        for i in 0..transactions.len() {
            for j in (i + 1)..transactions.len() {
                if self.has_conflict(&transactions[i], &transactions[j]) {
                    conflicts += 1;
                }
            }
        }

        Ok(conflicts as f64 / total_pairs as f64)
    }

    /// 计算读写比例
    async fn calculate_read_write_ratio(&self, transactions: &[Transaction]) -> Result<f64> {
        let total_reads: usize = transactions.iter().map(|tx| tx.read_set.len()).sum();
        let total_writes: usize = transactions.iter().map(|tx| tx.write_set.len()).sum();

        if total_writes == 0 {
            Ok(f64::INFINITY)
        } else {
            Ok(total_reads as f64 / total_writes as f64)
        }
    }

    /// 计算地址空间熵
    async fn calculate_address_entropy(&self, transactions: &[Transaction]) -> Result<f64> {
        let mut address_counts = HashMap::new();
        let mut total_accesses = 0;

        for tx in transactions {
            for addr in &tx.read_set {
                *address_counts.entry(addr.clone()).or_insert(0) += 1;
                total_accesses += 1;
            }
            for addr in &tx.write_set {
                *address_counts.entry(addr.clone()).or_insert(0) += 1;
                total_accesses += 1;
            }
        }

        if total_accesses == 0 {
            return Ok(0.0);
        }

        // 计算香农熵
        let mut entropy = 0.0;
        for count in address_counts.values() {
            let probability = *count as f64 / total_accesses as f64;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        Ok(entropy)
    }

    // 其他特征计算方法...
    async fn calculate_size_distribution(
        &self,
        transactions: &[Transaction],
    ) -> Result<SizeDistribution> {
        let sizes: Vec<usize> = transactions.iter().map(|tx| tx.data.len()).collect();
        Ok(SizeDistribution::from_samples(&sizes))
    }

    async fn calculate_temporal_locality(&self, _transactions: &[Transaction]) -> Result<f64> {
        // TODO: 实现时间局部性计算
        Ok(0.5)
    }

    async fn calculate_spatial_locality(&self, _transactions: &[Transaction]) -> Result<f64> {
        // TODO: 实现空间局部性计算
        Ok(0.5)
    }

    async fn calculate_gas_pattern(&self, transactions: &[Transaction]) -> Result<GasPattern> {
        let gas_limits: Vec<u64> = transactions.iter().map(|tx| tx.gas_limit).collect();
        Ok(GasPattern::from_samples(&gas_limits))
    }

    fn has_conflict(&self, tx1: &Transaction, tx2: &Transaction) -> bool {
        // 检查写写冲突
        for write1 in &tx1.write_set {
            if tx2.write_set.contains(write1) {
                return true;
            }
        }

        // 检查读写冲突
        for write1 in &tx1.write_set {
            if tx2.read_set.contains(write1) {
                return true;
            }
        }

        for write2 in &tx2.write_set {
            if tx1.read_set.contains(write2) {
                return true;
            }
        }

        false
    }
}

impl PerformancePredictor {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            training_data: Vec::new(),
            model_accuracy: HashMap::new(),
        }
    }

    /// 预测特定策略的性能
    pub fn predict(
        &self,
        strategy: StrategyType,
        features: &WorkloadFeatures,
    ) -> Result<Option<PerformancePrediction>> {
        if let Some(model) = self.models.get(&strategy) {
            let prediction = model.predict(features)?;
            let confidence = self.model_accuracy.get(&strategy).copied().unwrap_or(0.5);

            Ok(Some(PerformancePrediction {
                strategy,
                predicted_tps: prediction.tps,
                predicted_latency: prediction.latency,
                predicted_efficiency: prediction.efficiency,
                confidence,
            }))
        } else {
            warn!("No model available for strategy {:?}", strategy);
            Ok(None)
        }
    }

    /// 更新预测模型
    pub fn update_model(
        &mut self,
        strategy: StrategyType,
        features: &WorkloadFeatures,
        actual: &ActualPerformance,
    ) -> Result<()> {
        // 添加训练样本
        self.training_data.push(TrainingExample {
            strategy,
            features: features.clone(),
            actual_performance: actual.clone(),
        });

        // 重新训练模型
        self.retrain_model(strategy)?;

        debug!("Updated model for strategy {:?}", strategy);
        Ok(())
    }

    fn retrain_model(&mut self, strategy: StrategyType) -> Result<()> {
        let strategy_data: Vec<_> = self
            .training_data
            .iter()
            .filter(|example| example.strategy == strategy)
            .collect();

        if strategy_data.len() < 5 {
            // 数据不足，使用默认模型
            self.models
                .insert(strategy, PredictionModel::default_for_strategy(strategy));
            return Ok(());
        }

        // 训练新模型
        let new_model = PredictionModel::train(strategy, &strategy_data)?;

        // 计算模型准确性
        let accuracy = self.calculate_model_accuracy(strategy, &new_model, &strategy_data)?;

        self.models.insert(strategy, new_model);
        self.model_accuracy.insert(strategy, accuracy);

        Ok(())
    }

    fn calculate_model_accuracy(
        &self,
        strategy: StrategyType,
        model: &PredictionModel,
        test_data: &[&TrainingExample],
    ) -> Result<f64> {
        if test_data.is_empty() {
            return Ok(0.5);
        }

        let mut total_error = 0.0;
        for example in test_data {
            let prediction = model.predict(&example.features)?;
            let actual = &example.actual_performance;

            // 计算相对误差
            let tps_error = (prediction.tps - actual.tps).abs() / actual.tps.max(1.0);
            let latency_error =
                (prediction.latency - actual.latency).abs() / actual.latency.max(1.0);

            total_error += (tps_error + latency_error) / 2.0;
        }

        let avg_error = total_error / test_data.len() as f64;
        let accuracy = (1.0 - avg_error.min(1.0)).max(0.0);

        Ok(accuracy)
    }
}

// 辅助类型定义
#[derive(Debug, Clone)]
pub enum SelectionAlgorithm {
    GreedyBest,
    EpsilonGreedy(f64),
    UCB1,
    ThompsonSampling,
}

#[derive(Debug, Clone)]
pub struct FeatureExtractor {
    // 特征提取配置
}

#[derive(Debug, Clone)]
pub struct ConflictPatternAnalyzer {
    // 冲突模式分析配置
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeDistribution {
    pub mean: f64,
    pub std_dev: f64,
    pub percentiles: Vec<(f64, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPattern {
    pub average_gas: f64,
    pub gas_variance: f64,
    pub high_gas_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct ResourceUtilization {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub io_usage: f64,
}

#[derive(Debug, Clone)]
pub struct TrainingExample {
    pub strategy: StrategyType,
    pub features: WorkloadFeatures,
    pub actual_performance: ActualPerformance,
}

#[derive(Debug, Clone)]
pub struct PredictionModel {
    strategy: StrategyType,
    // 简化的线性模型参数
    weights: Vec<f64>,
    bias: f64,
}

#[derive(Debug, Clone)]
pub struct PredictionResult {
    pub tps: f64,
    pub latency: f64,
    pub efficiency: f64,
}

#[derive(Debug, Clone)]
pub struct StrategyMetrics {
    pub avg_tps: f64,
    pub avg_latency: f64,
    pub usage_count: usize,
    pub success_rate: f64,
}

#[derive(Debug, Clone)]
pub struct AdaptiveSchedulerStats {
    pub current_strategy: StrategyType,
    pub total_decisions: usize,
    pub strategy_distribution: HashMap<StrategyType, usize>,
    pub average_performance: f64,
    pub adaptation_efficiency: f64,
}

// 实现方法
impl StrategySelector {
    pub fn new() -> Self {
        Self {
            available_strategies: vec![
                StrategyType::SolanaParallel,
                StrategyType::AptosSTM,
                StrategyType::SuiObject,
            ],
            selection_algorithm: SelectionAlgorithm::GreedyBest,
        }
    }

    pub fn select_best(&self, predictions: &[PerformancePrediction]) -> Result<StrategyType> {
        if predictions.is_empty() {
            return Ok(StrategyType::SolanaParallel); // 默认策略
        }

        match self.selection_algorithm {
            SelectionAlgorithm::GreedyBest => {
                // 选择预测TPS最高的策略
                Ok(predictions
                    .iter()
                    .max_by(|a, b| a.predicted_tps.partial_cmp(&b.predicted_tps).unwrap())
                    .unwrap()
                    .strategy)
            }
            SelectionAlgorithm::EpsilonGreedy(epsilon) => {
                if rand::random::<f64>() < epsilon {
                    // 随机探索
                    Ok(self.available_strategies
                        [rand::random::<usize>() % self.available_strategies.len()])
                } else {
                    // 贪心选择
                    self.select_best(predictions)
                }
            }
            _ => {
                // TODO: 实现其他选择算法
                Ok(StrategyType::SolanaParallel)
            }
        }
    }
}

impl FeatureExtractor {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConflictPatternAnalyzer {
    pub fn new() -> Self {
        Self {}
    }
}

impl SizeDistribution {
    pub fn from_samples(samples: &[usize]) -> Self {
        if samples.is_empty() {
            return Self {
                mean: 0.0,
                std_dev: 0.0,
                percentiles: vec![],
            };
        }

        let mean = samples.iter().sum::<usize>() as f64 / samples.len() as f64;
        let variance = samples
            .iter()
            .map(|&x| (x as f64 - mean).powi(2))
            .sum::<f64>()
            / samples.len() as f64;
        let std_dev = variance.sqrt();

        let mut sorted_samples = samples.to_vec();
        sorted_samples.sort();

        let percentiles = vec![
            (0.5, sorted_samples[samples.len() / 2]),
            (0.9, sorted_samples[(samples.len() as f64 * 0.9) as usize]),
            (0.99, sorted_samples[(samples.len() as f64 * 0.99) as usize]),
        ];

        Self {
            mean,
            std_dev,
            percentiles,
        }
    }
}

impl GasPattern {
    pub fn from_samples(samples: &[u64]) -> Self {
        if samples.is_empty() {
            return Self {
                average_gas: 0.0,
                gas_variance: 0.0,
                high_gas_ratio: 0.0,
            };
        }

        let average_gas = samples.iter().sum::<u64>() as f64 / samples.len() as f64;
        let gas_variance = samples
            .iter()
            .map(|&x| (x as f64 - average_gas).powi(2))
            .sum::<f64>()
            / samples.len() as f64;

        let high_gas_threshold = average_gas * 2.0;
        let high_gas_count = samples
            .iter()
            .filter(|&&gas| gas as f64 > high_gas_threshold)
            .count();
        let high_gas_ratio = high_gas_count as f64 / samples.len() as f64;

        Self {
            average_gas,
            gas_variance,
            high_gas_ratio,
        }
    }
}

impl PredictionModel {
    pub fn default_for_strategy(strategy: StrategyType) -> Self {
        let (weights, bias) = match strategy {
            StrategyType::SolanaParallel => (vec![100.0, -50.0, 20.0], 1000.0),
            StrategyType::AptosSTM => (vec![80.0, -30.0, 30.0], 800.0),
            StrategyType::SuiObject => (vec![120.0, -20.0, 10.0], 1200.0),
        };

        Self {
            strategy,
            weights,
            bias,
        }
    }

    pub fn train(strategy: StrategyType, data: &[&TrainingExample]) -> Result<Self> {
        // 简化的线性回归训练
        // 实际实现应该使用更复杂的ML算法
        Ok(Self::default_for_strategy(strategy))
    }

    pub fn predict(&self, features: &WorkloadFeatures) -> Result<PredictionResult> {
        // 简化的线性预测
        let feature_vec = vec![
            features.transaction_count as f64,
            features.conflict_density,
            features.read_write_ratio.min(10.0), // 限制范围
        ];

        let mut tps = self.bias;
        for (i, &weight) in self.weights.iter().enumerate() {
            if i < feature_vec.len() {
                tps += weight * feature_vec[i];
            }
        }

        tps = tps.max(1.0); // 确保为正数

        let latency = 1000.0 / tps; // 简化的延迟计算
        let efficiency = (1.0 - features.conflict_density) * 0.9 + 0.1;

        Ok(PredictionResult {
            tps,
            latency,
            efficiency,
        })
    }
}

impl PerformanceHistory {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            strategy_effectiveness: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: PerformanceRecord) {
        self.records.push(record.clone());

        // 更新策略效果统计
        let metrics = self
            .strategy_effectiveness
            .entry(record.strategy)
            .or_insert_with(|| StrategyMetrics {
                avg_tps: 0.0,
                avg_latency: 0.0,
                usage_count: 0,
                success_rate: 0.0,
            });

        // 更新平均值
        let old_count = metrics.usage_count as f64;
        let new_count = old_count + 1.0;

        metrics.avg_tps = (metrics.avg_tps * old_count + record.actual_performance.tps) / new_count;
        metrics.avg_latency =
            (metrics.avg_latency * old_count + record.actual_performance.latency) / new_count;
        metrics.usage_count += 1;

        // 简化的成功率计算
        metrics.success_rate = if record.actual_performance.tps > 100.0 {
            1.0
        } else {
            0.0
        };
    }

    pub fn get_strategy_distribution(&self) -> HashMap<StrategyType, usize> {
        let mut distribution = HashMap::new();
        for record in &self.records {
            *distribution.entry(record.strategy).or_insert(0) += 1;
        }
        distribution
    }

    pub fn get_average_performance(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let total_tps: f64 = self.records.iter().map(|r| r.actual_performance.tps).sum();

        total_tps / self.records.len() as f64
    }

    pub fn get_average_performance_for_strategy(&self, strategy: StrategyType) -> f64 {
        let strategy_records: Vec<_> = self
            .records
            .iter()
            .filter(|r| r.strategy == strategy)
            .collect();

        if strategy_records.is_empty() {
            return 0.0;
        }

        let total_tps: f64 = strategy_records
            .iter()
            .map(|r| r.actual_performance.tps)
            .sum();

        total_tps / strategy_records.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adaptive_scheduler_creation() {
        let scheduler = AdaptiveScheduler::new().unwrap();
        assert_eq!(
            *scheduler.current_strategy.read().await,
            StrategyType::SolanaParallel
        );
    }

    #[tokio::test]
    async fn test_workload_feature_extraction() {
        let analyzer = WorkloadAnalyzer::new();
        let transactions = vec![Transaction {
            hash: "0x1".to_string(),
            from: "0xA".to_string(),
            to: Some("0xB".to_string()),
            data: vec![1, 2, 3],
            gas_limit: 21000,
            gas_price: 1000000000,
            nonce: 0,
            read_set: vec!["0xA".to_string()],
            write_set: vec!["0xB".to_string()],
        }];

        let features = analyzer.extract_features(&transactions).await.unwrap();
        assert_eq!(features.transaction_count, 1);
        assert!(features.conflict_density >= 0.0 && features.conflict_density <= 1.0);
    }

    #[test]
    fn test_size_distribution() {
        let samples = vec![100, 200, 150, 300, 250];
        let dist = SizeDistribution::from_samples(&samples);
        assert!(dist.mean > 0.0);
        assert!(dist.std_dev >= 0.0);
    }

    #[test]
    fn test_prediction_model() {
        let model = PredictionModel::default_for_strategy(StrategyType::SolanaParallel);
        let features = WorkloadFeatures {
            transaction_count: 100,
            conflict_density: 0.1,
            read_write_ratio: 2.0,
            address_entropy: 3.0,
            transaction_size_distribution: SizeDistribution::from_samples(&[100, 200]),
            temporal_locality: 0.5,
            spatial_locality: 0.5,
            gas_usage_pattern: GasPattern::from_samples(&[21000, 50000]),
        };

        let prediction = model.predict(&features).unwrap();
        assert!(prediction.tps > 0.0);
        assert!(prediction.latency > 0.0);
        assert!(prediction.efficiency >= 0.0 && prediction.efficiency <= 1.0);
    }
}
