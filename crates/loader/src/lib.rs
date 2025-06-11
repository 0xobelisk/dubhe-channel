//! Dubhe Channel Loader
//!
//! Bytecode / ABI 编译缓存与动态插件加载
//!
//! 核心功能：
//! 1. EVM → RISC-V 编译
//! 2. Move/BPF/WASM → RISC-V 编译  
//! 3. LRU + 持久层编译缓存
//! 4. 动态 .so 插件安全加载

pub mod cache;
pub mod compiler;
pub mod dyn_lib;
pub mod error;
pub mod move_compiler;
pub mod types;

pub use cache::*;
pub use compiler::*;
pub use dyn_lib::*;
pub use error::*;
pub use move_compiler::*;
pub use types::*;

use anyhow::Result;
use std::sync::Arc;
use tracing::info;

/// 代码加载器主管理器
pub struct CodeLoader {
    compiler: DefaultCompiler,
    move_compiler: MoveToRiscVCompiler,
    cache: Arc<CompilationCache>,
    plugin_manager: PluginManager,
}

impl CodeLoader {
    pub fn new() -> Result<Self> {
        let cache = Arc::new(CompilationCache::new("./cache")?);
        let compiler = DefaultCompiler::new();
        let move_compiler = MoveToRiscVCompiler::new(move_compiler::MoveCompilerConfig {
            target_arch: move_compiler::RiscVTarget::RV64IMC,
            optimization_level: move_compiler::OptimizationLevel::Speed,
            enable_gas_metering: true,
            enable_debug_info: false,
            stackless_bytecode: true,
        })?;
        let plugin_manager = PluginManager::new();

        info!("Code loader initialized with Move compiler");

        Ok(Self {
            compiler,
            move_compiler,
            cache,
            plugin_manager,
        })
    }

    /// 加载合约代码（优先从缓存读取）
    pub async fn load_contract(
        &self,
        meta: &dubhe_adapter::ContractMeta,
    ) -> Result<CompiledContract> {
        let cache_key = self.generate_cache_key(meta);

        // 尝试从缓存加载
        if let Some(cached) = self.cache.get(&cache_key).await? {
            info!("Contract loaded from cache: {}", meta.address);
            return Ok(cached);
        }

        // 缓存未命中，进行编译
        info!("Compiling contract: {}", meta.address);

        let compiled = match meta.contract_type {
            dubhe_adapter::ContractType::Move => {
                // 使用专门的 Move 编译器
                info!("Using Move → RISC-V compiler for {}", meta.address);
                self.move_compiler.compile_sui_package(meta).await?
            }
            _ => {
                // 使用通用编译器
                info!(
                    "Using default compiler for {:?} contract {}",
                    meta.contract_type, meta.address
                );
                self.compiler.compile(meta).await?
            }
        };

        // 存入缓存
        self.cache.put(&cache_key, &compiled).await?;

        Ok(compiled)
    }

    /// 加载动态插件
    pub fn load_plugin(&mut self, path: &str) -> Result<PluginHandle> {
        self.plugin_manager.load_plugin(path)
    }

    /// 卸载插件
    pub fn unload_plugin(&mut self, handle: PluginHandle) -> Result<()> {
        self.plugin_manager.unload_plugin(handle)
    }

    fn generate_cache_key(&self, meta: &dubhe_adapter::ContractMeta) -> String {
        // 简化实现，避免依赖问题
        format!(
            "{}-{}-{:?}",
            meta.address,
            meta.bytecode.len(),
            meta.contract_type
        )
    }
}
