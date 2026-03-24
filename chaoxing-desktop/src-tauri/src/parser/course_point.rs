//! 章节列表 HTML 解析
//!
//! 精确复刻 Python decode.py 中 decode_course_point() 的逻辑，
//! 从 div.chapter_unit 中提取章节知识点信息。

use regex::Regex;
use scraper::{Html, Selector};
use std::sync::OnceLock;

use crate::error::AppError;
use crate::models::chapter::{ChapterPoint, ChapterTree};

/// 缓存编译后的正则和选择器
struct Selectors {
    chapter_unit: Selector,
    li: Selector,
    div: Selector,
    click_title: Selector,
    job_count: Selector,
    hover_tips: Selector,
    id_re: Regex,
}

static SELECTORS: OnceLock<Selectors> = OnceLock::new();

fn selectors() -> &'static Selectors {
    SELECTORS.get_or_init(|| Selectors {
        chapter_unit: Selector::parse("div.chapter_unit").unwrap(),
        li: Selector::parse("li").unwrap(),
        div: Selector::parse("div").unwrap(),
        click_title: Selector::parse("a.clicktitle").unwrap(),
        job_count: Selector::parse("input.knowledgeJobCount").unwrap(),
        hover_tips: Selector::parse("span.bntHoverTips").unwrap(),
        id_re: Regex::new(r"^cur(\d{1,20})$").unwrap(),
    })
}

/// 解析章节列表页面，提取章节知识点信息
///
/// 精确对应 Python decode_course_point() 的行为：
/// - 遍历所有 div.chapter_unit
/// - 对每个 chapter_unit，遍历其中的 li > div[id]
/// - 从 div.id 中使用正则提取知识点 ID
/// - 从 a.clicktitle 提取标题
/// - 从 input.knowledgeJobCount 提取任务数量
/// - 从 span.bntHoverTips 判断是否需要解锁和是否已完成
pub fn parse_course_point(html_text: &str) -> Result<ChapterTree, AppError> {
    let document = Html::parse_document(html_text);
    let sel = selectors();
    let mut tree = ChapterTree {
        has_locked: false,
        points: Vec::new(),
    };

    for chapter_unit in document.select(&sel.chapter_unit) {
        let points = extract_points_from_chapter(&chapter_unit, sel);
        for point in &points {
            if point.need_unlock {
                tree.has_locked = true;
            }
        }
        tree.points.extend(points);
    }

    Ok(tree)
}

/// 从单个 chapter_unit 中提取所有知识点
fn extract_points_from_chapter(
    chapter_unit: &scraper::ElementRef,
    sel: &Selectors,
) -> Vec<ChapterPoint> {
    let mut point_list = Vec::new();

    for raw_point in chapter_unit.select(&sel.li) {
        // 获取 li 下的第一个 div（对应 Python 的 raw_point.div）
        let point = match raw_point.select(&sel.div).next() {
            Some(d) => d,
            None => continue,
        };

        // div 必须有 id 属性
        let div_id = match point.value().attr("id") {
            Some(id) => id,
            None => continue,
        };

        // 正则提取 point_id: "cur12345" -> "12345"
        let point_id = match sel.id_re.captures(div_id) {
            Some(caps) => caps.get(1).map(|m| m.as_str().to_string()),
            None => continue,
        };
        let point_id = match point_id {
            Some(id) => id,
            None => continue,
        };

        // 提取标题：a.clicktitle 的文本，去除换行和首尾空白
        let title = point
            .select(&sel.click_title)
            .next()
            .map(|el| {
                el.text()
                    .collect::<String>()
                    .replace('\n', "")
                    .trim()
                    .to_string()
            })
            .unwrap_or_default();

        // 提取任务数量和解锁状态
        let mut job_count: u32 = 1; // 默认为 1
        let mut need_unlock = false;

        if let Some(input) = point.select(&sel.job_count).next() {
            if let Some(val) = input.value().attr("value") {
                job_count = val.parse().unwrap_or(1);
            }
        } else if let Some(tips) = point.select(&sel.hover_tips).next() {
            let tips_text = tips.text().collect::<String>();
            if tips_text.contains("解锁") {
                need_unlock = true;
            }
        }

        // 判断是否已完成
        let has_finished = point
            .select(&sel.hover_tips)
            .next()
            .map(|tips| tips.text().collect::<String>().contains("已完成"))
            .unwrap_or(false);

        point_list.push(ChapterPoint {
            id: point_id,
            title,
            job_count,
            has_finished,
            need_unlock,
        });
    }

    point_list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_html() {
        let html = "<html><body></body></html>";
        let result = parse_course_point(html).unwrap();
        assert!(!result.has_locked);
        assert!(result.points.is_empty());
    }

    #[test]
    fn test_parse_course_point() {
        let html = r#"
        <html><body>
        <div class="chapter_unit">
            <li>
                <div id="cur10001">
                    <a class="clicktitle">
                        第一章 绪论
                    </a>
                    <input class="knowledgeJobCount" value="3" />
                    <span class="bntHoverTips">已完成</span>
                </div>
            </li>
            <li>
                <div id="cur10002">
                    <a class="clicktitle">第二章 基础</a>
                    <input class="knowledgeJobCount" value="5" />
                </div>
            </li>
        </div>
        </body></html>
        "#;

        let result = parse_course_point(html).unwrap();
        assert!(!result.has_locked);
        assert_eq!(result.points.len(), 2);

        assert_eq!(result.points[0].id, "10001");
        assert_eq!(result.points[0].title, "第一章 绪论");
        assert_eq!(result.points[0].job_count, 3);
        assert!(result.points[0].has_finished);
        assert!(!result.points[0].need_unlock);

        assert_eq!(result.points[1].id, "10002");
        assert_eq!(result.points[1].title, "第二章 基础");
        assert_eq!(result.points[1].job_count, 5);
        assert!(!result.points[1].has_finished);
    }

    #[test]
    fn test_parse_locked_point() {
        let html = r#"
        <html><body>
        <div class="chapter_unit">
            <li>
                <div id="cur20001">
                    <a class="clicktitle">第三章 进阶</a>
                    <span class="bntHoverTips">需要解锁</span>
                </div>
            </li>
        </div>
        </body></html>
        "#;

        let result = parse_course_point(html).unwrap();
        assert!(result.has_locked);
        assert_eq!(result.points.len(), 1);
        assert!(result.points[0].need_unlock);
        assert_eq!(result.points[0].job_count, 1); // 默认值
    }

    #[test]
    fn test_skip_div_without_id() {
        let html = r#"
        <html><body>
        <div class="chapter_unit">
            <li>
                <div>
                    <a class="clicktitle">无ID节点</a>
                </div>
            </li>
            <li>
                <div id="invalid_format">
                    <a class="clicktitle">ID格式不匹配</a>
                </div>
            </li>
            <li>
                <div id="cur30001">
                    <a class="clicktitle">有效节点</a>
                </div>
            </li>
        </div>
        </body></html>
        "#;

        let result = parse_course_point(html).unwrap();
        assert_eq!(result.points.len(), 1);
        assert_eq!(result.points[0].id, "30001");
    }
}
