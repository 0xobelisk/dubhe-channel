//! Sui 专用类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sui 对象引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiObjectRef {
    pub object_id: String,
    pub version: u64,
    pub digest: String,
}

/// Sui 对象数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiObjectData {
    pub object_id: String,
    pub version: u64,
    pub digest: String,
    pub object_type: Option<String>,
    pub owner: Option<SuiOwner>,
    pub previous_transaction: Option<String>,
    pub storage_rebate: Option<u64>,
    pub content: Option<SuiParsedData>,
    pub bcs: Option<String>,
}

/// Sui 对象拥有者
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SuiOwner {
    AddressOwner { address: String },
    ObjectOwner { object_id: String },
    Shared { initial_shared_version: u64 },
    Immutable,
}

/// Sui 解析数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "dataType")]
pub enum SuiParsedData {
    MoveObject(SuiMoveObject),
    Package(SuiMovePackage),
}

/// Sui Move 对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiMoveObject {
    pub object_type: String,
    pub has_public_transfer: bool,
    pub fields: HashMap<String, serde_json::Value>,
}

/// Sui Move 包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiMovePackage {
    pub disassembled: HashMap<String, serde_json::Value>,
}

/// Sui 交易块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiTransactionBlock {
    pub digest: String,
    pub transaction: Option<SuiTransaction>,
    pub effects: Option<SuiTransactionBlockEffects>,
    pub events: Option<Vec<SuiEvent>>,
    pub object_changes: Option<Vec<SuiObjectChange>>,
    pub balance_changes: Option<Vec<SuiBalanceChange>>,
    pub timestamp_ms: Option<u64>,
    pub checkpoint: Option<u64>,
}

/// Sui 交易
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiTransaction {
    pub data: SuiTransactionData,
    pub tx_signatures: Vec<String>,
}

/// Sui 交易数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiTransactionData {
    pub message_version: String,
    pub transaction: SuiTransactionKind,
    pub sender: String,
    pub gas_data: SuiGasData,
}

/// Sui 交易类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SuiTransactionKind {
    ProgrammableTransaction {
        inputs: Vec<SuiCallArg>,
        commands: Vec<SuiCommand>,
    },
}

/// Sui 调用参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SuiCallArg {
    Pure { bytes: String },
    Object(SuiObjectArg),
}

/// Sui 对象参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "Object")]
pub enum SuiObjectArg {
    ImmOrOwnedObject(SuiObjectRef),
    SharedObject {
        object_id: String,
        initial_shared_version: u64,
        mutable: bool,
    },
    Receiving(SuiObjectRef),
}

/// Sui 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum SuiCommand {
    MoveCall {
        package: String,
        module: String,
        function: String,
        type_arguments: Vec<String>,
        arguments: Vec<SuiArgument>,
    },
    TransferObjects {
        objects: Vec<SuiArgument>,
        address: SuiArgument,
    },
    SplitCoins {
        coin: SuiArgument,
        amounts: Vec<SuiArgument>,
    },
    MergeCoins {
        destination: SuiArgument,
        sources: Vec<SuiArgument>,
    },
    Publish {
        modules: Vec<String>,
        dependencies: Vec<String>,
    },
    MakeMoveVec {
        move_type: Option<String>,
        objects: Vec<SuiArgument>,
    },
    Upgrade {
        modules: Vec<String>,
        dependencies: Vec<String>,
        package_id: String,
        ticket: SuiArgument,
    },
}

/// Sui 参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SuiArgument {
    GasCoin,
    Input { input: u16 },
    Result { cmd: u16 },
    NestedResult { cmd: u16, idx: u16 },
}

/// Sui Gas 数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiGasData {
    pub payment: Vec<SuiObjectRef>,
    pub owner: String,
    pub price: u64,
    pub budget: u64,
}

