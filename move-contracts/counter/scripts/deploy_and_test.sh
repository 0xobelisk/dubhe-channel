#!/bin/bash

# Counter 合约部署和测试脚本
# 用于 Dubhe Channel Phase 1 演示

set -e

echo "🚀 Dubhe Channel Counter 合约部署脚本"
echo "========================================="

# 检查依赖
command -v sui >/dev/null 2>&1 || { echo "❌ 需要安装 Sui CLI" >&2; exit 1; }

# 配置参数
NETWORK="testnet"
GAS_BUDGET=20000000  # 0.02 SUI

echo "📋 配置信息:"
echo "   网络: $NETWORK"
echo "   Gas 预算: $GAS_BUDGET"
echo ""

# 1. 构建合约
echo "🔨 构建 Counter 合约..."
cd "$(dirname "$0")/.."
sui move build

if [ $? -eq 0 ]; then
    echo "✅ 合约构建成功"
else
    echo "❌ 合约构建失败"
    exit 1
fi

# 2. 部署合约
echo ""
echo "📦 部署 Counter 合约到 $NETWORK..."

DEPLOY_RESULT=$(sui client publish --gas-budget $GAS_BUDGET --json)

if [ $? -eq 0 ]; then
    echo "✅ 合约部署成功"
    
    # 解析部署结果
    PACKAGE_ID=$(echo "$DEPLOY_RESULT" | jq -r '.objectChanges[] | select(.type == "published") | .packageId')
    echo "📦 Package ID: $PACKAGE_ID"
    
    # 保存部署信息
    echo "$DEPLOY_RESULT" > deployment.json
    echo "Package ID: $PACKAGE_ID" > package_id.txt
    
else
    echo "❌ 合约部署失败"
    exit 1
fi

# 3. 创建 Counter 对象
echo ""
echo "🎯 创建 Counter 共享对象..."

CREATE_RESULT=$(sui client call \
    --package $PACKAGE_ID \
    --module counter \
    --function create \
    --gas-budget $GAS_BUDGET \
    --json)

if [ $? -eq 0 ]; then
    echo "✅ Counter 对象创建成功"
    
    # 解析创建结果
    COUNTER_ID=$(echo "$CREATE_RESULT" | jq -r '.objectChanges[] | select(.type == "created") | .objectId')
    echo "🎯 Counter Object ID: $COUNTER_ID"
    
    # 保存对象信息
    echo "$CREATE_RESULT" > counter_creation.json
    echo "Counter ID: $COUNTER_ID" > counter_id.txt
    
else
    echo "❌ Counter 对象创建失败"
    exit 1
fi

# 4. 测试基本功能
echo ""
echo "🧪 测试 Counter 基本功能..."

# 测试递增
echo "   测试递增功能..."
INCREMENT_RESULT=$(sui client call \
    --package $PACKAGE_ID \
    --module counter \
    --function increment \
    --args $COUNTER_ID \
    --gas-budget $GAS_BUDGET \
    --json)

if [ $? -eq 0 ]; then
    echo "   ✅ 递增测试成功"
else
    echo "   ❌ 递增测试失败"
fi

# 查询当前值
echo "   查询当前计数值..."
VALUE_RESULT=$(sui client call \
    --package $PACKAGE_ID \
    --module counter \
    --function value \
    --args $COUNTER_ID \
    --gas-budget $GAS_BUDGET \
    --json)

if [ $? -eq 0 ]; then
    echo "   ✅ 值查询成功"
    # 注意: 实际值需要从执行结果中解析
else
    echo "   ❌ 值查询失败"
fi

# 5. 生成 Dubhe Channel 配置
echo ""
echo "⚙️  生成 Dubhe Channel 配置..."

cat > ../../../dubhe_counter_config.toml << EOF
# Dubhe Channel Counter 合约配置
# 由 deploy_and_test.sh 自动生成

[counter_demo]
package_id = "$PACKAGE_ID"
counter_object_id = "$COUNTER_ID"
network = "$NETWORK"

# 支持的函数调用
[counter_demo.functions]
increment = "counter::increment"
reset = "counter::reset"  
set_value = "counter::set_value"
value = "counter::value"
owner = "counter::owner"

# Gas 预算配置
[counter_demo.gas_budgets]
increment = 10000
reset = 12000
set_value = 10000
value = 8000
owner = 8000

# Phase 1 演示配置
[phase1_demo]
max_concurrent_sessions = 10
batch_size = 5
test_iterations = 20
EOF

