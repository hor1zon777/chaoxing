use std::path::{Path, PathBuf};

pub fn get_cookie_file_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(PathBuf::from))
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| std::env::temp_dir())
        })
        .join(".chaoxing-desktop-cookies.txt")
}

pub fn clear_cookie_file(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}


/// 保存 cookies 到文件 (格式: k1=v1;k2=v2)
pub fn save_cookies_to_file(cookies: &[(String, String)], path: &Path) -> std::io::Result<()> {
    let content = cookies
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(";");
    std::fs::write(path, content)
}

/// 从文件加载 cookies
pub fn load_cookies_from_file(path: &Path) -> std::io::Result<Vec<(String, String)>> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(path)?;
    let cookies = content
        .split(';')
        .filter_map(|pair| {
            let pair = pair.trim();
            if pair.is_empty() {
                return None;
            }
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.trim().to_string();
            let value = parts.next().unwrap_or("").trim().to_string();
            Some((key, value))
        })
        .collect();
    Ok(cookies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_roundtrip() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_cookies_chaoxing.txt");
        let cookies = vec![
            ("key1".to_string(), "val1".to_string()),
            ("key2".to_string(), "val2".to_string()),
        ];
        save_cookies_to_file(&cookies, &path).unwrap();
        let loaded = load_cookies_from_file(&path).unwrap();
        assert_eq!(loaded, cookies);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_nonexistent() {
        let path = PathBuf::from("/tmp/nonexistent_cookies_chaoxing.txt");
        let loaded = load_cookies_from_file(&path).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_empty_cookies() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_empty_cookies_chaoxing.txt");
        let cookies: Vec<(String, String)> = vec![];
        save_cookies_to_file(&cookies, &path).unwrap();
        let loaded = load_cookies_from_file(&path).unwrap();
        assert!(loaded.is_empty());
        std::fs::remove_file(&path).ok();
    }
}
