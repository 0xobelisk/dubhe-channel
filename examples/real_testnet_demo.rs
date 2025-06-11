/// 真实的 Sui Testnet Phase 1 链下执行演示
/// 使用实际部署的 Counter 合约进行完整的 6 步执行流程
use anyhow::Result;
use log::info;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration};

// Dubhe crates
use dubhe_adapter::sui::SuiAdapter;
use dubhe_adapter::types::{SuiConfig, SuiNetworkType};
use dubhe_loader::CodeLoader;
use dubhe_node::offchain_execution::{
    ExecutionRequest, OffchainExecutionManager, OffchainExecutionResult,
};
use dubhe_vm_runtime::{VmManager, VmType};

// 真实部署的合约信息
const PACKAGE_ID: &str = "0xd4b5a6302ff1cb0a2c8d771a59b00efea442836bf909a5662c0622d9a1adadab";
const COUNTER_OBJECT_ID: &str =
    "0x4ea3c1dd3df67af61cfc305b9e86edeec572ba5b1806ee6e97f3975acd186d9a";
const TESTNET_RPC: &str = "https://fullnode.testnet.sui.io:443";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("🚀 Dubhe Channel Phase 1 - 真实测试网演示");
    info!("==============================================");
    info!("📦 Package ID: {}", PACKAGE_ID);
    info!("🎯 Counter Object ID: {}", COUNTER_OBJECT_ID);
    info!("🌐 Network: Sui Testnet");
    info!("");

    // 初始化真实的 Dubhe 系统
    let offchain_manager = setup_real_dubhe_system().await?;

    // 演示 1: 单笔真实交易
    demo_real_single_transaction(&offchain_manager).await?;

    // 演示 2: 多种 Counter 操作
    demo_real_counter_operations(&offchain_manager).await?;

    // 演示 3: 批量执行
    demo_real_batch_execution(&offchain_manager).await?;

    info!("✅ 真实测试网演示完成！");
    Ok(())
}

/// 设置真实的 Dubhe 系统
async fn setup_real_dubhe_system() -> Result<Arc<OffchainExecutionManager>> {
    info!("🔧 初始化真实 Dubhe 系统...");

    // 1. 配置真实的 Sui 适配器
    let sui_config = SuiConfig {
        rpc_url: TESTNET_RPC.to_string(),
        ws_url: Some("wss://fullnode.testnet.sui.io:443".to_string()),
        network_type: SuiNetworkType::Testnet,
        package_ids: vec![PACKAGE_ID.to_string()],
    };

    let sui_adapter = Arc::new(SuiAdapter::new(sui_config).await?);
    info!("✅ Sui 适配器初始化成功");

    // 2. 初始化 CKB-VM 管理器
    let vm_manager = Arc::new(VmManager::new(VmType::CkbVM));
    info!("✅ CKB-VM 管理器初始化成功");

    // 3. 初始化代码加载器
    let code_loader = Arc::new(CodeLoader::new()?);
    info!("✅ 代码加载器初始化成功");

    // 4. 创建链下执行管理器
    let offchain_manager =
        OffchainExecutionManager::new(sui_adapter.clone(), vm_manager.clone(), code_loader.clone())
            .await?;

    info!("✅ 链下执行管理器初始化成功");
    Ok(Arc::new(offchain_manager))
}

/// 演示真实的单笔交易
async fn demo_real_single_transaction(manager: &Arc<OffchainExecutionManager>) -> Result<()> {
    info!("");
    info!("📈 演示 1: 真实单笔 Counter 交易执行");
    info!("==========================================");

    let request = ExecutionRequest {
        session_id: "real_counter_increment".to_string(),
        package_id: PACKAGE_ID.to_string(),
        function_name: "counter::increment".to_string(),
        arguments: vec![json!(COUNTER_OBJECT_ID)],
        shared_objects: vec![COUNTER_OBJECT_ID.to_string()],
        gas_budget: 10000,
    };

    info!("🎯 执行请求:");
    info!("   会话ID: {}", request.session_id);
    info!("   函数: {}", request.function_name);
    info!("   Gas预算: {}", request.gas_budget);

    let start_time = Instant::now();
    let result = manager.execute_offchain(request).await?;
    let execution_time = start_time.elapsed();

    info!("");
    info!("📊 执行结果:");
    info!("   ✅ 成功: {}", result.success);
    info!("   ⏱️  执行时间: {}ms", execution_time.as_millis());
    info!("   ⛽ Gas使用: {}", result.gas_used);
    info!(
        "   💰 Gas节省: {} ({:.1}%)",
        10000 - result.gas_used,
        ((10000 - result.gas_used) as f64 / 10000.0 * 100.0)
    );
    info!("   🔄 修改对象: {} 个", result.modified_objects.len());

    if let Some(error) = &result.error {
        info!("   ❌ 错误: {}", error);
    }

    Ok(())
}

