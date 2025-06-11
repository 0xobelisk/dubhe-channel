/// 标准Sui测试网Counter共享对象
/// 任何人都可以增加计数，只有所有者可以重置
module counter::counter {
    use sui::object::{Self, UID};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};

    /// Counter结构体 - 共享对象
    public struct Counter has key {
        id: UID,
        owner: address,
        value: u64
    }

    /// 创建新的Counter并将其设为共享对象
    public fun create(ctx: &mut TxContext) {
        transfer::share_object(Counter {
            id: object::new(ctx),
            owner: tx_context::sender(ctx),
            value: 0
        })
    }

    /// 任何人都可以增加计数
    public fun increment(counter: &mut Counter) {
        counter.value = counter.value + 1;
    }

    /// 只有所有者可以重置计数
    public fun reset(counter: &mut Counter, ctx: &TxContext) {
        assert!(counter.owner == tx_context::sender(ctx), 0);
        counter.value = 0;
    }

    /// 只有所有者可以设置特定值
    public fun set_value(counter: &mut Counter, value: u64, ctx: &TxContext) {
        assert!(counter.owner == tx_context::sender(ctx), 0);
        counter.value = value;
    }

    /// 获取当前计数值（只读）
    public fun value(counter: &Counter): u64 {
        counter.value
    }

    /// 获取所有者地址（只读）
    public fun owner(counter: &Counter): address {
        counter.owner
    }
} 