/// Phase 1 链下执行完整演示
/// 基于 Counter 合约的端到端流程示例
use anyhow::Result;
use log::{error, info, warn};
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};

use dubhe_adapter::{
    sui::SuiAdapter,
    types::{SuiConfig, SuiNetworkType},
};
use dubhe_loader::CodeLoader;
use dubhe_node::{
    offchain_execution::{ExecutionRequest, ExecutionStats, OffchainExecutionManager},
    DubheNode,
};
use dubhe_vm_runtime::{VmManager, VmType};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("🚀 Dubhe Channel Phase 1 链下执行演示");
    info!("📋 基于 Counter 合约的完整流程");

    // 1. 初始化系统组件
    let (node, offchain_manager) = setup_dubhe_system().await?;

    // 2. 单笔交易演示
    demo_single_transaction(&offchain_manager).await?;

    // 3. 批量交易演示
    demo_batch_transactions(&offchain_manager).await?;

    // 4. 性能分析演示
    demo_performance_analysis(&offchain_manager).await?;

    // 5. 错误恢复演示
    demo_error_handling(&offchain_manager).await?;

    info!("✅ 演示完成");
    Ok(())
}

/// 初始化 Dubhe 系统
async fn setup_dubhe_system() -> Result<(Arc<DubheNode>, Arc<OffchainExecutionManager>)> {
    info!("🔧 初始化 Dubhe Channel 系统...");

    // 配置 Sui 适配器
    let sui_config = SuiConfig {
        rpc_url: "https://fullnode.testnet.sui.io".to_string(),
        ws_url: None,
        network_type: SuiNetworkType::Testnet,
        package_ids: vec![
            "0x1".to_string(),   // Sui Framework
            "0x2".to_string(),   // Sui System
            "0x5".to_string(),   // Clock object
            "0x403".to_string(), // System state
        ],
    };

    let sui_adapter = Arc::new(SuiAdapter::new(sui_config).await?);

    // 初始化 VM 管理器 (CKB-VM)
    let vm_manager = Arc::new(VmManager::new(VmType::CkbVM));

    // 初始化代码加载器
    let code_loader = Arc::new(CodeLoader::new()?);

    // 创建链下执行管理器
    let offchain_manager = Arc::new(
        OffchainExecutionManager::new(sui_adapter.clone(), vm_manager.clone(), code_loader.clone())
            .await?,
    );

    // 创建 Dubhe 节点
    let node = Arc::new(
        DubheNode::new(
            sui_adapter,
            vm_manager,
            code_loader,
            offchain_manager.clone(),
        )
        .await?,
    );

    info!("✅ 系统初始化完成");
    Ok((node, offchain_manager))
}

/// 演示单笔交易的链下执行
async fn demo_single_transaction(manager: &Arc<OffchainExecutionManager>) -> Result<()> {
    info!("📈 演示 1: 单笔 Counter 交易链下执行");

    // 假设我们已经部署了 counter 合约，对象ID为此值
    let counter_object_id = "0x123456789abcdef"; // 实际环境中从部署获取

    let request = ExecutionRequest {
        session_id: "counter_increment_demo".to_string(),
        package_id: counter_object_id.to_string(),
        function_name: "counter::increment".to_string(),
        arguments: vec![json!(counter_object_id)],
        shared_objects: vec![counter_object_id.to_string()],
        gas_budget: 10000,
    };

    info!("   🔄 执行请求: counter::increment");
    info!("   📦 对象ID: {}", counter_object_id);

    let start_time = Instant::now();

    match manager.execute_offchain(request).await {
        Ok(result) => {
            let execution_time = start_time.elapsed();

            info!("   ✅ 执行成功!");
            info!("   ⏱️  执行时间: {}ms", result.execution_time_ms);
            info!("   ⛽ Gas 使用: {}", result.gas_used);
            info!(
                "   💰 预估节省: {}%",
                ((10000 - result.gas_used) as f64 / 10000.0 * 100.0).round()
            );
            info!(
                "   🔄 状态变更: {} 个对象",
                result.effects.created_objects.len() + result.effects.modified_objects.len()
            );

            if !result.events.is_empty() {
                info!("   📝 触发事件: {} 个", result.events.len());
            }
        }
        Err(e) => {
            error!("   ❌ 执行失败: {}", e);
        }
    }

    Ok(())
}

