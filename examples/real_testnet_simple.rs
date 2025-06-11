/// 简化的真实 Sui Testnet Phase 1 演示
/// 专注于真实的状态同步和结果同步
use anyhow::Result;
use log::info;
use serde_json::json;
use std::time::Instant;

// Dubhe crates
use dubhe_adapter::sui::SuiAdapter;
use dubhe_adapter::types::{SuiConfig, SuiNetworkType};

// 真实部署的合约信息
const PACKAGE_ID: &str = "0xd4b5a6302ff1cb0a2c8d771a59b00efea442836bf909a5662c0622d9a1adadab";
const COUNTER_OBJECT_ID: &str =
    "0x4ea3c1dd3df67af61cfc305b9e86edeec572ba5b1806ee6e97f3975acd186d9a";
const TESTNET_RPC: &str = "https://fullnode.testnet.sui.io:443";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    info!("🚀 启动真实 Sui Testnet Phase 1 演示");
    info!("📦 Package ID: {}", PACKAGE_ID);
    info!("🔗 Counter Object ID: {}", COUNTER_OBJECT_ID);

    // 1. 配置真实的 Sui 适配器
    let sui_config = SuiConfig {
        rpc_url: TESTNET_RPC.to_string(),
        ws_url: Some("wss://fullnode.testnet.sui.io:443".to_string()),
        network_type: SuiNetworkType::Testnet,
        package_ids: vec![PACKAGE_ID.to_string()],
    };

    let sui_adapter = SuiAdapter::new(sui_config).await?;
    info!("✅ Sui 适配器初始化成功");

    // 演示真实的 Step 2: 状态同步到链下
    demo_real_state_sync(&sui_adapter).await?;

    // 演示真实的 Step 3: 结果同步回测试网
    demo_real_result_sync(&sui_adapter).await?;

    info!("🎉 所有真实演示完成！");
    Ok(())
}

/// 演示真实的状态同步到链下
async fn demo_real_state_sync(sui_adapter: &SuiAdapter) -> Result<()> {
    info!("\n🔄 ===== 演示真实的状态同步到链下 =====");

    let start_time = Instant::now();

    // 1. 获取对象的完整状态数据
    info!("📥 获取对象完整状态数据...");
    let object_data = sui_adapter.get_object_data(COUNTER_OBJECT_ID).await?;
    info!(
        "✅ 获取到对象状态数据: {} 字节",
        object_data.to_string().len()
    );

    // 2. 获取对象的原始 BCS 数据
    info!("📥 获取对象 BCS 数据...");
    let bcs_data = sui_adapter.get_object_bcs_data(COUNTER_OBJECT_ID).await?;
    info!("✅ 获取到 BCS 数据: {} 字节", bcs_data.len());

    // 3. 解析对象内容
    if let Some(content) = object_data["data"]["content"].as_object() {
        info!("📊 对象内容:");
        for (key, value) in content {
            info!("  - {}: {}", key, value);
        }
    }

    // 4. 显示对象元数据
    info!("📋 对象元数据:");
    info!(
        "  - 类型: {}",
        object_data["data"]["type"].as_str().unwrap_or("unknown")
    );
    info!(
        "  - 版本: {}",
        object_data["data"]["version"].as_str().unwrap_or("0")
    );
    info!(
        "  - 所有者: {}",
        object_data["data"]["owner"].as_str().unwrap_or("unknown")
    );

    let sync_time = start_time.elapsed();
    info!("✅ 状态同步完成，耗时: {}ms", sync_time.as_millis());

    Ok(())
}

/// 演示真实的结果同步回测试网
async fn demo_real_result_sync(sui_adapter: &SuiAdapter) -> Result<()> {
    info!("\n🔄 ===== 演示真实的结果同步回测试网 =====");

    let start_time = Instant::now();

    // 1. 构建 Move 函数调用交易 (增加计数器)
    info!("🛠️ 构建 Move 函数调用交易...");

    let sender = "0x105b79ec1ee0a31c2faa544104f93b084f78cd8a9d9bb6a02654db21ac9fef8f";
    let package_id = PACKAGE_ID;
    let module = "counter";
    let function = "increment";
    let type_arguments = vec![];
    let arguments = vec![json!(COUNTER_OBJECT_ID)];
    let gas_budget = 50000;

    let tx_data = sui_adapter
        .build_move_call_transaction(
            sender,
            package_id,
            module,
            function,
            type_arguments,
            arguments,
            gas_budget,
        )
        .await?;

    info!("✅ 交易构建成功");

    // 2. 执行干跑测试
    info!("🧪 执行干跑测试...");
    let dry_run_result = sui_adapter.dry_run_transaction(&tx_data).await?;

    // 3. 检查干跑结果
    if let Some(status) = dry_run_result["effects"]["status"]["status"].as_str() {
        if status == "success" {
            info!("✅ 干跑测试成功");

            // 显示 Gas 消耗
            if let Some(gas_used) = dry_run_result["effects"]["gasUsed"]["computationCost"].as_u64()
            {
                info!("⛽ Gas 消耗: {}", gas_used);
                info!("💰 预算剩余: {}", gas_budget - gas_used);
                let gas_efficiency = ((gas_budget - gas_used) as f64 / gas_budget as f64) * 100.0;
                info!("📊 Gas 效率: {:.1}%", gas_efficiency);
            }

            // 显示对象变更
            if let Some(object_changes) = dry_run_result["objectChanges"].as_array() {
                info!("🔄 对象变更:");
                for change in object_changes {
                    if let Some(change_type) = change["type"].as_str() {
                        info!("  - 类型: {}", change_type);
                        if let Some(object_id) = change["objectId"].as_str() {
                            info!("    对象ID: {}", object_id);
                        }
                    }
                }
            }
        } else {
            info!("❌ 干跑测试失败: {}", status);
            if let Some(error) = dry_run_result["effects"]["status"]["error"].as_str() {
                info!("📝 错误详情: {}", error);
            }
        }
    }

    let sync_time = start_time.elapsed();
    info!("✅ 结果同步验证完成，耗时: {}ms", sync_time.as_millis());

    // 注意：这里只是干跑，没有真正执行交易
    // 真正执行需要私钥签名，这里为了演示安全性暂不实现
    info!("ℹ️  注意：这是干跑测试，没有真正修改链上状态");
    info!("🔐 真正执行需要私钥签名");

    Ok(())
}
