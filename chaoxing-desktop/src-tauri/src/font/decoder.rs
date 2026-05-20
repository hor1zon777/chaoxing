use std::collections::HashMap;
use std::sync::OnceLock;

use base64::{engine::general_purpose, Engine};
use regex::Regex;

use crate::font::glyph::font2map;
use crate::font::hash_dao::FontHashDao;
use crate::font::radicals::replace_radicals;

/// 提取 base64 字体数据的正则表达式
static FONT_BASE64_RE: OnceLock<Regex> = OnceLock::new();

fn font_base64_regex() -> &'static Regex {
    FONT_BASE64_RE.get_or_init(|| {
        // 匹配 CSS @font-face 中的 base64 数据
        // 对应 Python: r"base64,([\w\W]+?)'"
        Regex::new(r"base64,([\w\W]+?)'").expect("编译字体正则失败")
    })
}

/// 超星加密字体解码器。
///
/// 对应 Python `font_decoder.py` 中的 `FontDecoder` 类。
/// 用于解码超星平台使用特殊字体加密的内容。
///
/// 工作流程:
/// 1. 从 HTML 中的 `<style id="cxSecretStyle">` 提取 base64 编码的 TTF 字体
/// 2. 解析字体文件，生成每个字形的 MD5 哈希
/// 3. 用哈希在预计算表中反查原始 Unicode 字符
/// 4. 最终替换康熙部首为标准汉字
pub struct FontDecoder {
    /// 目标字体的字形哈希映射 (glyph_name → md5_hash)
    font_map: HashMap<String, String>,
}

impl FontDecoder {
    /// 从 HTML 内容创建解码器。
    ///
    /// 提取 `<style id="cxSecretStyle">` 中的 base64 字体数据，
    /// 解析字体并生成字形哈希映射。
    ///
    /// 对应 Python `FontDecoder.__init_font_map(html_content)`。
    ///
    /// # 返回
    /// - `Some(FontDecoder)` — 成功解析字体
    /// - `None` — HTML 中未找到加密字体数据或解析失败
    pub fn from_html(html: &str) -> Option<Self> {
        // 查找 <style id="cxSecretStyle"> 标签内容
        // 使用 scraper 解析 HTML
        let document = scraper::Html::parse_document(html);
        let selector =
            scraper::Selector::parse("style#cxSecretStyle").ok()?;
        let style_element = document.select(&selector).next()?;
        let style_text = style_element.text().collect::<String>();

        if style_text.is_empty() {
            return None;
        }

        // 从 CSS 文本中提取 base64 字体数据
        let captures = font_base64_regex().captures(&style_text)?;
        let b64_data = captures.get(1)?.as_str();

        // 解码 base64
        let font_bytes = general_purpose::STANDARD.decode(b64_data).ok()?;

        let font_map = font2map(&font_bytes);
        if font_map.is_empty() {
            return None;
        }

        Some(Self { font_map })
    }

    /// 解密加密文本。
    ///
    /// 对应 Python `cxsecret_font.decrypt(dst_fontmap, encrypted_text)`。
    ///
    /// # 算法
    /// 1. 将每个字符的 Unicode 码点转为 `"uniXXXX"` 格式的字形名称
    /// 2. 在目标字体映射中查找该字形的 MD5 哈希
    /// 3. 用哈希在预计算表中反查原始 Unicode 字符名
    /// 4. 将字符名转回实际字符
    /// 5. 最后对整个结果做康熙部首替换
    pub fn decode(&self, text: &str) -> String {
        let dao = FontHashDao::global();
        let mut result = String::with_capacity(text.len());

        for ch in text.chars() {
            let unicode = ch as u32;
            let glyph_name = format!("uni{:04X}", unicode);

            // 在目标字体映射中查找该字符的哈希
            if let Some(hash) = self.font_map.get(&glyph_name) {
                // 用哈希在预计算表中反查原始 unicode
                if let Some(original_name) = dao.lookup_by_hash(hash) {
                    if let Some(stripped) = original_name.strip_prefix("uni")
                    {
                        if let Ok(code) = u32::from_str_radix(stripped, 16) {
                            if let Some(decoded_char) = char::from_u32(code) {
                                result.push(decoded_char);
                                continue;
                            }
                        }
                    }
                }
            }
            // 找不到映射，保留原字符
            result.push(ch);
        }

        // 最后做康熙部首替换
        replace_radicals(&result)
    }

    /// 获取字体映射表（用于调试）
    #[allow(dead_code)]
    pub fn font_map(&self) -> &HashMap<String, String> {
        &self.font_map
    }
}

/// 让 `font::decoder::FontDecoder` 实现 parser 层定义的同名 trait，
/// 使其可以作为 `parse_questions_info` 的字体解码器注入。
impl crate::parser::questions::FontDecoder for FontDecoder {
    fn decode(&self, text: &str) -> String {
        FontDecoder::decode(self, text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_html_no_style_tag() {
        let html = "<html><body>Hello</body></html>";
        assert!(FontDecoder::from_html(html).is_none());
    }

    #[test]
    fn test_from_html_empty_style() {
        let html = r#"<html><style id="cxSecretStyle"></style></html>"#;
        assert!(FontDecoder::from_html(html).is_none());
    }

    #[test]
    fn test_from_html_no_base64() {
        let html = r#"<html><style id="cxSecretStyle">body { color: red; }</style></html>"#;
        assert!(FontDecoder::from_html(html).is_none());
    }

    #[test]
    fn test_decode_preserves_ascii() {
        // 手动构造一个空映射的 decoder 来测试回退逻辑
        let decoder = FontDecoder {
            font_map: HashMap::new(),
        };
        // ASCII 字符不在 font_map 中，应原样保留
        assert_eq!(decoder.decode("hello world"), "hello world");
    }

    #[test]
    fn test_decode_applies_radical_replacement() {
        // 空映射 + 康熙部首输入 → 应该被替换
        let decoder = FontDecoder {
            font_map: HashMap::new(),
        };
        let input = "\u{2F00}\u{2F08}"; // ⼀⼈
        let output = decoder.decode(input);
        assert_eq!(output, "\u{4E00}\u{4EBA}"); // 一人
    }

    #[test]
    fn test_font_base64_regex() {
        let re = font_base64_regex();
        let css_text = "src: url('data:application/font-ttf;charset=utf-8;base64,AAECAW==')\nformat('truetype');";
        let captures = re.captures(css_text);
        assert!(captures.is_some());
        assert_eq!(captures.unwrap().get(1).unwrap().as_str(), "AAECAW==");
    }
}