echo "✅ 配置文件已生成: ../../../dubhe_counter_config.toml"

# 6. 创建 Rust 演示代码
echo ""
echo "🦀 生成 Rust 演示代码..."

cat > ../../../examples/counter_demo_generated.rs << EOF
/// 自动生成的 Counter 合约演示代码
/// 基于实际部署的合约: $PACKAGE_ID

use anyhow::Result;
use serde_json::json;

// 实际部署的合约信息
pub const PACKAGE_ID: &str = "$PACKAGE_ID";
pub const COUNTER_OBJECT_ID: &str = "$COUNTER_ID";

/// Counter 合约函数调用示例
pub mod counter_calls {
    use super::*;
    use dubhe_node::offchain_execution::ExecutionRequest;

    pub fn increment_request() -> ExecutionRequest {
        ExecutionRequest {
            session_id: "counter_increment".to_string(),
            package_id: PACKAGE_ID.to_string(),
            function_name: "counter::increment".to_string(),
            arguments: vec![json!(COUNTER_OBJECT_ID)],
            shared_objects: vec![COUNTER_OBJECT_ID.to_string()],
            gas_budget: 10000,
        }
    }

    pub fn set_value_request(value: u64) -> ExecutionRequest {
        ExecutionRequest {
            session_id: format!("counter_set_value_{}", value),
            package_id: PACKAGE_ID.to_string(),
            function_name: "counter::set_value".to_string(),
            arguments: vec![json!(COUNTER_OBJECT_ID), json!(value)],
            shared_objects: vec![COUNTER_OBJECT_ID.to_string()],
            gas_budget: 10000,
        }
    }

    pub fn reset_request() -> ExecutionRequest {
        ExecutionRequest {
            session_id: "counter_reset".to_string(),
            package_id: PACKAGE_ID.to_string(),
            function_name: "counter::reset".to_string(),
            arguments: vec![json!(COUNTER_OBJECT_ID)],
            shared_objects: vec![COUNTER_OBJECT_ID.to_string()],
            gas_budget: 12000,
        }
    }

    pub fn value_request() -> ExecutionRequest {
        ExecutionRequest {
            session_id: "counter_value".to_string(),
            package_id: PACKAGE_ID.to_string(),
            function_name: "counter::value".to_string(),
            arguments: vec![json!(COUNTER_OBJECT_ID)],
            shared_objects: vec![COUNTER_OBJECT_ID.to_string()],
            gas_budget: 8000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::counter_calls::*;

    #[test]
    fn test_contract_constants() {
        assert!(!PACKAGE_ID.is_empty());
        assert!(!COUNTER_OBJECT_ID.is_empty());
        println!("Package ID: {}", PACKAGE_ID);
        println!("Counter Object ID: {}", COUNTER_OBJECT_ID);
    }

    #[test]
    fn test_request_generation() {
        let increment_req = increment_request();
        assert_eq!(increment_req.function_name, "counter::increment");
        assert_eq!(increment_req.gas_budget, 10000);

        let set_value_req = set_value_request(42);
        assert_eq!(set_value_req.function_name, "counter::set_value");
        assert!(set_value_req.arguments.len() == 2);

        let reset_req = reset_request();
        assert_eq!(reset_req.function_name, "counter::reset");

        let value_req = value_request();
        assert_eq!(value_req.function_name, "counter::value");
    }
}
EOF

echo "✅ Rust 演示代码已生成: ../../../examples/counter_demo_generated.rs"

# 7. 总结
echo ""
echo "🎉 部署完成总结"
echo "================"
echo "📦 Package ID: $PACKAGE_ID"
echo "🎯 Counter Object ID: $COUNTER_ID"
echo "🌐 网络: $NETWORK"
echo ""
echo "📁 生成的文件:"
echo "   - deployment.json (完整部署信息)"
echo "   - package_id.txt (包ID)"
echo "   - counter_id.txt (对象ID)"
echo "   - ../../../dubhe_counter_config.toml (Dubhe配置)"
echo "   - ../../../examples/counter_demo_generated.rs (Rust演示)"
echo ""
echo "🚀 下一步:"
echo "   1. 使用生成的配置运行 Dubhe Channel"
echo "   2. 执行 Phase 1 链下加速演示"
echo "   3. 测试 Counter 合约的各种功能"
echo ""
echo "💡 运行演示命令:"
echo "   cargo run --example phase1_offchain_demo"
echo "   cargo run --example counter_demo_generated" 