/// 演示批量交易的并发执行
async fn demo_batch_transactions(manager: &Arc<OffchainExecutionManager>) -> Result<()> {
    info!("📊 演示 2: 批量 Counter 交易并发执行");

    let counter_objects = vec![
        "0x123456789abcdef1",
        "0x123456789abcdef2",
        "0x123456789abcdef3",
        "0x123456789abcdef4",
        "0x123456789abcdef5",
    ];

    info!("   🔄 准备执行 {} 个并发会话", counter_objects.len());

    let start_time = Instant::now();
    let mut handles = Vec::new();

    // 创建并发任务
    for (i, object_id) in counter_objects.iter().enumerate() {
        let manager_clone = manager.clone();
        let object_id = object_id.to_string();

        let request = ExecutionRequest {
            session_id: format!("batch_counter_{}", i),
            package_id: object_id.clone(),
            function_name: "counter::increment".to_string(),
            arguments: vec![json!(object_id)],
            shared_objects: vec![object_id.clone()],
            gas_budget: 8000,
        };

        let handle = tokio::spawn(async move { manager_clone.execute_offchain(request).await });

        handles.push(handle);
    }

    // 等待所有任务完成
    let mut successful = 0;
    let mut total_gas_used = 0;
    let mut total_gas_saved = 0;

    for (i, handle) in handles.into_iter().enumerate() {
        match handle.await? {
            Ok(result) => {
                successful += 1;
                total_gas_used += result.gas_used;
                total_gas_saved += 8000 - result.gas_used;
                info!(
                    "   ✅ 会话 {} 完成: {}ms, gas: {}",
                    i, result.execution_time_ms, result.gas_used
                );
            }
            Err(e) => {
                warn!("   ⚠️  会话 {} 失败: {}", i, e);
            }
        }
    }

    let total_time = start_time.elapsed();
    let throughput = successful as f64 / total_time.as_secs_f64();

    info!("   📈 批量执行结果:");
    info!("   ✅ 成功执行: {}/{}", successful, counter_objects.len());
    info!("   ⏱️  总耗时: {}ms", total_time.as_millis());
    info!("   🚀 吞吐量: {:.2} TPS", throughput);
    info!("   ⛽ 总 Gas 使用: {}", total_gas_used);
    info!(
        "   💰 总 Gas 节省: {} ({:.1}%)",
        total_gas_saved,
        (total_gas_saved as f64 / (successful as f64 * 8000.0) * 100.0)
    );

    Ok(())
}

/// 演示性能分析和监控
async fn demo_performance_analysis(manager: &Arc<OffchainExecutionManager>) -> Result<()> {
    info!("📊 演示 3: 性能分析和系统监控");

    // 获取当前系统统计
    let stats = manager.get_execution_stats().await;

    info!("   📈 当前系统状态:");
    info!("   🔄 活跃会话: {}", stats.active_sessions);
    info!("   🔒 锁定对象: {}", stats.locked_objects);
    info!("   ⏳ 待处理执行: {}", stats.pending_executions);
    info!("   💰 累计 Gas 节省: {}", stats.total_gas_saved);

    // 执行一系列测试来收集性能数据
    info!("   🧪 执行性能测试序列...");

    let test_scenarios = vec![
        ("简单递增", "counter::increment", 5000),
        ("值设置", "counter::set_value", 7000),
        ("重置计数", "counter::reset", 6000),
        ("读取值", "counter::value", 3000),
    ];

    for (name, function, gas_budget) in test_scenarios {
        let request = ExecutionRequest {
            session_id: format!("perf_test_{}", name),
            package_id: "0x123456789abcdef".to_string(),
            function_name: function.to_string(),
            arguments: vec![json!("0x123456789abcdef")],
            shared_objects: vec!["0x123456789abcdef".to_string()],
            gas_budget,
        };

        let start = Instant::now();
        match manager.execute_offchain(request).await {
            Ok(result) => {
                let latency = start.elapsed().as_millis();
                let gas_efficiency =
                    ((gas_budget - result.gas_used) as f64 / gas_budget as f64) * 100.0;

                info!(
                    "   📊 {}: {}ms, gas: {}/{} ({:.1}% 节省)",
                    name, latency, result.gas_used, gas_budget, gas_efficiency
                );
            }
            Err(e) => {
                warn!("   ⚠️  {} 测试失败: {}", name, e);
            }
        }

        // 短暂等待避免过度负载
        sleep(Duration::from_millis(100)).await;
    }

    // 最终统计
    let final_stats = manager.get_execution_stats().await;
    let gas_improvement = final_stats.total_gas_saved - stats.total_gas_saved;

    info!("   📈 测试期间 Gas 节省增长: {}", gas_improvement);

    Ok(())
}

