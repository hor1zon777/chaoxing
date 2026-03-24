//! 课程列表 HTML 解析
//!
//! 精确复刻 Python decode.py 中 decode_course_list() 的逻辑，
//! 使用 scraper 提取 div.course 下的课程信息。

use regex::Regex;
use scraper::{Html, Selector};
use std::sync::OnceLock;

use crate::error::AppError;
use crate::models::course::Course;

/// 缓存编译后的正则和选择器，避免重复编译
struct Selectors {
    course: Selector,
    not_open_a: Selector,
    not_open_div: Selector,
    clazz_id: Selector,
    course_id: Selector,
    link_a: Selector,
    course_name: Selector,
    desc: Selector,
    teacher: Selector,
    cpi_re: Regex,
}

static SELECTORS: OnceLock<Selectors> = OnceLock::new();

fn selectors() -> &'static Selectors {
    SELECTORS.get_or_init(|| Selectors {
        course: Selector::parse("div.course").unwrap(),
        not_open_a: Selector::parse("a.not-open-tip").unwrap(),
        not_open_div: Selector::parse("div.not-open-tip").unwrap(),
        clazz_id: Selector::parse("input.clazzId").unwrap(),
        course_id: Selector::parse("input.courseId").unwrap(),
        link_a: Selector::parse("a").unwrap(),
        course_name: Selector::parse("span.course-name").unwrap(),
        desc: Selector::parse("p.margint10").unwrap(),
        teacher: Selector::parse("p.color3").unwrap(),
        cpi_re: Regex::new(r"cpi=(.*?)&").unwrap(),
    })
}

/// 解析课程列表页面 HTML，提取课程信息列表
///
/// 精确对应 Python decode_course_list() 的行为：
/// - 选取所有 div.course
/// - 跳过含 a.not-open-tip 或 div.not-open-tip 的未开放课程
/// - 提取 id, info, roleid, clazzId, courseId, cpi, title, desc, teacher
pub fn parse_course_list(html_text: &str) -> Result<Vec<Course>, AppError> {
    let document = Html::parse_document(html_text);
    let sel = selectors();
    let mut course_list = Vec::new();

    for course_el in document.select(&sel.course) {
        // 跳过未开放课程
        if course_el.select(&sel.not_open_a).next().is_some()
            || course_el.select(&sel.not_open_div).next().is_some()
        {
            continue;
        }

        let attrs = course_el.value();

        // 必需属性
        let id = attrs.attr("id").unwrap_or_default().to_string();
        let info = attrs.attr("info").unwrap_or_default().to_string();
        let roleid = attrs.attr("roleid").unwrap_or_default().to_string();

        // input.clazzId 的 value
        let clazz_id = course_el
            .select(&sel.clazz_id)
            .next()
            .and_then(|el| el.value().attr("value"))
            .unwrap_or_default()
            .to_string();

        // input.courseId 的 value
        let course_id = course_el
            .select(&sel.course_id)
            .next()
            .and_then(|el| el.value().attr("value"))
            .unwrap_or_default()
            .to_string();

        // 从第一个 <a> 的 href 中提取 cpi 参数
        let cpi = course_el
            .select(&sel.link_a)
            .next()
            .and_then(|el| el.value().attr("href"))
            .and_then(|href| {
                sel.cpi_re
                    .captures(href)
                    .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
            })
            .unwrap_or_default();

        // span.course-name 的 title 属性
        let title = course_el
            .select(&sel.course_name)
            .next()
            .and_then(|el| el.value().attr("title"))
            .unwrap_or_default()
            .to_string();

        // p.margint10 的 title 属性（可选）
        let desc = course_el
            .select(&sel.desc)
            .next()
            .and_then(|el| el.value().attr("title"))
            .unwrap_or_default()
            .to_string();

        // p.color3 的 title 属性
        let teacher = course_el
            .select(&sel.teacher)
            .next()
            .and_then(|el| el.value().attr("title"))
            .unwrap_or_default()
            .to_string();

        course_list.push(Course {
            id,
            course_id,
            clazz_id,
            cpi,
            title,
            teacher,
            desc,
            info,
            roleid,
        });
    }

    Ok(course_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_html() {
        let html = "<html><body></body></html>";
        let result = parse_course_list(html).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_course_list() {
        let html = r#"
        <html><body>
        <div class="course" id="c001" info="some-info" roleid="3">
            <input class="clazzId" value="12345" />
            <input class="courseId" value="67890" />
            <a href="/course/detail?cpi=abc123&otherparam=1">链接</a>
            <span class="course-name" title="高等数学">高等数学</span>
            <p class="margint10" title="课程描述">课程描述</p>
            <p class="color3" title="张三">张三</p>
        </div>
        </body></html>
        "#;

        let result = parse_course_list(html).unwrap();
        assert_eq!(result.len(), 1);

        let course = &result[0];
        assert_eq!(course.id, "c001");
        assert_eq!(course.course_id, "67890");
        assert_eq!(course.clazz_id, "12345");
        assert_eq!(course.cpi, "abc123");
        assert_eq!(course.title, "高等数学");
        assert_eq!(course.desc, "课程描述");
        assert_eq!(course.teacher, "张三");
        assert_eq!(course.info, "some-info");
        assert_eq!(course.roleid, "3");
    }

    #[test]
    fn test_skip_not_open_courses() {
        let html = r##"
        <html><body>
        <div class="course" id="c001" info="" roleid="">
            <a class="not-open-tip" href="#">未开放</a>
            <input class="clazzId" value="111" />
            <input class="courseId" value="222" />
            <a href="/course?cpi=x&">链接</a>
            <span class="course-name" title="已关闭课程"></span>
            <p class="color3" title="教师"></p>
        </div>
        <div class="course" id="c002" info="" roleid="">
            <div class="not-open-tip">未开放</div>
            <input class="clazzId" value="333" />
            <input class="courseId" value="444" />
            <a href="/course?cpi=y&">链接</a>
            <span class="course-name" title="另一门已关闭课程"></span>
            <p class="color3" title="教师"></p>
        </div>
        <div class="course" id="c003" info="ok" roleid="1">
            <input class="clazzId" value="555" />
            <input class="courseId" value="666" />
            <a href="/course?cpi=z&">链接</a>
            <span class="course-name" title="开放课程"></span>
            <p class="color3" title="李四"></p>
        </div>
        </body></html>
        "##;

        let result = parse_course_list(html).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "c003");
        assert_eq!(result[0].title, "开放课程");
    }

    #[test]
    fn test_missing_optional_fields() {
        let html = r#"
        <html><body>
        <div class="course" id="c001" info="" roleid="">
            <input class="clazzId" value="111" />
            <input class="courseId" value="222" />
            <a href="/course?cpi=abc&">链接</a>
            <span class="course-name" title="测试课程"></span>
            <p class="color3" title="教师A"></p>
        </div>
        </body></html>
        "#;

        let result = parse_course_list(html).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].desc, ""); // p.margint10 不存在时为空
    }
}
