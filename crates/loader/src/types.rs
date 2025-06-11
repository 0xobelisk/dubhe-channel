//! Loader 类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 编译后的合约
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledContract {
    pub original_address: String,
    pub source_type: dubhe_adapter::ContractType,
    pub risc_v_code: Vec<u8>,
    pub entry_points: Vec<String>,
    pub metadata: ContractMetadata,
    pub compiled_at: u64,
}

/// 合约元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub gas_metering: bool,
    pub memory_limit: u64,
    pub stack_limit: u64,
    pub call_depth_limit: u32,
    pub exports: HashMap<String, FunctionSignature>,
}

/// 函数签名
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub name: String,
    pub inputs: Vec<ParamType>,
    pub outputs: Vec<ParamType>,
    pub mutability: Mutability,
}

/// 参数类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamType {
    Uint(usize),
    Int(usize),
    Bool,
    String,
    Bytes,
    Address,
    Array(Box<ParamType>),
    Tuple(Vec<ParamType>),
}

/// 函数可变性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mutability {
    Pure,
    View,
    NonPayable,
    Payable,
}

/// 编译配置
#[derive(Debug, Clone)]
pub struct CompilationConfig {
    pub optimization_level: OptimizationLevel,
    pub target_arch: TargetArch,
    pub enable_gas_metering: bool,
    pub enable_debug_info: bool,
}

/// 优化级别
#[derive(Debug, Clone)]
pub enum OptimizationLevel {
    None,
    Speed,
    Size,
    Aggressive,
}

/// 目标架构
#[derive(Debug, Clone)]
pub enum TargetArch {
    RiscV32,
    RiscV64,
}

impl Default for CompilationConfig {
    fn default() -> Self {
        Self {
            optimization_level: OptimizationLevel::Speed,
            target_arch: TargetArch::RiscV64,
            enable_gas_metering: true,
            enable_debug_info: false,
        }
    }
}

/// 插件句柄
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PluginHandle(pub u64);

/// 插件接口
pub trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn compile(&self, bytecode: &[u8], config: &CompilationConfig) -> anyhow::Result<Vec<u8>>;
}
