//! 题目信息 HTML 解析
//!
//! 精确复刻 Python decode.py 中 decode_questions_info() 的逻辑，
//! 从题目页面 HTML 中提取表单数据和问题列表。
//! 注：字体解密 (FontDecoder) 将在 font 模块中实现，此处预留接口。

use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::error::AppError;
use crate::models::work::{Question, QuestionType};

/// 题目表单解析结果
#[derive(Debug, Clone)]
pub struct QuestionsFormData {
    /// 表单隐藏字段 (排除 answer* 字段)
    pub form_fields: HashMap<String, String>,
    /// 解析出的题目列表
    pub questions: Vec<Question>,
    /// answerwqbid 字段（所有问题 ID 的逗号分隔拼接）
    pub answerwqbid: String,
    /// 页面是否包含字体加密
    pub has_font_encryption: bool,
}

/// 缓存编译后的选择器
struct Selectors {
    form: Selector,
    input: Selector,
    font_style: Selector,
    single_ques: Selector,
    timu: Selector,
    title_div: Selector,
    ul: Selector,
    li: Selector,
}

static SELECTORS: OnceLock<Selectors> = OnceLock::new();

fn selectors() -> &'static Selectors {
    SELECTORS.get_or_init(|| Selectors {
        form: Selector::parse("form").unwrap(),
        input: Selector::parse("input").unwrap(),
        font_style: Selector::parse("style#cxSecretStyle").unwrap(),
        single_ques: Selector::parse("div.singleQuesId").unwrap(),
        timu: Selector::parse("div.TiMu").unwrap(),
        title_div: Selector::parse("div.Zy_TItle").unwrap(),
        ul: Selector::parse("ul").unwrap(),
        li: Selector::parse("li").unwrap(),
    })
}

/// 字体解码器 trait，支持外部注入字体解密实现
pub trait FontDecoder {
    fn decode(&self, text: &str) -> String;
}

/// 解析题目信息，提取表单数据和问题列表
///
/// 精确对应 Python decode_questions_info() 的行为：
/// - 提取 form 内所有非 answer 字段的 input
/// - 检测字体加密 (style#cxSecretStyle)
/// - 遍历 div.singleQuesId 解析每道题目
/// - 生成 answerwqbid 字段
pub fn parse_questions_info(
    html_content: &str,
    font_decoder: Option<&dyn FontDecoder>,
) -> Result<QuestionsFormData, AppError> {
    let document = Html::parse_document(html_content);
    let sel = selectors();

    // 提取表单字段
    let form_fields = extract_form_data(&document, sel);

    // 检查字体加密
    let has_font_encryption = document.select(&sel.font_style).next().is_some();

    // 处理所有问题
    let form_el = document.select(&sel.form).next();
    let mut questions = Vec::new();

    if let Some(form) = form_el {
        for div_tag in form.select(&sel.single_ques) {
            if let Some(question) = process_question(&div_tag, sel, font_decoder) {
                questions.push(question);
            }
        }
    }

    // 生成 answerwqbid：空 questions 时返回空字符串，
    // 避免下游把单独的 "," 误解析为 1 个空 ID
    let answerwqbid = if questions.is_empty() {
        String::new()
    } else {
        questions
            .iter()
            .map(|q| q.id.as_str())
            .collect::<Vec<_>>()
            .join(",")
            + ","
    };

    Ok(QuestionsFormData {
        form_fields,
        questions,
        answerwqbid,
        has_font_encryption,
    })
}

/// 从 form 中提取所有非 answer 字段的 input
fn extract_form_data(
    document: &Html,
    sel: &Selectors,
) -> HashMap<String, String> {
    let mut form_data = HashMap::new();

    let form = match document.select(&sel.form).next() {
        Some(f) => f,
        None => return form_data,
    };

    for input_el in form.select(&sel.input) {
        let attrs = input_el.value();
        let name = match attrs.attr("name") {
            Some(n) => n,
            None => continue,
        };
        // 跳过 answer 相关字段
        if name.contains("answer") {
            continue;
        }
        let value = attrs.attr("value").unwrap_or("").to_string();
        form_data.insert(name.to_string(), value);
    }

    form_data
}

/// 处理单个问题
fn process_question(
    div_tag: &scraper::ElementRef,
    sel: &Selectors,
    font_decoder: Option<&dyn FontDecoder>,
) -> Option<Question> {
    // 提取问题 ID
    let question_id = div_tag.value().attr("data")?.to_string();

    // 提取题目类型代码
    let type_code = div_tag
        .select(&sel.timu)
        .next()
        .and_then(|el| el.value().attr("data"))
        .unwrap_or("")
        .to_string();

    let question_type = QuestionType::from_code(&type_code);

    // 提取题目内容
    let title = div_tag
        .select(&sel.title_div)
        .next()
        .map(|el| extract_title(&el, font_decoder))
        .unwrap_or_default();

    // 提取选项
    let mut options_list: Vec<String> = Vec::new();
    if let Some(ul) = div_tag.select(&sel.ul).next() {
        for li in ul.select(&sel.li) {
            let choice = extract_choice(&li, font_decoder);
            if !choice.is_empty() {
                options_list.push(choice);
            }
        }
    }
    // 排序选项（与 Python 行为一致）
    options_list.sort();
    let options = options_list.join("\n");

    // 构建 answer_field
    let mut answer_field = HashMap::new();
    answer_field.insert(format!("answer{}", question_id), String::new());
    answer_field.insert(format!("answertype{}", question_id), type_code);

    Some(Question {
        id: question_id,
        title,
        options,
        question_type,
        answer_field,
    })
}