/// 演示多种真实 Counter 操作
async fn demo_real_counter_operations(manager: &Arc<OffchainExecutionManager>) -> Result<()> {
    info!("");
    info!("🔧 演示 2: 真实 Counter 合约各种操作");
    info!("=======================================");

    let operations = vec![
        (
            "查询当前值",
            "counter::value",
            vec![json!(COUNTER_OBJECT_ID)],
            8000,
        ),
        (
            "再次递增",
            "counter::increment",
            vec![json!(COUNTER_OBJECT_ID)],
            10000,
        ),
        (
            "设置值为100",
            "counter::set_value",
            vec![json!(COUNTER_OBJECT_ID), json!(100)],
            10000,
        ),
        (
            "查询新值",
            "counter::value",
            vec![json!(COUNTER_OBJECT_ID)],
            8000,
        ),
        (
            "重置计数器",
            "counter::reset",
            vec![json!(COUNTER_OBJECT_ID)],
            12000,
        ),
    ];

    let mut total_gas_used = 0;
    let mut total_gas_saved = 0;

    for (name, function, args, gas_budget) in operations {
        info!("   🧪 测试: {}", name);

        let request = ExecutionRequest {
            session_id: format!("real_counter_{}", function.replace("::", "_")),
            package_id: PACKAGE_ID.to_string(),
            function_name: function.to_string(),
            arguments: args,
            shared_objects: vec![COUNTER_OBJECT_ID.to_string()],
            gas_budget,
        };

        match manager.execute_offchain(request).await {
            Ok(result) => {
                let gas_saved = gas_budget - result.gas_used;
                total_gas_used += result.gas_used;
                total_gas_saved += gas_saved;

                info!(
                    "      ✅ {}: {}ms, Gas: {}/{} (节省 {})",
                    name, result.execution_time_ms, result.gas_used, gas_budget, gas_saved
                );
            }
            Err(e) => {
                info!("      ❌ {} 失败: {}", name, e);
            }
        }

        // 在操作之间稍作等待
        sleep(Duration::from_millis(500)).await;
    }

    info!("");
    info!("📈 操作总结:");
    info!("   ⛽ 总Gas使用: {}", total_gas_used);
    info!("   💰 总Gas节省: {}", total_gas_saved);

    Ok(())
}

/// 演示真实的批量执行
async fn demo_real_batch_execution(manager: &Arc<OffchainExecutionManager>) -> Result<()> {
    info!("");
    info!("📊 演示 3: 真实批量交易执行");
    info!("===============================");

    // 创建5个递增操作的批量请求
    let batch_size = 5;
    info!("   🔄 准备执行 {} 个批量操作", batch_size);

    let start_time = Instant::now();
    let mut results = Vec::new();

    for i in 0..batch_size {
        let request = ExecutionRequest {
            session_id: format!("real_batch_increment_{}", i),
            package_id: PACKAGE_ID.to_string(),
            function_name: "counter::increment".to_string(),
            arguments: vec![json!(COUNTER_OBJECT_ID)],
            shared_objects: vec![COUNTER_OBJECT_ID.to_string()],
            gas_budget: 8000,
        };

        match manager.execute_offchain(request).await {
            Ok(result) => {
                info!("   ✅ 批量操作 {} 完成: {}ms", i, result.execution_time_ms);
                results.push(result);
            }
            Err(e) => {
                info!("   ❌ 批量操作 {} 失败: {}", i, e);
            }
        }

        // 批量操作之间的短暂延迟
        sleep(Duration::from_millis(100)).await;
    }

    let total_time = start_time.elapsed();
    let successful = results.len();
    let total_gas_used: u64 = results.iter().map(|r| r.gas_used).sum();
    let total_gas_saved = successful as u64 * 8000 - total_gas_used;

    info!("");
    info!("📈 批量执行结果:");
    info!("   ✅ 成功执行: {}/{}", successful, batch_size);
    info!("   ⏱️  总耗时: {}ms", total_time.as_millis());
    info!(
        "   🚀 平均TPS: {:.2}",
        successful as f64 / total_time.as_secs_f64()
    );
    info!("   ⛽ 总Gas使用: {}", total_gas_used);
    info!(
        "   💰 总Gas节省: {} ({:.1}%)",
        total_gas_saved,
        (total_gas_saved as f64 / (successful as f64 * 8000.0) * 100.0)
    );

    Ok(())
}

/// 真实交易的 6 步流程展示
async fn show_real_execution_steps(
    manager: &Arc<OffchainExecutionManager>,
    request: ExecutionRequest,
) -> Result<OffchainExecutionResult> {
    info!("🔄 开始真实的 6 步执行流程:");

    info!("   🔒 Step 1: 锁定 Sui 测试网共享对象");
    info!("      对象ID: {}", COUNTER_OBJECT_ID);

    info!("   📝 Step 2: 创建链下执行会话");
    info!("      会话ID: {}", request.session_id);

    info!("   ⬇️ Step 3: 从测试网同步状态到链下");
    info!("      RPC: {}", TESTNET_RPC);

    info!("   ⚡ Step 4: CKB-VM 执行 Move 逻辑");
    info!("      函数: {}", request.function_name);

    info!("   ⬆️ Step 5: 同步执行结果回测试网");

    let result = manager.execute_offchain(request).await?;

    info!("   🔓 Step 6: 释放测试网对象锁");
    info!("✅ 真实执行流程完成");

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_real_counter_increment() {
        env_logger::init();

        let manager = setup_real_dubhe_system().await.unwrap();

        let request = ExecutionRequest {
            session_id: "test_real_increment".to_string(),
            package_id: PACKAGE_ID.to_string(),
            function_name: "counter::increment".to_string(),
            arguments: vec![json!(COUNTER_OBJECT_ID)],
            shared_objects: vec![COUNTER_OBJECT_ID.to_string()],
            gas_budget: 10000,
        };

        let result = manager.execute_offchain(request).await.unwrap();

        assert!(result.success);
        assert!(result.gas_used > 0);
        assert!(result.gas_used < 10000);
        assert_eq!(result.modified_objects.len(), 1);
    }
}
