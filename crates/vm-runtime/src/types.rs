//! VM Runtime 类型定义

use serde::{Deserialize, Serialize};

/// VM 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VmType {
    PolkaVM, // PolkaVM RV32 Harvard 架构
    CkbVM,   // CKB-VM RV64 全指令集
    Cartesi, // Cartesi Linux 沙箱
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: Vec<u8>,
    pub gas_used: u64,
    pub cycles_used: u64,
    pub error: Option<String>,
}

/// VM 快照
#[derive(Debug, Clone)]
pub struct VmSnapshot {
    pub data: Vec<u8>,
    pub vm_type: VmType,
}

/// 执行限制
#[derive(Debug, Clone)]
pub struct ExecutionLimits {
    pub max_memory: u64,
    pub max_cycles: u64,
    pub max_stack: u64,
    pub timeout_ms: u64,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            max_memory: 64 * 1024 * 1024, // 64MB
            max_cycles: 1_000_000,        // 1M cycles
            max_stack: 1 * 1024 * 1024,   // 1MB
            timeout_ms: 30_000,           // 30 seconds
        }
    }
}
