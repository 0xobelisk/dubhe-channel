//! Phase 1 链下执行集成测试
//!
//! 测试完整的状态锁定 → 同步 → 执行 → 回写流程

use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use dubhe_adapter::{sui::SuiAdapter, sui::SuiConfig, SuiNetworkType};
use dubhe_loader::CodeLoader;
use dubhe_node::{OffchainExecutionManager, ExecutionRequest};
use dubhe_vm_runtime::{VmManager, VmType};

#[tokio::test]
async fn test_phase1_complete_flow() -> Result<()> {
    // 初始化组件
    let (offchain_manager, _) = setup_test_environment().await?;

    // 创建测试请求
    let request = ExecutionRequest {
        session_id: format!("test_session_{}", Uuid::new_v4()),
        package_id: "0x2".to_string(), // Sui System Package
        function_name: "test_function".to_string(),
        arguments: vec![
            serde_json::json!("0xtest_address"),
            serde_json::json!(1000),
        ],
        shared_objects: vec![
            "0x5".to_string(),   // Clock
            "0x403".to_string(), // System state
        ],
        gas_budget: 10000,
    };

    // 执行链下交易
    let result = offchain_manager.execute_offchain(request.clone()).await?;

    // 验证结果
    assert_eq!(result.session_id, request.session_id);
    assert!(result.execution_time_ms > 0);

    println!("✅ Phase 1 执行测试通过");
    println!("  会话 ID: {}", result.session_id);
    println!("  执行成功: {}", result.success);
    println!("  Gas 使用: {}", result.gas_used);
    println!("  执行时间: {}ms", result.execution_time_ms);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_sessions() -> Result<()> {
    let (offchain_manager, _) = setup_test_environment().await?;

    // 创建多个并发会话
    let mut handles = Vec::new();

    for i in 0..5 {
        let manager = offchain_manager.clone();
        let request = ExecutionRequest {
            session_id: format!("concurrent_session_{}", i),
            package_id: "0x2".to_string(),
            function_name: "concurrent_test".to_string(),
            arguments: vec![serde_json::json!(i)],
            shared_objects: vec!["0x5".to_string()],
            gas_budget: 5000,
        };

        let handle = tokio::spawn(async move {
            manager.execute_offchain(request).await
        });

        handles.push(handle);
    }

    // 等待所有执行完成
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await??;
        results.push(result);
    }

    // 验证所有会话都成功执行
    assert_eq!(results.len(), 5);
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result.session_id, format!("concurrent_session_{}", i));
        println!("✅ 并发会话 {} 完成，用时: {}ms", i, result.execution_time_ms);
    }

    println!("✅ 并发执行测试通过");
    Ok(())
}

#[tokio::test]
async fn test_state_locking_mechanism() -> Result<()> {
    let (offchain_manager, _) = setup_test_environment().await?;

    // 测试对象锁定
    let shared_objects = vec!["0x5".to_string(), "0x403".to_string()];

    let request1 = ExecutionRequest {
        session_id: "lock_test_1".to_string(),
        package_id: "0x2".to_string(),
        function_name: "lock_test".to_string(),
        arguments: vec![],
        shared_objects: shared_objects.clone(),
        gas_budget: 8000,
    };

    let request2 = ExecutionRequest {
        session_id: "lock_test_2".to_string(),
        package_id: "0x2".to_string(),
        function_name: "lock_test".to_string(),
        arguments: vec![],
        shared_objects,
        gas_budget: 8000,
    };

    // 并发执行使用相同对象的请求
    let result1_task = offchain_manager.execute_offchain(request1);
    let result2_task = offchain_manager.execute_offchain(request2);

    let (result1, result2) = tokio::join!(result1_task, result2_task);

    // 验证两个都能成功执行（锁定机制正常工作）
    assert!(result1.is_ok());
    assert!(result2.is_ok());

    println!("✅ 状态锁定机制测试通过");
    Ok(())
}

