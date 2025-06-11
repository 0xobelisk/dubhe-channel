//! 配置文件验证工具
//!
//! 用于验证 config.toml 文件格式是否正确

use anyhow::Result;
use std::env;
use tracing::{error, info, Level};
use tracing_subscriber;

// 导入本地配置模块
use dubhe_node::config::NodeConfig;

fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // 获取配置文件路径
    let config_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "config.toml".to_string());

    info!("🔍 Validating configuration file: {}", config_path);

    // 验证配置文件
    match NodeConfig::load(&config_path) {
        Ok(config) => {
            info!("✅ Configuration file is valid!");
            info!("📊 Configuration summary:");
            info!("  - API RPC bind: {}", config.api.rpc_bind);
            info!("  - API gRPC bind: {}", config.api.grpc_bind);
            info!("  - API WebSocket bind: {}", config.api.ws_bind);
            info!("  - Data directory: {}", config.node.data_dir);
            info!("  - Execution strategy: {:?}", config.node.strategy);
            info!("  - Worker threads: {}", config.scheduler.worker_threads);
            info!("  - VM type: {:?}", config.vm.default_vm);
            info!("  - VM max instances: {}", config.vm.max_instances);

            // 验证适配器配置
            if config.adapters.ethereum.is_some() {
                info!("  - Ethereum adapter: enabled");
            }
            if config.adapters.sui.is_some() {
                info!("  - Sui adapter: enabled");
            }
            if config.adapters.solana.is_some() {
                info!("  - Solana adapter: enabled");
            }
            if config.adapters.aptos.is_some() {
                info!("  - Aptos adapter: enabled");
            }
            if config.adapters.bitcoin.is_some() {
                info!("  - Bitcoin adapter: enabled");
            }

            // 验证可观测性配置
            if config.observability.enable_prometheus {
                info!(
                    "  - Prometheus metrics: enabled on port {}",
                    config.observability.prometheus_port
                );
            }
            if config.observability.enable_tracing {
                info!("  - Distributed tracing: enabled");
            }

            // 验证安全配置
            if config.security.enable_access_control {
                info!("  - Access control: enabled");
            }
            if config.security.enable_tee {
                info!("  - TEE: enabled");
            }

            // 验证告警配置
            if config.alerting.enable_alerts {
                info!("  - Alerting: enabled");
            }

            // 验证开发配置
            if config.development.debug_mode {
                info!("  - Debug mode: enabled");
            }

            info!("🎉 All configuration sections validated successfully!");
        }
        Err(e) => {
            error!("❌ Configuration validation failed: {}", e);
            error!("💡 Common issues:");
            error!("   - Check TOML syntax (missing quotes, brackets, etc.)");
            error!("   - Verify all required fields are present");
            error!("   - Ensure numeric values are within valid ranges");
            error!("   - Check that string values don't contain invalid characters");
            std::process::exit(1);
        }
    }

    Ok(())
}