/// 演示错误处理和系统恢复
async fn demo_error_handling(manager: &Arc<OffchainExecutionManager>) -> Result<()> {
    info!("🛠️  演示 4: 错误处理和系统恢复");

    // 测试无效包ID
    info!("   🧪 测试无效包ID处理...");
    let invalid_request = ExecutionRequest {
        session_id: "error_test_invalid_package".to_string(),
        package_id: "0xinvalid_package_id".to_string(),
        function_name: "counter::increment".to_string(),
        arguments: vec![],
        shared_objects: vec!["0x123456789abcdef".to_string()],
        gas_budget: 5000,
    };

    match manager.execute_offchain(invalid_request).await {
        Ok(result) if !result.success => {
            info!(
                "   ✅ 错误正确处理: {}",
                result.error.as_ref().unwrap_or(&"未知错误".to_string())
            );
        }
        Err(e) => {
            info!("   ✅ 错误正确捕获: {}", e);
        }
        Ok(_) => {
            warn!("   ⚠️  预期的错误未被捕获");
        }
    }

    // 测试无效函数名
    info!("   🧪 测试无效函数名处理...");
    let invalid_function_request = ExecutionRequest {
        session_id: "error_test_invalid_function".to_string(),
        package_id: "0x123456789abcdef".to_string(),
        function_name: "counter::nonexistent_function".to_string(),
        arguments: vec![],
        shared_objects: vec!["0x123456789abcdef".to_string()],
        gas_budget: 5000,
    };

    match manager.execute_offchain(invalid_function_request).await {
        Ok(result) if !result.success => {
            info!(
                "   ✅ 函数错误正确处理: {}",
                result.error.as_ref().unwrap_or(&"未知错误".to_string())
            );
        }
        Err(e) => {
            info!("   ✅ 函数错误正确捕获: {}", e);
        }
        Ok(_) => {
            warn!("   ⚠️  预期的函数错误未被捕获");
        }
    }

    // 验证系统仍然可以处理正常请求
    info!("   🔄 验证系统恢复能力...");
    let recovery_request = ExecutionRequest {
        session_id: "recovery_test".to_string(),
        package_id: "0x123456789abcdef".to_string(),
        function_name: "counter::increment".to_string(),
        arguments: vec![json!("0x123456789abcdef")],
        shared_objects: vec!["0x123456789abcdef".to_string()],
        gas_budget: 6000,
    };

    match manager.execute_offchain(recovery_request).await {
        Ok(result) => {
            info!("   ✅ 系统恢复验证成功: {}ms", result.execution_time_ms);
        }
        Err(e) => {
            error!("   ❌ 系统恢复失败: {}", e);
        }
    }

    // 最终系统状态检查
    let final_stats = manager.get_execution_stats().await;
    info!("   📊 最终系统状态:");
    info!("   🔄 活跃会话: {}", final_stats.active_sessions);
    info!("   🔒 锁定对象: {}", final_stats.locked_objects);
    info!("   ✅ 错误处理演示完成");

    Ok(())
}
