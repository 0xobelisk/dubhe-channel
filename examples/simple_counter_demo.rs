/// 简化的 Counter 合约演示
/// 展示 Phase 1 链下执行的核心概念
use anyhow::Result;
use log::info;
use serde_json::json;
use std::time::Instant;

/// 模拟的执行请求
#[derive(Debug, Clone)]
pub struct SimpleExecutionRequest {
    pub session_id: String,
    pub package_id: String,
    pub function_name: String,
    pub arguments: Vec<serde_json::Value>,
    pub shared_objects: Vec<String>,
    pub gas_budget: u64,
}

/// 模拟的执行结果
#[derive(Debug, Clone)]
pub struct SimpleExecutionResult {
    pub session_id: String,
    pub success: bool,
    pub gas_used: u64,
    pub execution_time_ms: u64,
    pub modified_objects: Vec<String>,
    pub new_objects: Vec<String>,
    pub error: Option<String>,
}

/// 模拟的链下执行管理器
pub struct SimpleOffchainManager {
    pub active_sessions: u32,
    pub total_gas_saved: u64,
}

impl SimpleOffchainManager {
    pub fn new() -> Self {
        Self {
            active_sessions: 0,
            total_gas_saved: 0,
        }
    }

    /// 模拟链下执行
    pub async fn execute_offchain(
        &mut self,
        request: SimpleExecutionRequest,
    ) -> Result<SimpleExecutionResult> {
        let start_time = Instant::now();

        info!("🔄 开始执行: {}", request.function_name);
        info!("   会话ID: {}", request.session_id);
        info!("   包ID: {}", request.package_id);
        info!("   Gas预算: {}", request.gas_budget);

        self.active_sessions += 1;

        // 模拟执行时间
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // 模拟Gas使用(节省30%)
        let gas_used = (request.gas_budget as f64 * 0.7) as u64;
        let gas_saved = request.gas_budget - gas_used;
        self.total_gas_saved += gas_saved;

        let execution_time = start_time.elapsed().as_millis() as u64;

        self.active_sessions -= 1;

        info!(
            "✅ 执行完成: {}ms, Gas使用: {}, 节省: {}",
            execution_time, gas_used, gas_saved
        );

        Ok(SimpleExecutionResult {
            session_id: request.session_id,
            success: true,
            gas_used,
            execution_time_ms: execution_time,
            modified_objects: request.shared_objects.clone(),
            new_objects: vec![],
            error: None,
        })
    }

    pub fn get_stats(&self) -> (u32, u64) {
        (self.active_sessions, self.total_gas_saved)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("🚀 Dubhe Channel Phase 1 简化演示");
    info!("📋 基于 Counter 合约的核心流程展示");

    let mut manager = SimpleOffchainManager::new();

    // 演示1: 单笔交易
    demo_single_transaction(&mut manager).await?;

    // 演示2: 批量交易
    demo_batch_transactions(&mut manager).await?;

    // 演示3: 性能统计
    demo_performance_stats(&manager).await?;

    info!("✅ 演示完成");
    Ok(())
}

/// 演示单笔交易
async fn demo_single_transaction(manager: &mut SimpleOffchainManager) -> Result<()> {
    info!("📈 演示 1: 单笔 Counter 交易链下执行");

    let request = SimpleExecutionRequest {
        session_id: "counter_increment_demo".to_string(),
        package_id: "0x123456789abcdef".to_string(),
        function_name: "counter::increment".to_string(),
        arguments: vec![json!("0x123456789abcdef")],
        shared_objects: vec!["0x123456789abcdef".to_string()],
        gas_budget: 10000,
    };

    let result = manager.execute_offchain(request).await?;

    info!(
        "   💰 Gas节省率: {:.1}%",
        ((10000 - result.gas_used) as f64 / 10000.0 * 100.0)
    );
    info!("   🔄 修改对象: {} 个", result.modified_objects.len());

    Ok(())
}

/// 演示批量交易
async fn demo_batch_transactions(manager: &mut SimpleOffchainManager) -> Result<()> {
    info!("📊 演示 2: 批量 Counter 交易并发执行");

    let counter_objects = vec![
        "0x123456789abcdef1",
        "0x123456789abcdef2",
        "0x123456789abcdef3",
        "0x123456789abcdef4",
        "0x123456789abcdef5",
    ];

    let start_time = Instant::now();
    let mut handles = Vec::new();

    for (i, object_id) in counter_objects.iter().enumerate() {
        let request = SimpleExecutionRequest {
            session_id: format!("batch_counter_{}", i),
            package_id: object_id.to_string(),
            function_name: "counter::increment".to_string(),
            arguments: vec![json!(object_id)],
            shared_objects: vec![object_id.to_string()],
            gas_budget: 8000,
        };

        // 注意：这里为了简化，我们序列执行而不是并发
        let result = manager.execute_offchain(request).await?;
        info!("   ✅ 会话 {} 完成: {}ms", i, result.execution_time_ms);
        handles.push(result);
    }

    let total_time = start_time.elapsed();
    let successful = handles.len();
    let total_gas_used: u64 = handles.iter().map(|r| r.gas_used).sum();
    let total_gas_saved = successful as u64 * 8000 - total_gas_used;

    info!("   📈 批量执行结果:");
    info!("   ✅ 成功执行: {}/{}", successful, counter_objects.len());
    info!("   ⏱️  总耗时: {}ms", total_time.as_millis());
    info!(
        "   🚀 平均TPS: {:.2}",
        successful as f64 / total_time.as_secs_f64()
    );
    info!(
        "   💰 总Gas节省: {} ({:.1}%)",
        total_gas_saved,
        (total_gas_saved as f64 / (successful as f64 * 8000.0) * 100.0)
    );

    Ok(())
}

/// 演示性能统计
async fn demo_performance_stats(manager: &SimpleOffchainManager) -> Result<()> {
    info!("📊 演示 3: 系统性能统计");

    let (active_sessions, total_gas_saved) = manager.get_stats();

    info!("   📈 系统状态:");
    info!("   🔄 活跃会话: {}", active_sessions);
    info!("   💰 累计Gas节省: {}", total_gas_saved);
    info!("   📊 估算节省率: ~30%");

    // 模拟各种操作的性能数据
    let operations = vec![
        ("counter::increment", 10000, 7000),
        ("counter::set_value", 12000, 8400),
        ("counter::reset", 10000, 7000),
        ("counter::value", 8000, 5600),
    ];

    info!("   🧪 操作性能分析:");
    for (op, budget, used) in operations {
        let savings = budget - used;
        let savings_pct = (savings as f64 / budget as f64) * 100.0;
        info!(
            "   📊 {}: {}ms, gas: {}/{} ({:.1}% 节省)",
            op, 50, used, budget, savings_pct
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_execution() {
        let mut manager = SimpleOffchainManager::new();

        let request = SimpleExecutionRequest {
            session_id: "test".to_string(),
            package_id: "0x123".to_string(),
            function_name: "counter::increment".to_string(),
            arguments: vec![json!("0x123")],
            shared_objects: vec!["0x123".to_string()],
            gas_budget: 10000,
        };

        let result = manager.execute_offchain(request).await.unwrap();

        assert!(result.success);
        assert!(result.gas_used < 10000); // 应该有gas节省
        assert_eq!(result.modified_objects.len(), 1);
    }

    #[test]
    fn test_manager_creation() {
        let manager = SimpleOffchainManager::new();
        let (active, saved) = manager.get_stats();

        assert_eq!(active, 0);
        assert_eq!(saved, 0);
    }
}
