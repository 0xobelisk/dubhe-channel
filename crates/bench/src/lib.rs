//! Dubhe Channel Benchmarks
//!
//! TPS / Loader / Scheduler 压测工具

pub mod loader_bench;
pub mod scheduler_bench;

use anyhow::Result;

/// 基准测试配置
#[derive(Debug, Clone)]
pub struct BenchConfig {
    pub duration_secs: u64,
    pub concurrent_requests: usize,
    pub warmup_secs: u64,
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            duration_secs: 60,
            concurrent_requests: 100,
            warmup_secs: 10,
        }
    }
}
