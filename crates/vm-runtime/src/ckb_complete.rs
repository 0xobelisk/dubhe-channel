//! 完整的 CKB-VM 集成实现
//!
//! 这是一个更接近生产环境的 CKB-VM 集成示例

use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, info, warn};

use crate::error::VmError;
use crate::traits::VmInstance;
use crate::types::*;

/// 完整的 CKB-VM 实例实现
pub struct CompleteCkbVmInstance {
    limits: ExecutionLimits,
    code_loaded: bool,
    code_cache: Vec<u8>,
    memory_size: usize,
    cycle_count: u64,
    registers: [u64; 32], // RISC-V 寄存器
}

impl CompleteCkbVmInstance {
    pub fn new() -> Result<Self> {
        info!("Creating complete CKB-VM instance");

        Ok(Self {
            limits: ExecutionLimits::default(),
            code_loaded: false,
            code_cache: Vec::new(),
            memory_size: 0,
            cycle_count: 0,
            registers: [0u64; 32],
        })
    }

    /// 初始化 RISC-V 寄存器
    fn init_registers(&mut self) {
        // 重置所有寄存器
        self.registers.fill(0);

        // 设置栈指针 (sp = x2)
        self.registers[2] = 0x7FFF_FFF0; // 栈顶地址

        // 设置全局指针 (gp = x3)
        self.registers[3] = 0x1000_0000; // 全局数据段

        debug!("RISC-V registers initialized");
    }

    /// 解析 RISC-V 指令
    fn decode_instruction(&self, instruction: u32) -> Result<RiscVInstruction> {
        let opcode = instruction & 0x7F;
        let rd = ((instruction >> 7) & 0x1F) as usize;
        let funct3 = (instruction >> 12) & 0x7;
        let rs1 = ((instruction >> 15) & 0x1F) as usize;
        let rs2 = ((instruction >> 20) & 0x1F) as usize;
        let funct7 = instruction >> 25;

        match opcode {
            0x13 => {
                // I-type instructions (ADDI, etc.)
                let imm = (instruction as i32) >> 20; // 符号扩展
                Ok(RiscVInstruction::IType {
                    opcode,
                    rd,
                    funct3,
                    rs1,
                    imm,
                })
            }
            0x33 => {
                // R-type instructions (ADD, SUB, etc.)
                Ok(RiscVInstruction::RType {
                    opcode,
                    rd,
                    funct3,
                    rs1,
                    rs2,
                    funct7,
                })
            }
            0x73 => {
                // System instructions (EBREAK, ECALL)
                Ok(RiscVInstruction::System { opcode, funct3 })
            }
            _ => Err(
                VmError::ExecutionFailed(format!("Unsupported opcode: 0x{:02x}", opcode)).into(),
            ),
        }
    }

    /// 执行单条 RISC-V 指令
    fn execute_instruction(&mut self, instruction: RiscVInstruction) -> Result<bool> {
        self.cycle_count += 1;

        // 检查 cycle 限制
        if self.cycle_count > self.limits.max_cycles {
            return Err(VmError::ResourceLimitExceeded("Max cycles exceeded".to_string()).into());
        }

        match instruction {
            RiscVInstruction::IType {
                opcode: 0x13,
                rd,
                funct3: 0x0,
                rs1,
                imm,
            } => {
                // ADDI rd, rs1, imm
                if rd != 0 {
                    // x0 始终为 0，不能写入
                    self.registers[rd] = self.registers[rs1].wrapping_add(imm as u64);
                }
                debug!("ADDI x{}, x{}, {} -> {}", rd, rs1, imm, self.registers[rd]);
                Ok(false) // 继续执行
            }
            RiscVInstruction::RType {
                opcode: 0x33,
                rd,
                funct3: 0x0,
                rs1,
                rs2,
                funct7: 0x00,
            } => {
                // ADD rd, rs1, rs2
                if rd != 0 {
                    self.registers[rd] = self.registers[rs1].wrapping_add(self.registers[rs2]);
                }
                debug!("ADD x{}, x{}, x{} -> {}", rd, rs1, rs2, self.registers[rd]);
                Ok(false)
            }
            RiscVInstruction::RType {
                opcode: 0x33,
                rd,
                funct3: 0x0,
                rs1,
                rs2,
                funct7: 0x20,
            } => {
                // SUB rd, rs1, rs2
                if rd != 0 {
                    self.registers[rd] = self.registers[rs1].wrapping_sub(self.registers[rs2]);
                }
                debug!("SUB x{}, x{}, x{} -> {}", rd, rs1, rs2, self.registers[rd]);
                Ok(false)
            }
            RiscVInstruction::System {
                opcode: 0x73,
                funct3: 0x0,
            } => {
                // EBREAK - 停止执行
                info!("EBREAK encountered, stopping execution");
                Ok(true) // 停止执行
            }
            _ => {
                warn!("Unimplemented instruction: {:?}", instruction);
                Ok(false)
            }
        }
    }

