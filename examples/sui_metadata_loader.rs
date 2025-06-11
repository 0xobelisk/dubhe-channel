//! Sui 包 Metadata 加载示例
//!
//! 展示如何通过配置文件中的 packageId 加载 Move 模块的标准化 metadata

use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber;

use dubhe_adapter::{
    sui::SuiAdapter,
    types::{SuiConfig, SuiNetworkType},
    ChainAdapter,
};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("🚀 Starting Sui Metadata Loader Example");

    // 创建 Sui 测试网配置
    let config = SuiConfig {
        rpc_url: "https://fullnode.testnet.sui.io".to_string(),
        ws_url: None,
        network_type: SuiNetworkType::Testnet,
        package_ids: vec![
            "0x1".to_string(), // Sui Framework
            "0x2".to_string(), // Sui System
                               // 添加您自己的包ID进行测试
        ],
    };

    // 创建 Sui 适配器
    let adapter = SuiAdapter::new(config).await?;

    info!("📡 Connected to Sui testnet");

    // 演示获取单个包的标准化模块
    info!("🔍 Getting normalized Move modules for package 0x1 (Sui Framework)...");
    match adapter.get_normalized_move_modules_by_package("0x1").await {
        Ok(modules) => {
            info!("✅ Successfully retrieved Sui Framework modules");
            info!(
                "📦 Module count: {}",
                modules.as_object().map(|obj| obj.len()).unwrap_or(0)
            );

            // 显示一些模块名称
            if let Some(obj) = modules.as_object() {
                let module_names: Vec<&String> = obj.keys().take(5).collect();
                info!("📋 Sample modules: {:?}", module_names);
            }
        }
        Err(e) => {
            info!("❌ Failed to get modules: {}", e);
        }
    }

    info!("🔄 Loading metadata for all configured packages...");

    // 批量加载所有配置包的 metadata
    match adapter.load_all_package_metadata().await {
        Ok(metadata_list) => {
            info!(
                "✅ Successfully loaded metadata for {} packages",
                metadata_list.len()
            );

            for (package_id, metadata) in metadata_list {
                let module_count = metadata.as_object().map(|obj| obj.len()).unwrap_or(0);
                info!("  📦 Package {}: {} modules", package_id, module_count);
            }
        }
        Err(e) => {
            info!("❌ Failed to load package metadata: {}", e);
        }
    }

    // 获取网络信息
    info!("🌐 Network Information:");
    info!("  Network Type: {:?}", SuiNetworkType::Testnet);
    info!(
        "  RPC URL: {}",
        SuiAdapter::get_fullnode_url(&SuiNetworkType::Testnet)
    );

    // 演示获取最新检查点
    match adapter.get_block_number().await {
        Ok(checkpoint) => {
            info!("🏁 Latest checkpoint: {}", checkpoint);
        }
        Err(e) => {
            info!("❌ Failed to get latest checkpoint: {}", e);
        }
    }

    info!("🎉 Sui Metadata Loader Example completed!");

    Ok(())
}
