//! 节点配置模块

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use dubhe_adapter::AdapterConfig;
use dubhe_api::ApiConfig;
use dubhe_scheduler::{SchedulerConfig, StrategyType};
use dubhe_vm_runtime::VmType;

/// 节点完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub api: ApiConfig,
    pub adapters: AdapterConfig,
    pub scheduler: SchedulerConfig,
    pub vm: VmConfig,
    pub node: NodeSettings,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub development: DevelopmentConfig,
    #[serde(default)]
    pub testing: TestingConfig,
    #[serde(default)]
    pub performance: PerformanceConfig,
    #[serde(default)]
    pub alerting: AlertingConfig,
}

/// VM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmConfig {
    pub default_vm: VmType,
    pub max_instances: usize,
    #[serde(default)]
    pub move_compiler: MoveCompilerSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveCompilerSettings {
    pub target_arch: String,        // "RV32IM" | "RV64IMC" | "RV64GC"
    pub optimization_level: String, // "None" | "Speed" | "Size" | "Aggressive"
    pub enable_gas_metering: bool,
    pub enable_debug_info: bool,
    pub stackless_bytecode: bool,
    pub enable_llvm: bool, // 是否启用 LLVM 后端
}

impl Default for MoveCompilerSettings {
    fn default() -> Self {
        Self {
            target_arch: "RV64IMC".to_string(),
            optimization_level: "Speed".to_string(),
            enable_gas_metering: true,
            enable_debug_info: false,
            stackless_bytecode: true,
            enable_llvm: false,
        }
    }
}

/// 节点设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSettings {
    pub data_dir: String,
    pub strategy: StrategyType,
    pub enable_metrics: bool,
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_tee: bool,
    pub enable_sgx: bool,
    pub enable_access_control: bool,
    pub audit_level: String,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_tee: false,
            enable_sgx: false,
            enable_access_control: false,
            audit_level: "Basic".to_string(),
        }
    }
}

/// 可观测性配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub enable_prometheus: bool,
    pub prometheus_port: u16,
    pub enable_tracing: bool,
    pub jaeger_endpoint: String,
    pub log_level: String,
    pub structured_logging: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            enable_prometheus: true,
            prometheus_port: 9100,
            enable_tracing: true,
            jaeger_endpoint: "http://localhost:14268/api/traces".to_string(),
            log_level: "info".to_string(),
            structured_logging: true,
        }
    }
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub cache_dir: String,
    pub memory_cache_size: usize,
    pub enable_compression: bool,
    pub cleanup_interval_hours: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_dir: "./cache".to_string(),
            memory_cache_size: 1000,
            enable_compression: true,
            cleanup_interval_hours: 24,
        }
    }
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub max_inbound_connections: usize,
    pub max_outbound_connections: usize,
    pub connection_timeout_secs: u64,
    pub heartbeat_interval_secs: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_inbound_connections: 100,
            max_outbound_connections: 50,
            connection_timeout_secs: 30,
            heartbeat_interval_secs: 10,
        }
    }
}

/// 开发者配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentConfig {
    pub debug_mode: bool,
    pub hot_reload: bool,
    pub verbose_errors: bool,
    pub enable_profiling: bool,
    pub profiling_sample_rate: f64,
}

impl Default for DevelopmentConfig {
    fn default() -> Self {
        Self {
            debug_mode: false,
            hot_reload: false,
            verbose_errors: false,
            enable_profiling: false,
            profiling_sample_rate: 0.1,
        }
    }
}

/// 测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingConfig {
    pub test_mode: bool,
    pub simulated_latency_ms: u64,
    pub enable_chaos_testing: bool,
    pub failure_injection_rate: f64,
}

