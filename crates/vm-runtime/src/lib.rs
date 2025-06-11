//! Dubhe Channel VM Runtime
//!
//! RISC-V VM 抽象层：PolkaVM / CKB-VM / Cartesi

pub mod ckb;
pub mod ckb_complete;
pub mod error;
pub mod polka;
pub mod traits;
pub mod types;

pub use error::*;
pub use traits::*;
pub use types::*;

use anyhow::Result;
use std::sync::Arc;

/// VM 实例管理器
pub struct VmManager {
    default_vm: VmType,
}

impl VmManager {
    pub fn new(default_vm: VmType) -> Self {
        Self { default_vm }
    }

    /// 创建 VM 实例
    pub fn create_instance(
        &self,
        vm_type: Option<VmType>,
    ) -> Result<Box<dyn VmInstance + Send + Sync>> {
        let vm_type = vm_type.unwrap_or(self.default_vm);

        match vm_type {
            #[cfg(feature = "polkavm")]
            VmType::PolkaVM => Ok(Box::new(polka::PolkaVmInstance::new()?)),

            #[cfg(feature = "ckb-vm")]
            VmType::CkbVM => Ok(Box::new(ckb::CkbVmInstance::new()?)),

            _ => Err(anyhow::anyhow!("Unsupported VM type: {:?}", vm_type)),
        }
    }
}