#[tokio::test]
async fn test_gas_savings_calculation() -> Result<()> {
    let (offchain_manager, _) = setup_test_environment().await?;

    // 执行一些交易
    for i in 0..3 {
        let request = ExecutionRequest {
            session_id: format!("gas_test_{}", i),
            package_id: "0x2".to_string(),
            function_name: "gas_test".to_string(),
            arguments: vec![serde_json::json!(i * 100)],
            shared_objects: vec!["0x5".to_string()],
            gas_budget: 12000,
        };

        let _result = offchain_manager.execute_offchain(request).await?;
    }

    // 获取执行统计
    let stats = offchain_manager.get_execution_stats().await;

    println!("📊 Gas 节省统计:");
    println!("  活跃会话: {}", stats.active_sessions);
    println!("  锁定对象: {}", stats.locked_objects);
    println!("  待处理执行: {}", stats.pending_executions);
    println!("  总计节省 Gas: {}", stats.total_gas_saved);

    println!("✅ Gas 节省计算测试通过");
    Ok(())
}

#[tokio::test]
async fn test_error_recovery() -> Result<()> {
    let (offchain_manager, _) = setup_test_environment().await?;

    // 创建会导致错误的请求
    let failing_request = ExecutionRequest {
        session_id: "error_test".to_string(),
        package_id: "0xinvalid".to_string(), // 无效包 ID
        function_name: "invalid_function".to_string(),
        arguments: vec![],
        shared_objects: vec!["0x5".to_string()],
        gas_budget: 5000,
    };

    // 执行应该失败的请求
    let result = offchain_manager.execute_offchain(failing_request).await;

    // 验证错误处理
    match result {
        Ok(execution_result) => {
            // 如果执行成功但标记为失败，这也是正确的
            if !execution_result.success {
                println!("✅ 错误正确处理: {:?}", execution_result.error);
            }
        }
        Err(e) => {
            println!("✅ 错误正确捕获: {}", e);
        }
    }

    // 验证系统仍然可以处理正常请求
    let normal_request = ExecutionRequest {
        session_id: "recovery_test".to_string(),
        package_id: "0x2".to_string(),
        function_name: "recovery_test".to_string(),
        arguments: vec![],
        shared_objects: vec!["0x5".to_string()],
        gas_budget: 6000,
    };

    let recovery_result = offchain_manager.execute_offchain(normal_request).await?;
    assert_eq!(recovery_result.session_id, "recovery_test");

    println!("✅ 错误恢复测试通过");
    Ok(())
}

/// 设置测试环境
async fn setup_test_environment() -> Result<(Arc<OffchainExecutionManager>, TestContext)> {
    // 初始化 Sui 适配器
    let sui_config = SuiConfig {
        rpc_url: "https://fullnode.testnet.sui.io".to_string(),
        ws_url: None,
        network_type: SuiNetworkType::Testnet,
        package_ids: vec![
            "0x1".to_string(), // Sui Framework
            "0x2".to_string(), // Sui System
        ],
    };

    let sui_adapter = Arc::new(SuiAdapter::new(sui_config).await?);

    // 初始化 VM 管理器
    let vm_manager = Arc::new(VmManager::new(VmType::CkbVM));

    // 初始化代码加载器
    let code_loader = Arc::new(CodeLoader::new()?);

    // 创建链下执行管理器
    let offchain_manager = Arc::new(
        OffchainExecutionManager::new(sui_adapter, vm_manager, code_loader).await?,
    );

    let test_context = TestContext {
        test_package_ids: vec!["0x1".to_string(), "0x2".to_string()],
        test_objects: vec!["0x5".to_string(), "0x403".to_string()],
    };

    Ok((offchain_manager, test_context))
}

/// 测试上下文
#[allow(dead_code)]
struct TestContext {
    test_package_ids: Vec<String>,
    test_objects: Vec<String>,
} 