    /// 从字节码中读取指令
    fn fetch_instruction(&self, pc: usize) -> Result<u32> {
        if pc + 4 > self.code_cache.len() {
            return Err(VmError::ExecutionFailed("PC out of bounds".to_string()).into());
        }

        let bytes = &self.code_cache[pc..pc + 4];
        let instruction = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Ok(instruction)
    }

    /// 模拟简单的内存管理
    fn setup_memory(&mut self, input: &[u8]) -> Result<()> {
        // 将输入数据放在寄存器 a0 和 a1 中
        self.registers[10] = input.len() as u64; // a0 = 输入长度
        self.registers[11] = 0x2000_0000; // a1 = 输入数据地址（模拟）

        self.memory_size = input.len() + 4096; // 基础内存 + 输入大小

        // 检查内存限制
        if self.memory_size as u64 > self.limits.max_memory {
            return Err(VmError::ResourceLimitExceeded("Memory limit exceeded".to_string()).into());
        }

        Ok(())
    }

    /// 提取执行结果
    fn extract_result(&self) -> ExecutionResult {
        // 从 a0 寄存器获取返回值
        let return_value = self.registers[10];
        let success = return_value == 0;

        // 简化的输出生成
        let output = if success {
            return_value.to_le_bytes().to_vec()
        } else {
            vec![]
        };

        ExecutionResult {
            success,
            output,
            gas_used: self.cycle_count, // 使用 cycle 作为 gas
            cycles_used: self.cycle_count,
            error: if success {
                None
            } else {
                Some(format!("Non-zero exit code: {}", return_value))
            },
        }
    }
}

#[async_trait]
impl VmInstance for CompleteCkbVmInstance {
    async fn load_code(&mut self, code: &[u8]) -> Result<()> {
        if code.is_empty() {
            return Err(VmError::CodeLoadingFailed("Empty code".to_string()).into());
        }

        // 验证代码是否为 4 字节对齐
        if code.len() % 4 != 0 {
            return Err(
                VmError::CodeLoadingFailed("Code must be 4-byte aligned".to_string()).into(),
            );
        }

        info!("Loading {} bytes of RISC-V code", code.len());

        // 缓存代码
        self.code_cache = code.to_vec();
        self.code_loaded = true;

        // 初始化虚拟机状态
        self.init_registers();
        self.cycle_count = 0;

        debug!("Code loaded and VM initialized");
        Ok(())
    }

