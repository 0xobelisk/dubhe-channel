//! State 类型定义

#[derive(Debug, Clone)]
pub struct StateData {
    pub key: String,
    pub value: Vec<u8>,
}
