//! 共识类型定义

#[derive(Debug, Clone)]
pub struct Block {
    pub hash: String,
    pub height: u64,
}
