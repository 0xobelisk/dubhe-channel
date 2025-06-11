//! VM Runtime Traits

use anyhow::Result;
use async_trait::async_trait;

use crate::types::*;

/// VM 实例 trait  
#[async_trait]
pub trait VmInstance {
    /// 加载代码到 VM
    async fn load_code(&mut self, code: &[u8]) -> Result<()>;
    
    /// 执行代码
    async fn execute(&mut self, input: &[u8]) -> Result<ExecutionResult>;
    
    /// 创建快照
    async fn snapshot(&self) -> Result<VmSnapshot>;
    
    /// 从快照恢复
    async fn restore(&mut self, snapshot: &VmSnapshot) -> Result<()>;
    
    /// 获取 VM 类型
    fn vm_type(&self) -> VmType;
    
    /// 设置执行限制
    fn set_limits(&mut self, limits: ExecutionLimits);
} 