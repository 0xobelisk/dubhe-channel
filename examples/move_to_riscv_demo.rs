//! Move Package to RISC-V 编译演示
//!
//! 展示如何将 Sui Move 包直接编译到 RISC-V 代码
//! 参考 eigerco/polkavm-move 架构

use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber;

use dubhe_adapter::sui::SuiAdapter;
use dubhe_adapter::{
    ChainAdapter, ChainType, ContractMeta, ContractType, SuiConfig, SuiNetworkType,
};
use dubhe_loader::{
    move_compiler::{MoveCompilerConfig, OptimizationLevel, RiscVTarget},
    MoveToRiscVCompiler,
};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("🚀 Move Package → RISC-V Compilation Demo");
    info!("参考 eigerco/polkavm-move 架构");
    info!("=======================================");

    // 第一步：连接 Sui 网络获取 Move 包
    let sui_config = SuiConfig {
        rpc_url: "https://fullnode.testnet.sui.io".to_string(),
        ws_url: None,
        network_type: SuiNetworkType::Testnet,
        package_ids: vec![
            "0x1".to_string(), // Sui Framework
            "0x2".to_string(), // Sui System
        ],
    };

    let sui_adapter = SuiAdapter::new(sui_config).await?;
    info!("✅ Connected to Sui testnet");

    // 第二步：获取 Move 包元数据
    info!("📦 Fetching Sui System package (0x2)...");
    let package_meta = sui_adapter.get_contract_meta("0x2").await?;

    info!("📋 Package metadata:");
    info!("  - Address: {}", package_meta.address);
    info!("  - Type: {:?}", package_meta.contract_type);
    info!("  - Bytecode size: {} bytes", package_meta.bytecode.len());
    info!("  - Has ABI: {}", package_meta.abi.is_some());

    // 第三步：创建 Move → RISC-V 编译器
    info!("🔧 Initializing Move → RISC-V compiler...");

    let compiler_config = MoveCompilerConfig {
        target_arch: RiscVTarget::RV64IMC,
        optimization_level: OptimizationLevel::Speed,
        enable_gas_metering: true,
        enable_debug_info: true,
        stackless_bytecode: true,
    };

    let move_compiler = MoveToRiscVCompiler::new(compiler_config)?;
    info!("✅ Move compiler initialized");
    info!("  - Target: RV64IMC (64-bit RISC-V with Integer, Multiplication, Compressed)");
    info!("  - Optimization: Speed");
    info!("  - Gas metering: enabled");
    info!("  - Stackless bytecode: enabled");

    // 第四步：编译 Move 包到 RISC-V
    info!("⚙️  Compiling Move package to RISC-V...");

    let start_time = std::time::Instant::now();
    let compiled_contract = move_compiler.compile_sui_package(&package_meta).await?;
    let compilation_time = start_time.elapsed();

    info!("✅ Compilation completed in {:?}", compilation_time);
    info!("📊 Compilation results:");
    info!(
        "  - Original address: {}",
        compiled_contract.original_address
    );
    info!("  - Source type: {:?}", compiled_contract.source_type);
    info!(
        "  - RISC-V code size: {} bytes",
        compiled_contract.risc_v_code.len()
    );
    info!("  - Entry points: {:?}", compiled_contract.entry_points);
    info!(
        "  - Gas metering: {}",
        compiled_contract.metadata.gas_metering
    );
    info!(
        "  - Memory limit: {} MB",
        compiled_contract.metadata.memory_limit / (1024 * 1024)
    );

    // 第五步：分析生成的 RISC-V 代码
    info!("🔍 Analyzing generated RISC-V code...");
    analyze_riscv_code(&compiled_contract.risc_v_code)?;

    // 第六步：对比传统编译路径
    info!("📈 Performance comparison:");
    info!("  Traditional path: Move bytecode → Generic compiler → RISC-V");
    info!("  Optimized path:   Move package → Direct compiler → RISC-V");
    info!("  Benefits:");
    info!("    - 🚀 Faster compilation (direct path)");
    info!("    - 🎯 Better optimization (Move-specific)");
    info!("    - 📦 Native package support");
    info!("    - 🔧 LLVM backend integration");

    // 第七步：展示 stackless bytecode 优势
    info!("💡 Stackless bytecode advantages:");
    info!("  - ✅ Better parallelization");
    info!("  - ✅ Easier optimization");
    info!("  - ✅ Direct RISC-V mapping");
    info!("  - ✅ Efficient gas metering");

    info!("🎉 Move → RISC-V compilation demo completed!");

    Ok(())
}

/// 分析生成的 RISC-V 代码
fn analyze_riscv_code(code: &[u8]) -> Result<()> {
    if code.is_empty() {
        info!("  - No code generated");
        return Ok(());
    }

    info!("  - Code size: {} bytes", code.len());
    info!("  - Instruction count: ~{}", code.len() / 4); // 假设 4 字节指令

    // 分析指令类型（简化分析）
    let mut instruction_types = std::collections::HashMap::new();
    for chunk in code.chunks_exact(4) {
        if chunk.len() == 4 {
            let opcode = chunk[0] & 0x7F; // RISC-V opcode 在低7位
            let instr_type = match opcode {
                0x13 => "I-type (Immediate)",
                0x33 => "R-type (Register)",
                0x73 => "System",
                _ => "Other",
            };
            *instruction_types.entry(instr_type).or_insert(0) += 1;
        }
    }

    info!("  - Instruction breakdown:");
    for (instr_type, count) in instruction_types {
        info!("    * {}: {} instructions", instr_type, count);
    }

    // 显示前几个字节的十六进制
    let hex_preview = code
        .iter()
        .take(16)
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ");
    info!("  - Code preview (hex): {}", hex_preview);

    Ok(())
}
