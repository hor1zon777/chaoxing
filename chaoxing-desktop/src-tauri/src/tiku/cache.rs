//! 题库缓存 DAO
//!
//! 对应 Python CacheDAO 类
//! JSON 文件缓存，支持原子写入和并发安全

use std::collections::HashMap;
use std::path::PathBuf;

use tokio::sync::RwLock;
use tracing;

/// 题库答案缓存
///
/// 使用 tokio::sync::RwLock 保证并发安全，
/// 文件写入使用先写临时文件再 rename 的原子操作模式。
pub struct CacheDAO {
    cache: RwLock<HashMap<String, String>>,
    file_path: PathBuf,
}

impl CacheDAO {
    /// 创建缓存实例
    ///
    /// 如果缓存文件存在，则加载；否则从空 HashMap 开始。
    pub fn new(file_path: PathBuf) -> Self {
        let cache = if file_path.exists() {
            match std::fs::read_to_string(&file_path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                    tracing::warn!("缓存文件 JSON 解析失败: {}，使用空缓存", e);
                    // 尝试备份损坏的缓存文件
                    let bak_path = file_path.with_extension("json.bak");
                    let _ = std::fs::copy(&file_path, &bak_path);
                    HashMap::new()
                }),
                Err(e) => {
                    tracing::warn!("读取缓存文件失败: {}，使用空缓存", e);
                    HashMap::new()
                }
            }
        } else {
            HashMap::new()
        };

        Self {
            cache: RwLock::new(cache),
            file_path,
        }
    }

    /// 查询缓存中的答案
    pub async fn get(&self, question: &str) -> Option<String> {
        self.cache.read().await.get(question).cloned()
    }

    /// 写入缓存（内存 + 持久化到文件）
    pub async fn set(&self, question: &str, answer: &str) {
        let mut cache = self.cache.write().await;
        cache.insert(question.to_string(), answer.to_string());
        // 持久化到文件（在持有写锁期间完成，保证一致性）
        self.save_to_file(&cache);
    }

    /// 原子写入缓存文件
    ///
    /// 对应 Python CacheDAO._write_cache() 的原子写入策略：
    /// 先写临时文件，再 rename 替换。
    fn save_to_file(&self, cache: &HashMap<String, String>) {
        // 确保父目录存在
        if let Some(parent) = self.file_path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    tracing::error!("创建缓存目录失败: {}", e);
                    return;
                }
            }
        }

        match serde_json::to_string_pretty(cache) {
            Ok(content) => {
                let tmp_path = self.file_path.with_extension("tmp");
                match std::fs::write(&tmp_path, &content) {
                    Ok(()) => {
                        if let Err(e) = std::fs::rename(&tmp_path, &self.file_path) {
                            tracing::error!("重命名缓存临时文件失败: {}", e);
                            // 清理临时文件
                            let _ = std::fs::remove_file(&tmp_path);
                        }
                    }
                    Err(e) => {
                        tracing::error!("写入缓存临时文件失败: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("序列化缓存失败: {}", e);
            }
        }
    }

    /// 获取缓存条目数量
    pub async fn len(&self) -> usize {
        self.cache.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_cache_get_set() {
        let tmp_dir = std::env::temp_dir().join("chaoxing_test_cache");
        let _ = std::fs::create_dir_all(&tmp_dir);
        let file_path = tmp_dir.join("test_cache.json");
        // 清理旧文件
        let _ = std::fs::remove_file(&file_path);

        let cache = CacheDAO::new(file_path.clone());
        assert!(cache.get("问题1").await.is_none());

        cache.set("问题1", "答案1").await;
        assert_eq!(cache.get("问题1").await, Some("答案1".to_string()));

        // 验证持久化
        let cache2 = CacheDAO::new(file_path.clone());
        assert_eq!(cache2.get("问题1").await, Some("答案1".to_string()));

        // 清理
        let _ = std::fs::remove_file(&file_path);
        let _ = std::fs::remove_dir(&tmp_dir);
    }

    #[tokio::test]
    async fn test_cache_nonexistent_file() {
        let path = PathBuf::from("/tmp/chaoxing_nonexistent_test_cache_12345.json");
        let _ = std::fs::remove_file(&path);
        let cache = CacheDAO::new(path.clone());
        assert_eq!(cache.len().await, 0);
        let _ = std::fs::remove_file(&path);
    }
}
