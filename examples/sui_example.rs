//! Sui Adapter Usage Example
//!
//! Demonstrates how to use Dubhe Channel's Sui adapter

use anyhow::Result;
use dubhe_adapter::{
    sui::SuiAdapter,
    traits::ChainAdapter,
    types::{SuiConfig, SuiNetworkType},
};
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("🚀 Starting Sui adapter example...");

    // Configure Sui adapter
    let config = SuiConfig {
        rpc_url: "https://fullnode.mainnet.sui.io".to_string(),
        ws_url: None,
        network_type: SuiNetworkType::Mainnet,
        package_ids: vec![
            "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
            "0x0000000000000000000000000000000000000000000000000000000000000002".to_string(),
        ],
    };

    // Create Sui adapter
    let adapter = SuiAdapter::new(config).await?;
    info!("✅ Sui adapter initialized");

    // Example 1: Get latest checkpoint
    match adapter.get_block_number().await {
        Ok(checkpoint) => info!("📦 Latest Sui checkpoint: {}", checkpoint),
        Err(e) => info!("❌ Failed to get checkpoint: {}", e),
    }

    // Example 2: Query address balance (using Sui Foundation address)
    let example_address = "0x0000000000000000000000000000000000000000000000000000000000000002";
    match adapter.get_balance(example_address).await {
        Ok(balance) => info!("💰 Balance for {}: {} MIST", example_address, balance),
        Err(e) => info!("❌ Failed to get balance: {}", e),
    }

    // Example 3: Get package info (Sui Framework)
    let sui_framework = "0x0000000000000000000000000000000000000000000000000000000000000002";
    match adapter.get_contract_meta(sui_framework).await {
        Ok(meta) => {
            info!("📦 Package info for Sui Framework:");
            info!("  - Address: {}", meta.address);
            info!("  - Chain Type: {:?}", meta.chain_type);
            info!("  - Contract Type: {:?}", meta.contract_type);
            info!("  - Bytecode Length: {} bytes", meta.bytecode.len());
            if let Some(abi) = &meta.abi {
                info!("  - ABI Length: {} chars", abi.len());
            }
        }
        Err(e) => info!("❌ Failed to get package info: {}", e),
    }

    // Example 4: Subscribe to new checkpoints
    info!("🔔 Starting checkpoint subscription...");
    match adapter.subscribe_new_blocks().await {
        Ok(mut receiver) => {
            info!("✅ Checkpoint subscription started");

            // Listen for the first 3 new checkpoints
            let mut count = 0;
            while count < 3 {
                if let Some(checkpoint) = receiver.recv().await {
                    info!("🆕 New checkpoint: {}", checkpoint);
                    count += 1;
                } else {
                    break;
                }
            }

            info!("✅ Checkpoint subscription completed");
        }
        Err(e) => info!("❌ Failed to subscribe to checkpoints: {}", e),
    }

    // Example 5: Subscribe to new transactions
    info!("🔔 Starting transaction subscription...");
    match adapter.subscribe_new_transactions().await {
        Ok(mut receiver) => {
            info!("✅ Transaction subscription started");

            // Listen for the first 5 new transactions
            let mut count = 0;
            while count < 5 {
                if let Some(tx_hash) = receiver.recv().await {
                    info!("🆕 New transaction: {}", tx_hash);

                    // Get transaction details
                    match adapter.get_transaction_receipt(&tx_hash).await {
                        Ok(receipt) => {
                            info!("   📄 Transaction details:");
                            info!("     - Status: {:?}", receipt.status);
                            info!("     - Gas Used: {}", receipt.gas_used);
                            info!("     - From: {}", receipt.from);
                            info!("     - Events: {}", receipt.logs.len());
                        }
                        Err(e) => info!("   ❌ Failed to get transaction details: {}", e),
                    }

                    count += 1;
                } else {
                    break;
                }
            }

            info!("✅ Transaction subscription completed");
        }
        Err(e) => info!("❌ Failed to subscribe to transactions: {}", e),
    }

    info!("🎉 Sui adapter example completed!");
    Ok(())
}
