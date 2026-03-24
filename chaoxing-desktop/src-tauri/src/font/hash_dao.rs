use std::collections::HashMap;
use std::sync::OnceLock;

/// 字体哈希映射 DAO（Data Access Object）。
///
/// 维护预计算的字形哈希双向索引表，用于通过哈希值反查 Unicode 字符名。
/// 对应 Python `cxsecret_font.py` 中的 `FontHashDAO` 类。
pub struct FontHashDao {
    /// Unicode 字符名 → MD5 哈希（如 "uni4E00" → "abc123..."）
    pub char_map: HashMap<String, String>,
    /// MD5 哈希 → Unicode 字符名（如 "abc123..." → "uni4E00"）
    pub hash_map: HashMap<String, String>,
}

/// 嵌入的字体映射表 JSON 资源
const FONT_MAP_JSON: &str = include_str!("../../resources/font_map_table.json");

/// 全局单例
static FONT_HASH_DAO: OnceLock<FontHashDao> = OnceLock::new();

impl FontHashDao {
    /// 从 JSON 字符串加载字体哈希映射。
    ///
    /// JSON 格式：`{ "字形名": "MD5哈希", ... }`
    fn load_from_json(json_str: &str) -> Self {
        let raw: HashMap<String, String> =
            serde_json::from_str(json_str).unwrap_or_default();

        let mut hash_map = HashMap::with_capacity(raw.len());
        for (name, hash) in &raw {
            hash_map.insert(hash.clone(), name.clone());
        }

        Self {
            char_map: raw,
            hash_map,
        }
    }

    /// 获取全局单例（从嵌入的 `font_map_table.json` 资源惰性加载）。
    pub fn global() -> &'static Self {
        FONT_HASH_DAO.get_or_init(|| Self::load_from_json(FONT_MAP_JSON))
    }

    /// 通过哈希值反查 Unicode 字符名。
    ///
    /// 对应 Python `FontHashDAO.find_char(font_hash)`。
    pub fn lookup_by_hash(&self, hash: &str) -> Option<&str> {
        self.hash_map.get(hash).map(|s| s.as_str())
    }

    /// 通过 Unicode 字符名查找哈希值。
    ///
    /// 对应 Python `FontHashDAO.find_hash(char)`。
    #[allow(dead_code)]
    pub fn lookup_by_name(&self, name: &str) -> Option<&str> {
        self.char_map.get(name).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_loads_successfully() {
        let dao = FontHashDao::global();
        // font_map_table.json 至少应有几百个条目
        assert!(
            dao.char_map.len() > 100,
            "字体映射表条目数过少: {}",
            dao.char_map.len()
        );
        // hash_map 可能小于 char_map，因为不同字形名可能有相同的哈希值
        // （如 "A" 和 "Alpha" 共享同一字形外形）
        assert!(
            dao.hash_map.len() > 100,
            "哈希反查表条目数过少: {}",
            dao.hash_map.len()
        );
        assert!(dao.hash_map.len() <= dao.char_map.len());
    }

    #[test]
    fn test_bidirectional_lookup() {
        let dao = FontHashDao::global();

        // 从 font_map_table.json 前几行可知 ".notdef" 的哈希
        let notdef_hash = dao.lookup_by_name(".notdef");
        assert!(notdef_hash.is_some(), ".notdef 应存在于映射表中");

        let hash = notdef_hash.unwrap();
        let name = dao.lookup_by_hash(hash);
        assert_eq!(name, Some(".notdef"));
    }

    #[test]
    fn test_missing_lookup() {
        let dao = FontHashDao::global();
        assert_eq!(dao.lookup_by_hash("nonexistent_hash_value"), None);
        assert_eq!(dao.lookup_by_name("nonexistent_glyph_name"), None);
    }
}
