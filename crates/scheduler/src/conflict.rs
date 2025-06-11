//! 冲突分析模块

use anyhow::Result;
use std::collections::{HashMap, HashSet};

use crate::types::Transaction;

/// 冲突图
#[derive(Debug, Clone)]
pub struct ConflictGraph {
    pub nodes: usize,
    pub edges: Vec<(usize, usize)>,
    pub read_conflicts: HashMap<String, Vec<usize>>,
    pub write_conflicts: HashMap<String, Vec<usize>>,
}

/// 冲突分析器
pub struct ConflictAnalyzer {
    // 状态分析缓存
}

impl ConflictAnalyzer {
    pub fn new() -> Self {
        Self {}
    }

    /// 分析交易冲突并构建冲突图
    pub async fn analyze(&mut self, transactions: &[Transaction]) -> Result<ConflictGraph> {
        let mut read_conflicts = HashMap::new();
        let mut write_conflicts = HashMap::new();
        let mut edges = Vec::new();

        // 构建读写映射
        for (i, tx) in transactions.iter().enumerate() {
            for addr in &tx.read_set {
                read_conflicts.entry(addr.clone()).or_insert_with(Vec::new).push(i);
            }
            for addr in &tx.write_set {
                write_conflicts.entry(addr.clone()).or_insert_with(Vec::new).push(i);
            }
        }

        // 检测冲突
        for (addr, writers) in &write_conflicts {
            // Write-Write 冲突
            for i in 0..writers.len() {
                for j in i + 1..writers.len() {
                    edges.push((writers[i], writers[j]));
                }
            }

            // Write-Read 冲突
            if let Some(readers) = read_conflicts.get(addr) {
                for &writer in writers {
                    for &reader in readers {
                        if writer != reader {
                            edges.push((writer, reader));
                        }
                    }
                }
            }
        }

        Ok(ConflictGraph {
            nodes: transactions.len(),
            edges,
            read_conflicts,
            write_conflicts,
        })
    }
} 