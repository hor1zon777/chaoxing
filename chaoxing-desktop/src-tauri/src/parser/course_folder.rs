//! 二级课程文件夹 HTML 解析
//!
//! 精确复刻 Python decode.py 中 decode_course_folder() 的逻辑，
//! 从 ul.file-list > li[fileid] 中提取文件夹 id 和名称。

use scraper::{Html, Selector};
use std::sync::OnceLock;

use crate::error::AppError;
use crate::models::course::CourseFolder;

/// 缓存编译后的选择器
struct Selectors {
    file_list_item: Selector,
    rename_input: Selector,
}

static SELECTORS: OnceLock<Selectors> = OnceLock::new();

fn selectors() -> &'static Selectors {
    SELECTORS.get_or_init(|| Selectors {
        file_list_item: Selector::parse("ul.file-list > li").unwrap(),
        rename_input: Selector::parse("input.rename-input").unwrap(),
    })
}

/// 解析二级课程列表页面，提取文件夹信息
///
/// 精确对应 Python decode_course_folder() 的行为：
/// - 选取 ul.file-list > li
/// - 跳过没有 fileid 属性的条目
/// - 提取 fileid 作为 id，input.rename-input 的 value 作为 rename
pub fn parse_course_folder(html_text: &str) -> Result<Vec<CourseFolder>, AppError> {
    let document = Html::parse_document(html_text);
    let sel = selectors();
    let mut folder_list = Vec::new();

    for li_el in document.select(&sel.file_list_item) {
        // 跳过没有 fileid 属性的元素
        let file_id = match li_el.value().attr("fileid") {
            Some(id) if !id.is_empty() => id.to_string(),
            _ => continue,
        };

        // 获取 input.rename-input 的 value
        let rename = li_el
            .select(&sel.rename_input)
            .next()
            .and_then(|el| el.value().attr("value"))
            .unwrap_or_default()
            .to_string();

        folder_list.push(CourseFolder {
            id: file_id,
            rename,
        });
    }

    Ok(folder_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_html() {
        let html = "<html><body></body></html>";
        let result = parse_course_folder(html).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_course_folder() {
        let html = r#"
        <html><body>
        <ul class="file-list">
            <li fileid="f001">
                <input class="rename-input" value="期末复习资料" />
            </li>
            <li fileid="f002">
                <input class="rename-input" value="课堂笔记" />
            </li>
        </ul>
        </body></html>
        "#;

        let result = parse_course_folder(html).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "f001");
        assert_eq!(result[0].rename, "期末复习资料");
        assert_eq!(result[1].id, "f002");
        assert_eq!(result[1].rename, "课堂笔记");
    }

    #[test]
    fn test_skip_items_without_fileid() {
        let html = r#"
        <html><body>
        <ul class="file-list">
            <li>
                <input class="rename-input" value="无ID项" />
            </li>
            <li fileid="">
                <input class="rename-input" value="空ID项" />
            </li>
            <li fileid="f100">
                <input class="rename-input" value="有效项" />
            </li>
        </ul>
        </body></html>
        "#;

        let result = parse_course_folder(html).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "f100");
        assert_eq!(result[0].rename, "有效项");
    }
}
