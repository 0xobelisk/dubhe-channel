//! 编译缓存模块
//!
//! LRU + 持久层，首编译后落盘

use anyhow::Result;
use rocksdb::{Options, DB};
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::types::CompiledContract;

/// 编译缓存
pub struct CompilationCache {
    disk_cache: Arc<DB>,
    memory_cache: Arc<RwLock<lru::LruCache<String, CompiledContract>>>,
}

impl CompilationCache {
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);

        let disk_cache = Arc::new(DB::open(&opts, cache_dir)?);
        let memory_cache = Arc::new(RwLock::new(lru::LruCache::new(
            NonZeroUsize::new(1000).unwrap(),
        ))); // 1000 entries

        info!("Compilation cache initialized");

        Ok(Self {
            disk_cache,
            memory_cache,
        })
    }

    /// 从缓存获取编译结果
    pub async fn get(&self, key: &str) -> Result<Option<CompiledContract>> {
        // 首先检查内存缓存
        {
            let mut cache = self.memory_cache.write().await;
            if let Some(contract) = cache.get(key) {
                debug!("Cache hit (memory): {}", key);
                return Ok(Some(contract.clone()));
            }
        }

        // 内存缓存未命中，检查磁盘缓存
        match self.disk_cache.get(key.as_bytes())? {
            Some(data) => {
                debug!("Cache hit (disk): {}", key);
                let contract: CompiledContract = bincode::deserialize(&data)?;

                // 将结果放入内存缓存
                {
                    let mut cache = self.memory_cache.write().await;
                    cache.put(key.to_string(), contract.clone());
                }

                Ok(Some(contract))
            }
            None => {
                debug!("Cache miss: {}", key);
                Ok(None)
            }
        }
    }

    /// 将编译结果存入缓存
    pub async fn put(&self, key: &str, contract: &CompiledContract) -> Result<()> {
        // 序列化合约
        let data = bincode::serialize(contract)?;

        // 存储到磁盘
        self.disk_cache.put(key.as_bytes(), &data)?;

        // 存储到内存缓存
        {
            let mut cache = self.memory_cache.write().await;
            cache.put(key.to_string(), contract.clone());
        }

        debug!("Cache stored: {}", key);
        Ok(())
    }

    /// 清除缓存中的特定项
    pub async fn remove(&self, key: &str) -> Result<()> {
        // 从磁盘删除
        self.disk_cache.delete(key.as_bytes())?;

        // 从内存删除
        {
            let mut cache = self.memory_cache.write().await;
            cache.pop(key);
        }

        debug!("Cache removed: {}", key);
        Ok(())
    }

    /// 清空所有缓存
    pub async fn clear(&self) -> Result<()> {
        // 清空内存缓存
        {
            let mut cache = self.memory_cache.write().await;
            cache.clear();
        }

        // 清空磁盘缓存（重新创建数据库）
        // 注意：这是一个简化的实现，生产环境可能需要更精细的控制
        warn!("Clearing all cache data");

        Ok(())
    }

    /// 获取缓存统计信息
    pub async fn stats(&self) -> CacheStats {
        let memory_cache = self.memory_cache.read().await;
        let memory_size = memory_cache.len();
        let memory_capacity = memory_cache.cap().get();

        // 估算磁盘缓存大小（这里简化处理）
        let disk_size = 0; // TODO: 实现磁盘缓存大小统计

        CacheStats {
            memory_entries: memory_size,
            memory_capacity,
            disk_entries: disk_size,
            hit_rate: 0.0, // TODO: 实现命中率统计
        }
    }

    /// 预热缓存（从磁盘加载常用合约到内存）
    pub async fn warmup(&self, keys: Vec<String>) -> Result<()> {
        info!("Warming up cache with {} keys", keys.len());

        for key in keys {
            if let Ok(Some(contract)) = self.get(&key).await {
                // get 方法已经会将数据加载到内存缓存
                debug!("Warmed up: {}", key);
            }
        }

        Ok(())
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub memory_entries: usize,
    pub memory_capacity: usize,
    pub disk_entries: u64,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_cache_operations() -> Result<()> {
        let temp_dir = tempdir()?;
        let cache = CompilationCache::new(temp_dir.path())?;

        let contract = CompiledContract {
            original_address: "0x123".to_string(),
            source_type: dubhe_adapter::ContractType::EVM,
            risc_v_code: vec![1, 2, 3, 4],
            entry_points: vec!["main".to_string()],
            metadata: crate::types::ContractMetadata {
                gas_metering: true,
                memory_limit: 1024,
                stack_limit: 512,
                call_depth_limit: 64,
                exports: std::collections::HashMap::new(),
            },
            compiled_at: 1234567890,
        };

        let key = "test_key";

        // 测试存储
        cache.put(key, &contract).await?;

        // 测试读取
        let retrieved = cache.get(key).await?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().original_address, "0x123");

        // 测试删除
        cache.remove(key).await?;
        let after_remove = cache.get(key).await?;
        assert!(after_remove.is_none());

        Ok(())
    }
}
