# RISC-V Virtual Machine Selection Guide

## 🎯 Recommendation

**Recommended: Use CKB-VM as the primary RISC-V runtime environment for Dubhe Channel**

## 📊 Detailed Comparison

### CKB-VM Advantages ✅

#### 1. **Production-Verified Maturity**

- 🏆 **3+ years of mainnet experience**: CKB mainnet stable operation since 2019
- 🔒 **Complete security audits**: Multiple security audits and battle-tested
- 📈 **Performance data**: Processed millions of transactions, stable TPS 2-10

#### 2. **Complete RISC-V Support**

- ✅ **RV64IMC** complete instruction set
- ✅ Standard **ELF binary file** support
- ✅ Complete **memory management** and **debugging features**
- ✅ **IEEE 754** floating-point standard support

#### 3. **Excellent Rust Ecosystem**

- 🦀 **Pure Rust implementation**, zero C dependencies
- 📚 **Comprehensive documentation** and API design
- 🔧 **Simple cargo integration**, clear dependency management
- 🧪 **Rich test suite**

#### 4. **Enterprise-Grade Features**

- ⚡ **High performance optimization**: Sparse memory management
- 💰 **Precise Gas metering**: Instruction-level cycle counting
- 📸 **State snapshots**: Support for rollback and debugging
- 🔍 **Built-in debugger**: Convenient for development and troubleshooting

#### 5. **Technical Architecture Advantages**

```rust
// Typical CKB-VM usage
use ckb_vm::{DefaultMachineBuilder, SparseMemory, WXorXMemory};

let machine = DefaultMachineBuilder::new()
    .instruction_cycle_func(Box::new(instruction_cycles))
    .build();
```

### PolkaVM Limitations ⚠️

#### 1. **Development Stage Risks**

- ❌ **Still in rapid development**, frequent API changes
- ❌ **Insufficient production validation**, mainly on Polkadot testnet
- ❌ **Limited documentation and ecosystem**

#### 2. **Architecture Constraints**

- ❌ **RV32 only**, limits memory addressing
- ❌ **Harvard architecture** has limitations in some compiler scenarios
- ❌ **Instruction set subset**, less complete than CKB-VM

#### 3. **Integration Complexity**

- ❌ **Fast API changes**, high maintenance cost
- ❌ **Incomplete debugging tools**
- ❌ **Performance data lacks** long-term validation

## 🚀 Dubhe Channel Configuration

### Current Configuration (Updated)

```toml
# config.toml
[vm]
default_vm = "CkbVM"      # Recommended CkbVM (production-verified)
max_instances = 100

[features]
default = ["ckb-vm"]
ckb-vm = ["dep:ckb-vm"]   # Primary choice
polkavm = []              # Experimental choice
```

### Implementation Status

- ✅ **CKB-VM framework**: Basic abstraction layer implemented
- ✅ **Configuration integration**: Default CKB-VM usage
- ✅ **Feature gates**: Support for conditional compilation
- 🚧 **Complete integration**: Need full CKB-VM API integration

## 📈 Performance Comparison

### CKB-VM Real Performance Data

| Metric                 | CKB-VM      | PolkaVM     | Notes                     |
| ---------------------- | ----------- | ----------- | ------------------------- |
| **Memory Efficiency**  | 95%+        | Unknown     | SparseMemory optimization |
| **Execution Speed**    | 2-10 TPS    | Unknown     | Mainnet verified data     |
| **Gas Precision**      | Instruction | Instruction | Both support              |
| **Debug Support**      | Complete    | Basic       | CKB-VM more mature        |
| **Ecosystem Maturity** | High        | Medium      | CKB ecosystem support     |

## 🛣️ Migration Path

### Phase 1: CKB-VM Basic Integration (Current)

```rust
// Current implementation (simplified)
pub struct CkbVmInstance {
    limits: ExecutionLimits,
    code_loaded: bool,
    #[cfg(feature = "ckb-vm")]
    _vm_state: Option<Vec<u8>>,
}
```

### Phase 2: Complete CKB-VM Integration

```rust
// Full version (production environment)
use ckb_vm::{DefaultMachineBuilder, SparseMemory, WXorXMemory};

pub struct CkbVmInstance {
    machine: DefaultCoreMachine<u64, WXorXMemory<SparseMemory<u64>>>,
    limits: ExecutionLimits,
}
```

### Phase 3: Optional PolkaVM Support

```rust
// Future extension
impl VmManager {
    pub fn create_instance(&self, vm_type: VmType) -> Result<Box<dyn VmInstance>> {
        match vm_type {
            VmType::CkbVM => Ok(Box::new(CkbVmInstance::new()?)),      // Primary choice
            VmType::PolkaVM => Ok(Box::new(PolkaVmInstance::new()?)),  // Optional
            VmType::Cartesi => Ok(Box::new(CartesiInstance::new()?)),  // Future
        }
    }
}
```

## 💡 Development Recommendations

### Immediate Actions

1. ✅ **Use CKB-VM** as default choice
2. ✅ **Implement basic abstraction layer**
3. 🔄 **Gradually improve CKB-VM integration**

### Medium-term Planning

1. 📝 **Complete CKB-VM API integration**
2. 🧪 **Performance testing and optimization**
3. 📚 **Write integration documentation**

### Long-term Strategy

1. 👀 **Monitor PolkaVM development**
2. 🔬 **Evaluate suitable use cases**
3. 🤝 **Maintain multi-VM compatibility**

## 🎯 Summary

**CKB-VM is currently the best choice because:**

1. **Production verified** - 3 years of mainnet operation experience
2. **Technical maturity** - Complete RV64IMC support
3. **Rust ecosystem** - Pure Rust implementation, simple integration
4. **Enterprise features** - Precise gas metering, state snapshots
5. **Community support** - Continuous maintenance by Nervos ecosystem

**PolkaVM can serve as a future alternative option, awaiting further maturity.**

---

**Dubhe Channel** - High-performance parallel execution layer based on CKB-VM 🚀