    async fn execute(&mut self, input: &[u8]) -> Result<ExecutionResult> {
        if !self.code_loaded {
            return Err(VmError::ExecutionFailed("No code loaded".to_string()).into());
        }

        info!("Executing RISC-V code with {} bytes input", input.len());

        // 设置内存和输入
        self.setup_memory(input)?;

        // 执行循环
        let mut pc = 0usize; // 程序计数器

        loop {
            // 获取指令
            let instruction_word = self.fetch_instruction(pc)?;
            let instruction = self.decode_instruction(instruction_word)?;

            debug!("PC: 0x{:08x}, Instruction: 0x{:08x}", pc, instruction_word);

            // 执行指令
            let should_stop = self.execute_instruction(instruction)?;

            if should_stop {
                break;
            }

            // 更新程序计数器
            pc += 4;

            // 防止无限循环
            if pc >= self.code_cache.len() {
                warn!("Reached end of code without EBREAK");
                break;
            }

            // 检查超时
            if self.cycle_count > self.limits.max_cycles {
                return Err(VmError::ResourceLimitExceeded("Execution timeout".to_string()).into());
            }
        }

        let result = self.extract_result();
        info!(
            "Execution completed: success={}, cycles={}",
            result.success, result.cycles_used
        );

        Ok(result)
    }

    async fn snapshot(&self) -> Result<VmSnapshot> {
        debug!("Creating VM snapshot");

        let snapshot_data = bincode::serialize(&VmState {
            registers: self.registers,
            cycle_count: self.cycle_count,
            memory_size: self.memory_size,
            code_loaded: self.code_loaded,
        })?;

        Ok(VmSnapshot {
            data: snapshot_data,
            vm_type: VmType::CkbVM,
        })
    }

    async fn restore(&mut self, snapshot: &VmSnapshot) -> Result<()> {
        if snapshot.vm_type != VmType::CkbVM {
            return Err(VmError::SnapshotFailed("VM type mismatch".to_string()).into());
        }

        debug!("Restoring VM from snapshot");

        let state: VmState = bincode::deserialize(&snapshot.data)?;

        self.registers = state.registers;
        self.cycle_count = state.cycle_count;
        self.memory_size = state.memory_size;
        self.code_loaded = state.code_loaded;

        debug!("VM state restored successfully");
        Ok(())
    }

    fn vm_type(&self) -> VmType {
        VmType::CkbVM
    }

    fn set_limits(&mut self, limits: ExecutionLimits) {
        debug!("Setting execution limits: {:?}", limits);
        self.limits = limits;
    }
}

/// RISC-V 指令类型
#[derive(Debug, Clone)]
enum RiscVInstruction {
    IType {
        opcode: u32,
        rd: usize,
        funct3: u32,
        rs1: usize,
        imm: i32,
    },
    RType {
        opcode: u32,
        rd: usize,
        funct3: u32,
        rs1: usize,
        rs2: usize,
        funct7: u32,
    },
    System {
        opcode: u32,
        funct3: u32,
    },
}

/// VM 状态（用于快照）
#[derive(serde::Serialize, serde::Deserialize)]
struct VmState {
    registers: [u64; 32],
    cycle_count: u64,
    memory_size: usize,
    code_loaded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_ckb_vm() -> Result<()> {
        let mut vm = CompleteCkbVmInstance::new()?;

        // RISC-V 代码：计算 5 + 3 = 8
        let code = vec![
            0x13, 0x05, 0x50, 0x00, // addi a0, zero, 5
            0x13, 0x06, 0x30, 0x00, // addi a1, zero, 3
            0x33, 0x05, 0xB5, 0x00, // add a0, a0, a1
            0x73, 0x00, 0x10, 0x00, // ebreak
        ];

        vm.load_code(&code).await?;
        let result = vm.execute(&[]).await?;

        assert!(result.success);
        assert_eq!(result.cycles_used, 4); // 4 条指令

        Ok(())
    }

    #[tokio::test]
    async fn test_vm_snapshot() -> Result<()> {
        let mut vm = CompleteCkbVmInstance::new()?;
        let code = vec![0x73, 0x00, 0x10, 0x00]; // ebreak

        vm.load_code(&code).await?;

        let snapshot = vm.snapshot().await?;
        assert!(!snapshot.data.is_empty());

        vm.restore(&snapshot).await?;

        Ok(())
    }
}
