use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
use base64::{engine::general_purpose, Engine as _};

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

const AES_KEY: &[u8; 16] = b"u2oh6Vu^HWe4_AES";
const AES_IV: &[u8; 16] = b"u2oh6Vu^HWe4_AES";

/// AES-128-CBC 加密，精确复刻 Python cipher.py 的行为
/// 输入: UTF-8 字符串 → PKCS7 填充 → AES-CBC 加密 → Base64 编码
pub fn encrypt(plaintext: &str) -> String {
    let plaintext_bytes = plaintext.as_bytes();
    // 计算 PKCS7 填充后的缓冲区大小
    let block_size = 16;
    let padded_len = ((plaintext_bytes.len() / block_size) + 1) * block_size;
    let mut buf = vec![0u8; padded_len];
    buf[..plaintext_bytes.len()].copy_from_slice(plaintext_bytes);

    let encryptor = Aes128CbcEnc::new(AES_KEY.into(), AES_IV.into());
    let ciphertext = encryptor
        .encrypt_padded_mut::<Pkcs7>(&mut buf, plaintext_bytes.len())
        .expect("加密失败");
    general_purpose::STANDARD.encode(ciphertext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_consistency() {
        // 基本测试: 确保加密结果非空且为有效 Base64
        let result = encrypt("test123");
        assert!(!result.is_empty());
        assert!(general_purpose::STANDARD.decode(&result).is_ok());
    }

    #[test]
    fn test_encrypt_deterministic() {
        // 相同输入应产生相同输出
        let r1 = encrypt("hello");
        let r2 = encrypt("hello");
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_encrypt_different_inputs() {
        // 不同输入应产生不同输出
        let r1 = encrypt("hello");
        let r2 = encrypt("world");
        assert_ne!(r1, r2);
    }
}
