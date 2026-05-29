//! 学习通移动端 `apis`（groupyd / noteyd 等）接口签名
//!
//! 仅用于复现用户自有账号下的合规接口调用，不涉及绕过认证或风控。
//!
//! 签名算法（已用真实抓包 12/12 验证）：
//! ```text
//! base    = "k1=v1&k2=v2&..."         // 按调用方给定的顺序拼接，值做 RFC3986 百分号编码
//! inf_enc = md5(base + "&DESKey=" + KEY)   // 32 位小写十六进制
//! query   = base + "&inf_enc=" + inf_enc
//! ```
//! 关键性质：服务端基于「收到的查询串」重算签名，因此只要「签名用的串 == 实际发送的串」即可，
//! 编码细节天然一致。本模块构造的 `query` 应原样拼到 URL 后发送。
//!
//! `token` 与 `KEY` 为客户端硬编码常量（token 后缀 `5b002b42` 配对 KEY `Z(AfY@XS`）。

use md5_digest::{Digest, Md5};
use uuid::Uuid;

/// 硬编码的接口 token（appKey 性质，所有请求固定）
pub const INF_ENC_TOKEN: &str = "4faa8662c59590c6f43ae9fe5b002b42";

/// 与 token 配对的签名盐值（参数名为 DESKey，但实际仅用于 MD5）
pub const INF_ENC_KEY: &str = "Z(AfY@XS";

/// 计算字符串的 32 位小写十六进制 MD5
fn md5_hex(input: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// RFC3986 百分号编码：保留 `A-Za-z0-9-_.~`，其余字节转大写 `%XX`
///
/// 与抓包一致（逗号 → `%2C`）。对多字节字符按 UTF-8 逐字节编码。
fn pct_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for &b in value.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// 基于给定查询参数（保持传入顺序）构造带 `inf_enc` 的完整查询串。
///
/// 返回形如 `k1=v1&k2=v2&...&inf_enc=<hash>`，可直接拼到 `?` 之后。
pub fn sign_query(params: &[(&str, &str)]) -> String {
    let base = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, pct_encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    let inf_enc = md5_hex(&format!("{}&DESKey={}", base, INF_ENC_KEY));
    format!("{}&inf_enc={}", base, inf_enc)
}

/// 课程讨论区的 `bbsid` = MD5(courseId)（已用真实抓包验证）
pub fn bbsid_from_course_id(course_id: &str) -> String {
    md5_hex(course_id)
}

/// 生成一次性 `_c_0_` / 帖子 `uuid`（大写、带连字符，对齐 iOS 客户端形态）
pub fn new_c0() -> String {
    Uuid::new_v4().to_string().to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 从签名查询串中取出 inf_enc 段
    fn inf_enc_of(query: &str) -> &str {
        query.rsplit("inf_enc=").next().unwrap()
    }

    #[test]
    fn test_sign_query_addtopic_golden() {
        // 真实抓包 entry[2]：addTopic，参数顺序 token,_time,_c_0_
        let q = sign_query(&[
            ("token", INF_ENC_TOKEN),
            ("_time", "1780024030004"),
            ("_c_0_", "320C0E12-8128-441C-A88C-8EDF7DFA1F31"),
        ]);
        assert_eq!(inf_enc_of(&q), "c850987cfd9cf614d511c6b657538297");
    }

    #[test]
    fn test_sign_query_list_golden_with_comma_encoding() {
        // 真实抓包 entry[0]：getTopicListWithPoff，tags 含逗号（须编码为 %2C）
        let q = sign_query(&[
            ("puid", "422989682"),
            ("pageSize", "20"),
            ("searchType", "0"),
            ("order", "0"),
            ("bbsid", "2c582a4d46fc5e142d7b7129c1c8620a"),
            ("_time", "1780024031859"),
            ("maxW", "320"),
            ("token", INF_ENC_TOKEN),
            ("tags", "classId0000001,classId138824952,courseId260301095"),
            ("_c_0_", "B02FBC90-FA2F-4900-BBE5-C9866A3CE748"),
        ]);
        assert_eq!(inf_enc_of(&q), "463d95684e1af86cf8caacc4b22b310a");
        // 逗号必须被编码进查询串
        assert!(q.contains("tags=classId0000001%2CclassId138824952%2CcourseId260301095"));
    }

    #[test]
    fn test_bbsid_from_course_id_golden() {
        assert_eq!(
            bbsid_from_course_id("260301095"),
            "2c582a4d46fc5e142d7b7129c1c8620a"
        );
    }

    #[test]
    fn test_pct_encode() {
        assert_eq!(pct_encode("classId138824952"), "classId138824952");
        assert_eq!(pct_encode("a,b"), "a%2Cb");
        assert_eq!(pct_encode("a b"), "a%20b");
        assert_eq!(pct_encode("[]"), "%5B%5D");
        // 中文按 UTF-8 逐字节编码
        assert_eq!(pct_encode("中"), "%E4%B8%AD");
        // 保留字符不变
        assert_eq!(pct_encode("A-z0_9.~"), "A-z0_9.~");
    }

    #[test]
    fn test_new_c0_format() {
        let c0 = new_c0();
        assert_eq!(c0.len(), 36);
        assert_eq!(c0.matches('-').count(), 4);
        assert_eq!(c0, c0.to_uppercase());
        // 两次生成不同
        assert_ne!(new_c0(), new_c0());
    }
}
