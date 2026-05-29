//! 学习通响应解码辅助
//!
//! 部分 `apis`（如 groupyd 讨论区）接口的响应实际为 GBK 编码，
//! 但 Content-Type 谎称 UTF-8，直接用 `reqwest::Response::text()` 会乱码。
//! 这里统一从原始字节解码：优先 UTF-8，失败再退回 GBK（兼容 GB18030）。

use encoding_rs::GBK;

/// 将学习通响应字节解码为字符串。
///
/// - 合法 UTF-8：原样返回。
/// - 否则按 GBK 解码（学习通历史接口常见编码）。
pub fn decode_cx_bytes(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (cow, _, _) = GBK.decode(bytes);
            cow.into_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_valid_utf8() {
        let s = "话题发布成功";
        assert_eq!(decode_cx_bytes(s.as_bytes()), s);
    }

    #[test]
    fn test_decode_gbk_fallback() {
        // "话题" 的 GBK 字节
        let gbk_bytes = [0xBB, 0xB0, 0xCC, 0xE2];
        assert_eq!(decode_cx_bytes(&gbk_bytes), "话题");
    }

    #[test]
    fn test_decode_ascii_json_with_gbk_msg() {
        // 结构为 ASCII、msg 为 GBK 中文：整体非法 UTF-8 → 走 GBK 分支，中文正确还原
        let mut bytes = b"{\"result\":1,\"msg\":\"".to_vec();
        bytes.extend_from_slice(&[0xB3, 0xC9, 0xB9, 0xA6]); // "成功" 的 GBK
        bytes.extend_from_slice(b"\"}");
        let s = decode_cx_bytes(&bytes);
        assert!(s.contains("\"result\":1"));
        assert!(s.contains("成功"));
    }
}
