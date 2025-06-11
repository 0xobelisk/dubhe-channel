//! Dubhe Channel Node
//!
//! 完整节点二进制：组合以上模块启动完整节点

use anyhow::Result;
use clap::{Arg, Command};
use tracing::{error, info};
use tracing_subscriber;

mod config;
mod node;

use config::NodeConfig;
use node::DubheNode;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 解析命令行参数
    let matches = Command::new("dubhe-node")
        .version("0.1.0")
        .author("Dubhe Channel Team")
        .about("Dubhe Channel: Off-chain execution layer with dynamic mainchain loading and parallel execution")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .default_value("config.toml"),
        )
        .arg(
            Arg::new("log-level")
                .long("log-level")
                .value_name("LEVEL")
                .help("Sets the log level")
                .default_value("info"),
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();

    info!("🚀 Starting Dubhe Channel Node...");
    info!("📄 Loading configuration from: {}", config_path);

    // 加载配置
    let config = NodeConfig::load(config_path)?;
    info!("✅ Configuration loaded successfully");

    // 创建并启动节点
    let mut node = DubheNode::new(config).await?;
    info!("🏗️  Node initialized successfully");

    // 启动所有服务
    match node.start().await {
        Ok(_) => {
            info!("🎉 Dubhe Channel Node started successfully!");
            info!("🌐 JSON-RPC: http://127.0.0.1:8545");
            info!("🚀 gRPC: http://127.0.0.1:9090");
            info!("💬 WebSocket: ws://127.0.0.1:8546");

            // 等待中断信号
            tokio::signal::ctrl_c().await?;
            info!("👋 Received shutdown signal, stopping node...");
        }
        Err(e) => {
            error!("❌ Failed to start node: {}", e);
            std::process::exit(1);
        }
    }

    info!("🛑 Dubhe Channel Node stopped");
    Ok(())
}