impl Default for TestingConfig {
    fn default() -> Self {
        Self {
            test_mode: false,
            simulated_latency_ms: 0,
            enable_chaos_testing: false,
            failure_injection_rate: 0.0,
        }
    }
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub tokio_worker_threads: usize,
    pub enable_numa_affinity: bool,
    pub memory_pool_size_mb: u64,
    pub gc_threshold_mb: u64,
    pub enable_cpu_affinity: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            tokio_worker_threads: 0,
            enable_numa_affinity: false,
            memory_pool_size_mb: 512,
            gc_threshold_mb: 256,
            enable_cpu_affinity: false,
        }
    }
}

/// 告警配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    pub enable_alerts: bool,
    pub email: EmailConfig,
    pub slack: SlackConfig,
    pub thresholds: AlertThresholds,
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            enable_alerts: false,
            email: EmailConfig::default(),
            slack: SlackConfig::default(),
            thresholds: AlertThresholds::default(),
        }
    }
}

/// 邮件告警配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub to_addresses: Vec<String>,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_port: 587,
            username: "your-email@gmail.com".to_string(),
            password: "your-app-password".to_string(),
            to_addresses: vec!["admin@dubhe.com".to_string()],
        }
    }
}

/// Slack 告警配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub webhook_url: String,
    pub channel: String,
}

impl Default for SlackConfig {
    fn default() -> Self {
        Self {
            webhook_url: "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK".to_string(),
            channel: "#dubhe-alerts".to_string(),
        }
    }
}

/// 告警阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub tps_drop_percent: f64,
    pub error_rate_percent: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 80.0,
            memory_usage_percent: 85.0,
            disk_usage_percent: 90.0,
            tps_drop_percent: 50.0,
            error_rate_percent: 5.0,
        }
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            adapters: AdapterConfig {
                ethereum: Some(dubhe_adapter::EthereumConfig {
                    rpc_url: "https://eth-mainnet.g.alchemy.com/v2/YOUR-API-KEY".to_string(),
                    ws_url: Some("wss://eth-mainnet.g.alchemy.com/v2/YOUR-API-KEY".to_string()),
                    chain_id: 1,
                }),
                solana: Some(dubhe_adapter::SolanaConfig {
                    rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                    ws_url: Some("wss://api.mainnet-beta.solana.com".to_string()),
                    commitment: "finalized".to_string(),
                }),
                aptos: Some(dubhe_adapter::AptosConfig {
                    rpc_url: "https://fullnode.mainnet.aptoslabs.com/v1".to_string(),
                    faucet_url: None,
                }),
                sui: Some(dubhe_adapter::SuiConfig {
                    rpc_url: "https://fullnode.testnet.sui.io".to_string(),
                    ws_url: None,
                    network_type: dubhe_adapter::SuiNetworkType::Testnet,
                    package_ids: vec!["0x1".to_string()],
                }),
                bitcoin: Some(dubhe_adapter::BitcoinConfig {
                    rpc_url: "http://127.0.0.1:8332".to_string(),
                    rpc_user: "bitcoin".to_string(),
                    rpc_password: "password".to_string(),
                }),
            },
            scheduler: SchedulerConfig::default(),
            vm: VmConfig {
                default_vm: VmType::CkbVM,
                max_instances: 100,
                move_compiler: MoveCompilerSettings::default(),
            },
            node: NodeSettings {
                data_dir: "./data".to_string(),
                strategy: StrategyType::SolanaParallel,
                enable_metrics: true,
            },
            security: SecurityConfig::default(),
            observability: ObservabilityConfig::default(),
            cache: CacheConfig::default(),
            network: NetworkConfig::default(),
            development: DevelopmentConfig::default(),
            testing: TestingConfig::default(),
            performance: PerformanceConfig::default(),
            alerting: AlertingConfig::default(),
        }
    }
}

impl NodeConfig {
    /// 从文件加载配置
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().exists() {
            let content = std::fs::read_to_string(path)?;
            let config: NodeConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            // 如果配置文件不存在，创建默认配置
            let config = Self::default();
            config.save(path)?;
            Ok(config)
        }
    }

    /// 保存配置到文件
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
