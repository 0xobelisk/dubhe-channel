//! Sui 适配器
//!
//! 基于 Sui JSON-RPC 实现的 Sui 轻节点客户端

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::sui_types::*;
use crate::traits::ChainAdapter;
use crate::types::*;

/// Sui 适配器
pub struct SuiAdapter {
    config: SuiConfig,
    client: Client,
}

impl SuiAdapter {
    pub async fn new(config: SuiConfig) -> Result<Self> {
        let client = Client::new();

        info!(
            "Sui adapter initialized for {} network: {}",
            format!("{:?}", config.network_type),
            config.rpc_url
        );

        Ok(Self { config, client })
    }

    /// 获取网络的完整节点 URL
    pub fn get_fullnode_url(network_type: &SuiNetworkType) -> String {
        match network_type {
            SuiNetworkType::Mainnet => "https://fullnode.mainnet.sui.io".to_string(),
            SuiNetworkType::Testnet => "https://fullnode.testnet.sui.io".to_string(),
            SuiNetworkType::Devnet => "https://fullnode.devnet.sui.io".to_string(),
            SuiNetworkType::Localnet => "http://127.0.0.1:9000".to_string(),
        }
    }

    /// 获取标准化的 Move 模块（类似于 TypeScript 版本中的 getNormalizedMoveModulesByPackage）
    pub async fn get_normalized_move_modules_by_package(&self, package_id: &str) -> Result<Value> {
        info!(
            "Getting normalized Move modules for package: {}",
            package_id
        );

        let result = self
            .call_rpc("sui_getNormalizedMoveModulesByPackage", json!([package_id]))
            .await?;

        debug!("Normalized Move modules: {}", result);
        Ok(result)
    }

    /// 批量加载配置中的所有包的 metadata
    pub async fn load_all_package_metadata(&self) -> Result<Vec<(String, Value)>> {
        let mut results = Vec::new();

        for package_id in &self.config.package_ids {
            info!("Loading metadata for package: {}", package_id);

            match self
                .get_normalized_move_modules_by_package(package_id)
                .await
            {
                Ok(metadata) => {
                    results.push((package_id.clone(), metadata));
                    info!(
                        "✅ Successfully loaded metadata for package: {}",
                        package_id
                    );
                }
                Err(e) => {
                    error!(
                        "❌ Failed to load metadata for package {}: {}",
                        package_id, e
                    );
                    // 继续处理其他包，不中断整个流程
                }
            }
        }

        info!(
            "Loaded metadata for {}/{} packages",
            results.len(),
            self.config.package_ids.len()
        );
        Ok(results)
    }

    /// 调用 Sui JSON-RPC 方法
    async fn call_rpc(&self, method: &str, params: Value) -> Result<Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let response = self
            .client
            .post(&self.config.rpc_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let response_json: Value = response.json().await?;

        if let Some(error) = response_json.get("error") {
            return Err(anyhow::anyhow!("Sui RPC error: {}", error));
        }

        Ok(response_json["result"].clone())
    }
}

#[async_trait]
impl ChainAdapter for SuiAdapter {
    async fn get_contract_meta(&self, address: &str) -> Result<ContractMeta> {
        info!("Getting Sui package metadata for: {}", address);

        // 获取 Sui 包信息
        let package_info = self
            .call_rpc(
                "sui_getObject",
                json!([
                    address,
                    {
                        "showType": true,
                        "showOwner": true,
                        "showPreviousTransaction": true,
                        "showDisplay": false,
                        "showContent": true,
                        "showBcs": true,
                        "showStorageRebate": true
                    }
                ]),
            )
            .await?;

        debug!("Sui package info: {}", package_info);

        // 解析包内容
        let content = package_info["data"]["content"].clone();
        let bytecode = if let Some(bcs) = package_info["data"]["bcs"].as_str() {
            hex::decode(bcs.strip_prefix("0x").unwrap_or(bcs)).unwrap_or_else(|_| vec![])
        } else {
            vec![]
        };

        // 获取创建者信息
        let creator = package_info["data"]["owner"]
            .as_str()
            .map(|s| s.to_string());

        // 获取创建时间
        let created_at = chrono::Utc::now().timestamp() as u64;

        Ok(ContractMeta {
            address: address.to_string(),
            chain_type: ChainType::Sui,
            contract_type: ContractType::Move,
            bytecode,
            abi: Some(content.to_string()),
            source_code: None,
            compiler_version: Some("move".to_string()),
            created_at,
            creator,
        })
    }