/// Sui 交易执行效果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiTransactionBlockEffects {
    pub message_version: String,
    pub status: SuiExecutionStatus,
    pub executed_epoch: u64,
    pub gas_used: SuiGasUsage,
    pub modified_at_versions: Vec<(String, u64)>,
    pub shared_objects: Vec<SuiObjectRef>,
    pub transaction_digest: String,
    pub created: Vec<SuiOwnedObjectRef>,
    pub mutated: Vec<SuiOwnedObjectRef>,
    pub unwrapped: Vec<SuiOwnedObjectRef>,
    pub deleted: Vec<SuiObjectRef>,
    pub unwrapped_then_deleted: Vec<SuiObjectRef>,
    pub wrapped: Vec<SuiObjectRef>,
    pub gas_object: SuiOwnedObjectRef,
    pub events_digest: Option<String>,
    pub dependencies: Vec<String>,
}

/// Sui 执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum SuiExecutionStatus {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failure")]
    Failure { error: String },
}

/// Sui Gas 使用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiGasUsage {
    pub computation_cost: u64,
    pub storage_cost: u64,
    pub storage_rebate: u64,
    pub non_refundable_storage_fee: u64,
}

/// Sui 拥有对象引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiOwnedObjectRef {
    pub reference: SuiObjectRef,
    pub owner: SuiOwner,
}

/// Sui 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiEvent {
    pub id: SuiEventId,
    pub package_id: String,
    pub transaction_module: String,
    pub sender: String,
    pub event_type: String,
    pub parsed_json: serde_json::Value,
    pub bcs: String,
    pub timestamp_ms: Option<u64>,
}

/// Sui 事件 ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiEventId {
    pub tx_digest: String,
    pub event_seq: u64,
}

/// Sui 对象变化
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SuiObjectChange {
    Created {
        sender: String,
        owner: SuiOwner,
        object_type: String,
        object_id: String,
        version: u64,
        digest: String,
    },
    Mutated {
        sender: String,
        owner: SuiOwner,
        object_type: String,
        object_id: String,
        version: u64,
        previous_version: u64,
        digest: String,
    },
    Deleted {
        sender: String,
        object_type: String,
        object_id: String,
        version: u64,
    },
    Wrapped {
        sender: String,
        object_type: String,
        object_id: String,
        version: u64,
    },
    Published {
        package_id: String,
        version: u64,
        digest: String,
        modules: Vec<String>,
    },
    Transferred {
        sender: String,
        recipient: SuiOwner,
        object_type: String,
        object_id: String,
        version: u64,
        digest: String,
    },
}

/// Sui 余额变化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiBalanceChange {
    pub owner: SuiOwner,
    pub coin_type: String,
    pub amount: String,
}

/// Sui 检查点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiCheckpoint {
    pub epoch: u64,
    pub sequence_number: u64,
    pub digest: String,
    pub network_total_transactions: u64,
    pub previous_digest: Option<String>,
    pub epoch_rolling_gas_cost_summary: SuiGasCostSummary,
    pub timestamp_ms: u64,
    pub transactions: Vec<String>,
    pub checkpoint_commitments: Vec<String>,
    pub validator_signature: String,
}

/// Sui Gas 费用摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiGasCostSummary {
    pub computation_cost: u64,
    pub storage_cost: u64,
    pub storage_rebate: u64,
    pub non_refundable_storage_fee: u64,
}

/// Sui 余额信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiBalance {
    pub coin_type: String,
    pub coin_object_count: u64,
    pub total_balance: String,
    pub locked_balance: HashMap<String, String>,
}

/// Sui 拥有对象过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "filter")]
pub enum SuiObjectDataFilter {
    MatchAll(Vec<SuiObjectDataFilter>),
    MatchAny(Vec<SuiObjectDataFilter>),
    MatchNone(Vec<SuiObjectDataFilter>),
    StructType(String),
    AddressOwner(String),
    ObjectOwner(String),
    ObjectId(String),
    ObjectIds(Vec<String>),
    Version(u64),
}

/// Sui 对象数据选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiObjectDataOptions {
    pub show_type: Option<bool>,
    pub show_owner: Option<bool>,
    pub show_previous_transaction: Option<bool>,
    pub show_display: Option<bool>,
    pub show_content: Option<bool>,
    pub show_bcs: Option<bool>,
    pub show_storage_rebate: Option<bool>,
}