/// 提取标题内容，支持字体解码
///
/// 收集元素中所有文本和 img 标签，然后清理格式
fn extract_title(
    element: &scraper::ElementRef,
    font_decoder: Option<&dyn FontDecoder>,
) -> String {
    let mut content = String::new();

    // 遍历所有子节点：文本 + img
    for node in element.descendants() {
        match node.value() {
            scraper::Node::Text(text) => {
                content.push_str(text);
            }
            scraper::Node::Element(el) if el.name() == "img" => {
                if let Some(src) = el.attr("src") {
                    content.push_str(&format!("<img src=\"{}\">", src));
                }
            }
            _ => {}
        }
    }

    // 清理格式
    let cleaned = content.replace('\r', "").replace('\t', "").replace('\n', "");

    // 字体解码
    match font_decoder {
        Some(decoder) => decoder.decode(&cleaned),
        None => cleaned,
    }
}

/// 提取选项内容，支持字体解码
///
/// 优先使用 aria-label 属性，其次使用文本内容
fn extract_choice(
    element: &scraper::ElementRef,
    font_decoder: Option<&dyn FontDecoder>,
) -> String {
    // 优先取 aria-label
    let choice = element
        .value()
        .attr("aria-label")
        .map(String::from)
        .unwrap_or_else(|| element.text().collect::<String>());

    if choice.is_empty() {
        return String::new();
    }

    // 清理格式
    let mut cleaned = choice.replace('\r', "").replace('\t', "").replace('\n', "");

    // 字体解码
    if let Some(decoder) = font_decoder {
        cleaned = decoder.decode(&cleaned);
    }

    // 去除尾部 "选择" 二字（与 Python 行为一致）
    cleaned = cleaned.trim().to_string();
    if cleaned.ends_with("选择") {
        cleaned = cleaned[..cleaned.len() - "选择".len()].trim_end().to_string();
    }

    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_html() {
        let html = "<html><body></body></html>";
        let result = parse_questions_info(html, None).unwrap();
        assert!(result.questions.is_empty());
        assert!(result.form_fields.is_empty());
        assert!(!result.has_font_encryption);
    }

    #[test]
    fn test_parse_form_fields() {
        let html = r#"
        <html><body>
        <form>
            <input name="classId" value="12345" />
            <input name="courseid" value="67890" />
            <input name="answer123" value="A" />
            <input name="answertype123" value="0" />
            <input name="token" value="abc" />
        </form>
        </body></html>
        "#;

        let result = parse_questions_info(html, None).unwrap();
        assert_eq!(result.form_fields.get("classId"), Some(&"12345".to_string()));
        assert_eq!(result.form_fields.get("courseid"), Some(&"67890".to_string()));
        assert_eq!(result.form_fields.get("token"), Some(&"abc".to_string()));
        // answer 相关字段应被过滤
        assert!(!result.form_fields.contains_key("answer123"));
        assert!(!result.form_fields.contains_key("answertype123"));
    }

    #[test]
    fn test_parse_single_question() {
        let html = r#"
        <html><body>
        <form>
            <div class="singleQuesId" data="q001">
                <div class="TiMu" data="0">
                    <div class="Zy_TItle">以下哪个是正确的？</div>
                    <ul>
                        <li aria-label="A. 选项一">A. 选项一选择</li>
                        <li aria-label="B. 选项二">B. 选项二选择</li>
                    </ul>
                </div>
            </div>
        </form>
        </body></html>
        "#;

        let result = parse_questions_info(html, None).unwrap();
        assert_eq!(result.questions.len(), 1);

        let q = &result.questions[0];
        assert_eq!(q.id, "q001");
        assert_eq!(q.title, "以下哪个是正确的？");
        assert!(matches!(q.question_type, QuestionType::Single));
        assert_eq!(result.answerwqbid, "q001,");

        // 检查 answer_field
        assert!(q.answer_field.contains_key("answerq001"));
        assert_eq!(q.answer_field.get("answertypeq001"), Some(&"0".to_string()));
    }

    #[test]
    fn test_detect_font_encryption() {
        let html = r#"
        <html><head>
        <style id="cxSecretStyle">@font-face{...}</style>
        </head><body>
        <form></form>
        </body></html>
        "#;

        let result = parse_questions_info(html, None).unwrap();
        assert!(result.has_font_encryption);
    }

    #[test]
    fn test_choice_strips_trailing_xuanze() {
        let html = r#"
        <html><body>
        <form>
            <div class="singleQuesId" data="q002">
                <div class="TiMu" data="3">
                    <div class="Zy_TItle">判断题</div>
                    <ul>
                        <li>对选择</li>
                        <li>错选择</li>
                    </ul>
                </div>
            </div>
        </form>
        </body></html>
        "#;

        let result = parse_questions_info(html, None).unwrap();
        assert_eq!(result.questions.len(), 1);
        let options: Vec<&str> = result.questions[0].options.split('\n').collect();
        assert!(options.contains(&"对"));
        assert!(options.contains(&"错"));
    }
}