    async fn get_transaction_receipt(&self, tx_hash: &str) -> Result<TransactionReceipt> {
        info!("Getting Sui transaction: {}", tx_hash);

        let tx_info = self
            .call_rpc(
                "sui_getTransactionBlock",
                json!([
                    tx_hash,
                    {
                        "showInput": true,
                        "showRawInput": false,
                        "showEffects": true,
                        "showEvents": true,
                        "showObjectChanges": true,
                        "showBalanceChanges": true
                    }
                ]),
            )
            .await?;

        debug!("Sui transaction info: {}", tx_info);

        // 解析交易信息
        let digest = tx_info["digest"].as_str().unwrap_or(tx_hash).to_string();
        let effects = &tx_info["effects"];

        let status = if effects["status"]["status"].as_str() == Some("success") {
            TransactionStatus::Success
        } else {
            TransactionStatus::Failed
        };

        // 获取 gas 使用量
        let gas_used = effects["gasUsed"]["computationCost"].as_u64().unwrap_or(0);

        // 获取发送者
        let sender = tx_info["transaction"]["data"]["sender"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // 解析事件日志
        let mut logs = vec![];
        if let Some(events) = tx_info["events"].as_array() {
            for event in events {
                logs.push(EventLog {
                    address: event["packageId"].as_str().unwrap_or("").to_string(),
                    topics: vec![event["type"].as_str().unwrap_or("").to_string()],
                    data: event["parsedJson"].to_string(),
                });
            }
        }

        Ok(TransactionReceipt {
            tx_hash: digest,
            block_hash: effects["transactionDigest"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            block_number: effects["checkpoint"].as_u64().unwrap_or(0),
            transaction_index: 0, // Sui 不使用传统的交易索引
            from: sender,
            to: None, // Sui 交易可能有多个接收者，这里简化处理
            gas_used,
            status,
            logs,
            contract_address: None,
        })
    }

    async fn get_balance(&self, address: &str) -> Result<u64> {
        info!("Getting Sui balance for: {}", address);

        let balance_info = self
            .call_rpc("suix_getBalance", json!([address, "0x2::sui::SUI"]))
            .await?;

        let balance = balance_info["totalBalance"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        debug!("Sui balance for {}: {}", address, balance);
        Ok(balance)
    }

    async fn get_nonce(&self, address: &str) -> Result<u64> {
        // Sui 使用序列号概念，获取最新的序列号
        info!("Getting Sui sequence number for: {}", address);

        let objects = self
            .call_rpc(
                "suix_getOwnedObjects",
                json!([
                    address,
                    {
                        "filter": {
                            "StructType": "0x2::coin::Coin<0x2::sui::SUI>"
                        },
                        "options": {
                            "showType": true,
                            "showOwner": true,
                            "showPreviousTransaction": true
                        }
                    },
                    null,
                    1
                ]),
            )
            .await?;

        // 从拥有的对象中推断序列号
        // 这是一个简化的实现，实际应该查询更详细的交易历史
        let sequence = objects["data"]
            .as_array()
            .map(|arr| arr.len() as u64)
            .unwrap_or(0);

        debug!("Sui sequence number for {}: {}", address, sequence);
        Ok(sequence)
    }

    async fn get_block_number(&self) -> Result<u64> {
        info!("Getting latest Sui checkpoint");

        let checkpoint_info = self
            .call_rpc("sui_getLatestCheckpointSequenceNumber", json!([]))
            .await?;

        let checkpoint = checkpoint_info
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        debug!("Latest Sui checkpoint: {}", checkpoint);
        Ok(checkpoint)
    }

    async fn subscribe_new_blocks(&self) -> Result<mpsc::Receiver<String>> {
        info!("Starting Sui checkpoint subscription");
        let (tx, rx) = mpsc::channel(1000);

        // 启动轮询任务来模拟订阅
        let config = self.config.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            let mut last_checkpoint = 0u64;
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));

            loop {
                interval.tick().await;

                match Self::get_latest_checkpoint(&client, &config.rpc_url).await {
                    Ok(current_checkpoint) => {
                        if current_checkpoint > last_checkpoint {
                            for checkpoint in (last_checkpoint + 1)..=current_checkpoint {
                                if tx.send(checkpoint.to_string()).await.is_err() {
                                    warn!("Sui checkpoint subscription channel closed");
                                    return;
                                }
                            }
                            last_checkpoint = current_checkpoint;
                        }
                    }
                    Err(e) => {
                        error!("Failed to get latest Sui checkpoint: {}", e);
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn subscribe_new_transactions(&self) -> Result<mpsc::Receiver<String>> {
        info!("Starting Sui transaction subscription");
        let (tx, rx) = mpsc::channel(1000);

        // 启动轮询任务来获取新交易
        let config = self.config.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            let mut last_checkpoint = 0u64;
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

            loop {
                interval.tick().await;

                match Self::get_latest_checkpoint(&client, &config.rpc_url).await {
                    Ok(current_checkpoint) => {
                        if current_checkpoint > last_checkpoint {
                            // 获取新检查点中的交易
                            if let Ok(transactions) = Self::get_checkpoint_transactions(
                                &client,
                                &config.rpc_url,
                                current_checkpoint,
                            )
                            .await
                            {
                                for tx_hash in transactions {
                                    if tx.send(tx_hash).await.is_err() {
                                        warn!("Sui transaction subscription channel closed");
                                        return;
                                    }
                                }
                            }
                            last_checkpoint = current_checkpoint;
                        }
                    }
                    Err(e) => {
                        error!("Failed to get Sui transactions: {}", e);
                    }
                }
            }
        });

        Ok(rx)
    }
}

impl SuiAdapter {
    /// 获取最新检查点号
    async fn get_latest_checkpoint(client: &Client, rpc_url: &str) -> Result<u64> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sui_getLatestCheckpointSequenceNumber",
            "params": []
        });

        let response = client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let response_json: Value = response.json().await?;

        response_json["result"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| anyhow::anyhow!("Failed to parse checkpoint number"))
    }

    /// 获取检查点中的交易列表
    async fn get_checkpoint_transactions(
        client: &Client,
        rpc_url: &str,
        checkpoint: u64,
    ) -> Result<Vec<String>> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sui_getCheckpoint",
            "params": [checkpoint.to_string()]
        });

        let response = client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let response_json: Value = response.json().await?;

        let transactions = response_json["result"]["transactions"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|tx| tx.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(transactions)
    }

    /// 获取对象的完整状态数据
    pub async fn get_object_data(&self, object_id: &str) -> Result<Value> {
        info!("Getting complete object data for: {}", object_id);

        let result = self
            .call_rpc(
                "sui_getObject",
                json!([
                    object_id,
                    {
                        "showType": true,
                        "showOwner": true,
                        "showPreviousTransaction": true,
                        "showDisplay": false,
                        "showContent": true,
                        "showBcs": true,
                        "showStorageRebate": true
                    }
                ]),
            )
            .await?;

        debug!("Complete object data for {}: {}", object_id, result);
        Ok(result)
    }

    /// 获取对象的原始 BCS 数据
    pub async fn get_object_bcs_data(&self, object_id: &str) -> Result<Vec<u8>> {
        info!("Getting BCS data for object: {}", object_id);

        let object_data = self.get_object_data(object_id).await?;

        if let Some(bcs_str) = object_data["data"]["bcs"].as_str() {
            let hex_str = bcs_str.strip_prefix("0x").unwrap_or(bcs_str);

            // 手动解码 hex 字符串
            let mut bcs_data = Vec::new();
            let chars: Vec<char> = hex_str.chars().collect();
            for chunk in chars.chunks(2) {
                if chunk.len() == 2 {
                    let hex_byte = format!("{}{}", chunk[0], chunk[1]);
                    if let Ok(byte) = u8::from_str_radix(&hex_byte, 16) {
                        bcs_data.push(byte);
                    } else {
                        return Err(anyhow::anyhow!("Invalid hex character in BCS data"));
                    }
                }
            }

            info!(
                "Retrieved {} bytes of BCS data for object {}",
                bcs_data.len(),
                object_id
            );
            Ok(bcs_data)
        } else {
            // 对于没有 BCS 数据的对象，我们使用对象内容的 JSON 序列化作为替代
            info!(
                "No BCS data found, using JSON content as fallback for object {}",
                object_id
            );
            let content_str = object_data["data"]["content"].to_string();
            Ok(content_str.as_bytes().to_vec())
        }
    }

    /// 执行 Move 函数调用 (干跑)
    pub async fn dry_run_transaction(&self, tx_data: &Value) -> Result<Value> {
        info!("Performing dry run transaction");

        let result = self
            .call_rpc("sui_dryRunTransactionBlock", json!([tx_data]))
            .await?;

        debug!("Dry run result: {}", result);
        Ok(result)
    }

    /// 执行实际的 Move 函数调用交易
    pub async fn execute_transaction(&self, tx_data: &Value, signature: &str) -> Result<String> {
        info!("Executing Move transaction");

        let result = self
            .call_rpc(
                "sui_executeTransactionBlock",
                json!([
                    tx_data,
                    [signature],
                    {
                        "showInput": true,
                        "showRawInput": false,
                        "showEffects": true,
                        "showEvents": true,
                        "showObjectChanges": true,
                        "showBalanceChanges": true
                    }
                ]),
            )
            .await?;

        let tx_hash = result["digest"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No transaction hash returned"))?
            .to_string();

        info!("Transaction executed successfully: {}", tx_hash);
        Ok(tx_hash)
    }

    /// 构建 Move 函数调用交易
    pub async fn build_move_call_transaction(
        &self,
        sender: &str,
        package_id: &str,
        module: &str,
        function: &str,
        type_arguments: Vec<String>,
        arguments: Vec<Value>,
        gas_budget: u64,
    ) -> Result<Value> {
        info!(
            "Building Move call: {}::{}::{} for sender {}",
            package_id, module, function, sender
        );

        // 构建 PTB (Programmable Transaction Block)
        let result = self
            .call_rpc(
                "unsafe_moveCall",
                json!([
                    sender,
                    package_id,
                    module,
                    function,
                    type_arguments,
                    arguments,
                    null, // gas_coin
                    gas_budget.to_string()
                ]),
            )
            .await?;

        debug!("Built transaction: {}", result);
        Ok(result)
    }
}
