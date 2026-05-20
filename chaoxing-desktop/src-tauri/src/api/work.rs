//! 章节检测（答题）API
//!
//! 对应 Python Chaoxing.study_work()
//! 完整流程: 获取题目 -> 解析 -> 搜题 -> 匹配选项 -> 提交/保存

use std::collections::HashMap;

use regex::Regex;
use std::sync::OnceLock;
use tracing;

use crate::api::client::HttpClient;
use crate::error::AppError;
use crate::font::decoder::FontDecoder;
use crate::models::events::TaskEvent;
use crate::models::job::{Job, JobInfo};
use crate::models::video::StudyResult;
use crate::models::work::QuestionType;
use crate::parser::questions::{parse_questions_info, QuestionsFormData};
use crate::tiku::answer_check::cut;
use crate::tiku::TikuManager;

/// 选项字母前缀清理正则
static CLEAN_RE: OnceLock<Regex> = OnceLock::new();

fn clean_re() -> &'static Regex {
    CLEAN_RE.get_or_init(|| Regex::new(r"^[A-Za-z][.、]?\s*|[.,!?;:，。！？；：]+$").unwrap())
}

/// 学习作业/章节检测
///
/// 对应 Python Chaoxing.study_work()
/// 完整答题流程：获取题目HTML -> 解析题目 -> 搜题 -> 匹配选项 -> 提交
pub async fn study_work(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    _cpi: &str,
    job: &Job,
    job_info: &JobInfo,
    tiku: &TikuManager,
    event_tx: Option<&tokio::sync::mpsc::UnboundedSender<TaskEvent>>,
) -> Result<StudyResult, AppError> {
    if tiku.disabled {
        tracing::info!("题库未启用，跳过章节检测: {}", job.name);
        return Ok(StudyResult::Success);
    }

    // 获取题目页面 HTML
    let work_id = job.jobid.replace("work-", "");
    let questions_data = fetch_questions(
        client, course_id, clazz_id, &work_id, &job.jobid, job_info, &job.enc,
    )
    .await?;

    let total_questions = questions_data.questions.len();
    if total_questions == 0 {
        tracing::warn!("章节检测: 未获取到题目");
        return Ok(StudyResult::Success);
    }

    tracing::info!(
        "章节检测开始: {} (共 {} 道题目，题库: {})",
        job.name,
        total_questions,
        tiku.provider.name()
    );

    // AI 搜题并匹配答案
    let mut found_answers = 0u32;
    let mut answer_map: HashMap<String, String> = HashMap::new();
    let mut answer_source_map: HashMap<String, String> = HashMap::new();

    for q in &questions_data.questions {
        let q_type_str = question_type_to_str(&q.question_type);

        // 搜题延迟
        if tiku.delay > 0.0 {
            tokio::time::sleep(std::time::Duration::from_secs_f64(tiku.delay)).await;
        }

        tracing::info!("正在查询: {} [{}]", q.title, q_type_str);
        let query_result = tiku.query(&q.title, q_type_str, &q.options).await;

        let (answer, source) = if let Some(ref res) = query_result.answer {
            // 根据题目类型匹配答案到选项
            let matched = match q_type_str {
                "multiple" => match_multiple_answer(res, &q.options),
                "single" => match_single_answer(res, &q.options),
                "judgement" => {
                    if res.trim().eq_ignore_ascii_case("true")
                        || res.trim() == "正确"
                        || res.trim() == "对"
                    {
                        "true".to_string()
                    } else if res.trim().eq_ignore_ascii_case("false")
                        || res.trim() == "错误"
                        || res.trim() == "错"
                    {
                        "false".to_string()
                    } else {
                        let is_true = tiku.judgement_select(res);
                        if is_true { "true".to_string() } else { "false".to_string() }
                    }
                }
                "completion" | "shortanswer" => res.trim().to_string(),
                _ => res.clone(),
            };

            if matched.is_empty() {
                if q_type_str == "completion" || q_type_str == "shortanswer" {
                    tracing::warn!(
                        "AI 返回填空/简答答案 '{}' 但内容为空，跳过该题（不保存不提交）: {}",
                        res,
                        q.title
                    );
                    (String::new(), "skip")
                } else {
                    tracing::warn!("AI 答案 '{}' 未能匹配选项，使用随机答案", res);
                    (random_answer(q_type_str, &q.options), "random")
                }
            } else {
                tracing::info!("AI 匹配成功: {} -> {}", q.title, matched);
                found_answers += 1;
                (matched, "cover")
            }
        } else if q_type_str == "completion" || q_type_str == "shortanswer" {
            // 填空/简答题 AI 失败：跳过该题，不写入提交表单
            tracing::warn!(
                "AI 未返回填空/简答答案: {}，跳过该题（不保存不提交）",
                q.title
            );
            (String::new(), "skip")
        } else {
            tracing::warn!("AI 未返回答案: {}，使用随机答案", q.title);
            (random_answer(q_type_str, &q.options), "random")
        };

        answer_source_map.insert(q.id.clone(), source.to_string());
        answer_map.insert(q.id.clone(), answer.clone());

        // 发送答题事件
        if let Some(tx) = event_tx {
            let _ = tx.send(TaskEvent::WorkQuestionAnswered {
                course_id: course_id.to_string(),
                question_title: q.title.clone(),
                answer: answer.clone(),
                source: source.to_string(),
            });
        }
    }

    let cover_rate = if total_questions > 0 {
        (found_answers as f64 / total_questions as f64) * 100.0
    } else {
        100.0
    };
    tracing::info!(
        "章节检测完成: AI 命中率 {:.0}% ({}/{})",
        cover_rate,
        found_answers,
        total_questions
    );

    // 决定提交模式
    let py_flag = determine_py_flag(tiku, cover_rate);

    // 组建提交表单
    let form_data = build_submit_form(
        &questions_data,
        &answer_map,
        &answer_source_map,
        &py_flag,
    );

    // 提交答案
    let result = submit_work(client, &form_data, &py_flag).await;

    // 发送作业提交事件
    if let Some(tx) = event_tx {
        let _ = tx.send(TaskEvent::WorkSubmitted {
            course_id: course_id.to_string(),
            chapter_title: job.name.clone(),
            cover_rate,
            submitted: py_flag.is_empty(),
        });
    }

    result
}

/// 获取题目页面并解析
///
/// 对应 Python fetch_response()（带重试）
/// 与 Python 原版保持一致：使用主 client，让 reqwest 自动跟随重定向
async fn fetch_questions(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    work_id: &str,
    jobid: &str,
    job_info: &JobInfo,
    enc: &str,
) -> Result<QuestionsFormData, AppError> {
    let api_url = "https://mooc1.chaoxing.com/mooc-ans/api/work";

    let params: Vec<(&str, &str)> = vec![
        ("api", "1"),
        ("workId", work_id),
        ("jobid", jobid),
        ("originJobId", jobid),
        ("needRedirect", "true"),
        ("skipHeader", "true"),
        ("knowledgeid", &job_info.knowledgeid),
        ("ktoken", &job_info.ktoken),
        ("cpi", &job_info.cpi),
        ("ut", "s"),
        ("clazzId", clazz_id),
        ("type", ""),
        ("enc", enc),
        ("mooc2", "1"),
        ("courseid", course_id),
    ];

    let mut last_err = None;

    for attempt in 0..3u32 {
        client.rate_limiter.limit_rate().await;

        let resp = client.client.get(api_url).query(&params).send().await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("请求失败: {}，重试中... ({}/3)", e, attempt + 1);
                last_err = Some(AppError::Network(e));
                let delay = std::time::Duration::from_secs(2u64.pow(attempt + 1));
                tokio::time::sleep(delay).await;
                continue;
            }
        };

        let status = resp.status().as_u16();

        let html = if status == 200 {
            match resp.text().await {
                Ok(h) => h,
                Err(e) => {
                    tracing::warn!("响应解码失败: {}，重试中... ({}/3)", e, attempt + 1);
                    last_err = Some(AppError::Parse(e.to_string()));
                    let delay = std::time::Duration::from_secs(2u64.pow(attempt + 1));
                    tokio::time::sleep(delay).await;
                    continue;
                }
            }
        } else {
            let body_preview = resp.text().await.unwrap_or_default();
            let preview: String = body_preview.chars().take(500).collect();
            tracing::warn!(
                "服务器返回错误状态码: {}，响应预览: {}，重试中... ({}/3)",
                status,
                preview.replace('\n', " "),
                attempt + 1
            );
            last_err = Some(AppError::Other(format!(
                "服务器返回错误状态码: {}",
                status
            )));
            let delay = std::time::Duration::from_secs(2u64.pow(attempt + 1));
            tokio::time::sleep(delay).await;
            continue;
        };

        // 检查是否教师未创建完成
        if html.contains("教师未创建完成该测验") {
            return Err(AppError::Other("教师未创建完成该测验".to_string()));
        }

        // 尝试构造字体解码器：当题目页含有 <style id="cxSecretStyle"> 时使用
        let font_decoder = FontDecoder::from_html(&html);
        if font_decoder.is_some() {
            tracing::info!("检测到加密字体，启用字体解密");
        }

        // 解析题目（解析失败也重试，与 Python 行为一致）
        let questions = match parse_questions_info(
            &html,
            font_decoder.as_ref().map(|d| d as &dyn crate::parser::questions::FontDecoder),
        ) {
            Ok(q) => q,
            Err(e) => {
                tracing::warn!("题目解析失败: {}，重试中... ({}/3)", e, attempt + 1);
                last_err = Some(e);
                let delay = std::time::Duration::from_secs(2u64.pow(attempt + 1));
                tokio::time::sleep(delay).await;
                continue;
            }
        };

        if !questions.questions.is_empty() {
            return Ok(questions);
        }

        let preview: String = html.chars().take(200).collect();
        tracing::warn!(
            "未解析到题目 (HTML 预览: {})，重试中... ({}/3)",
            preview.replace('\n', " "),
            attempt + 1
        );
        last_err = Some(AppError::Other(format!(
            "响应中未能解析到题目 (body 前200字: {})",
            preview.replace('\n', " ")
        )));
        let delay = std::time::Duration::from_secs(2u64.pow(attempt + 1));
        tokio::time::sleep(delay).await;
    }

    Err(last_err.unwrap_or_else(|| AppError::Other("获取题目超过最大重试次数".to_string())))
}

/// 匹配单选题答案到选项
fn match_single_answer(answer: &str, options: &str) -> String {
    let trimmed = answer.trim();

    // 如果答案本身就是一个选项字母 (A-Z)，直接返回
    if trimmed.len() == 1 {
        let c = trimmed.chars().next().unwrap();
        if c.is_ascii_uppercase() {
            // 验证该字母是否在选项中存在
            let first_chars: Vec<char> = options
                .split('\n')
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.chars().next())
                .collect();
            if first_chars.contains(&c) {
                return c.to_string();
            }
        }
    }

    let options_list = match cut(options) {
        Some(list) => list,
        None => return String::new(),
    };
    let cleaned = clean_res(&[answer.to_string()]);
    if cleaned.is_empty() {
        return String::new();
    }

    for o in &options_list {
        if is_subsequence(&cleaned[0], o) {
            return o.chars().next().map(|c| c.to_string()).unwrap_or_default();
        }
    }

    String::new()
}

/// 匹配多选题答案到选项
fn match_multiple_answer(answer: &str, options: &str) -> String {
    let options_list = match cut(options) {
        Some(list) => list,
        None => return String::new(),
    };

    let answer_parts = match cut(answer) {
        Some(list) => list,
        None => return String::new(),
    };

    let cleaned_answers = clean_res(&answer_parts);

    // 检查是否所有清理后的答案都是单个字母（AI 直接返回了 ABCD）
    let all_single_letters = cleaned_answers
        .iter()
        .all(|a| a.len() == 1 && a.chars().next().map_or(false, |c| c.is_ascii_uppercase()));

    if all_single_letters {
        let first_chars: Vec<char> = options_list
            .iter()
            .filter_map(|o| o.chars().next())
            .collect();
        let mut result = String::new();
        for a in &cleaned_answers {
            let c = a.chars().next().unwrap();
            if first_chars.contains(&c) {
                result.push(c);
            }
        }
        if !result.is_empty() {
            let mut chars: Vec<char> = result.chars().collect();
            chars.sort();
            return chars.into_iter().collect();
        }
    }

    // 子序列匹配
    let mut result = String::new();
    for a in &cleaned_answers {
        for o in &options_list {
            if is_subsequence(a, o) {
                if let Some(first_char) = o.chars().next() {
                    result.push(first_char);
                }
                break;
            }
        }
    }

    // 排序，否则提交失败
    let mut chars: Vec<char> = result.chars().collect();
    chars.sort();
    chars.into_iter().collect()
}

/// 清理答案中的字母编号和标点符号
///
/// 对应 Python clean_res()
fn clean_res(items: &[String]) -> Vec<String> {
    let re = clean_re();
    items
        .iter()
        .map(|c| {
            if c.len() > 1 {
                re.replace_all(c, "").trim().to_string()
            } else {
                c.clone()
            }
        })
        .collect()
}

/// 判断 a 是否是 o 的子序列
///
/// 对应 Python is_subsequence()
fn is_subsequence(a: &str, o: &str) -> bool {
    let mut o_chars = o.chars();
    a.chars().all(|c| o_chars.any(|oc| oc == c))
}

/// 随机选择答案
///
/// 对应 Python random_answer()
fn random_answer(q_type: &str, options: &str) -> String {
    use rand::Rng;

    // judgement 不需要 options
    if q_type != "judgement" && options.is_empty() {
        return String::new();
    }

    match q_type {
        "single" => {
            let option_lines: Vec<&str> = options.split('\n').filter(|s| !s.is_empty()).collect();
            if option_lines.is_empty() {
                return String::new();
            }
            let idx = rand::thread_rng().gen_range(0..option_lines.len());
            option_lines[idx]
                .chars()
                .next()
                .map(|c| c.to_string())
                .unwrap_or_default()
        }
        "multiple" => {
            let option_list = match cut(options) {
                Some(list) => list,
                None => return String::new(),
            };

            if option_list.is_empty() {
                return String::new();
            }

            let available = option_list.len();
            // 至少选 2 个，最多选 min(4, available)
            let min_count = std::cmp::min(2, available);
            let max_count = std::cmp::min(4, available);
            let count = if min_count >= max_count {
                min_count
            } else {
                rand::thread_rng().gen_range(min_count..=max_count)
            };

            // 随机选择
            use rand::seq::SliceRandom;
            let mut indices: Vec<usize> = (0..available).collect();
            indices.shuffle(&mut rand::thread_rng());
            let selected: Vec<char> = indices[..count]
                .iter()
                .filter_map(|&i| option_list[i].chars().next())
                .collect();
            let mut sorted: Vec<char> = selected;
            sorted.sort();
            let answer: String = sorted.into_iter().collect();
            tracing::info!("随机选择 -> {}", answer);
            answer
        }
        "judgement" => {
            let choice = rand::random::<bool>();
            let answer = if choice { "true" } else { "false" };
            tracing::info!("随机选择 -> {}", answer);
            answer.to_string()
        }
        _ => String::new(),
    }
}

/// 决定提交模式
///
/// 对应 Python study_work() 中的 pyFlag 逻辑
fn determine_py_flag(tiku: &TikuManager, cover_rate: f64) -> String {
    if tiku.get_submit_flag() == "1" {
        // 配置为仅保存不提交
        tracing::info!("题库配置为仅保存，不提交");
        "1".to_string()
    } else {
        // 提交模式：总是提交（AI 大模型已启用，信任其答案质量）
        tracing::info!("章节检测题库覆盖率: {:.0}%，将直接提交", cover_rate);
        String::new()
    }
}

/// 组建提交表单
///
/// 对应 Python study_work() 中的表单组建逻辑
fn build_submit_form(
    questions_data: &QuestionsFormData,
    answer_map: &HashMap<String, String>,
    answer_source_map: &HashMap<String, String>,
    py_flag: &str,
) -> HashMap<String, String> {
    let mut form = questions_data.form_fields.clone();

    // 添加 pyFlag
    form.insert("pyFlag".to_string(), py_flag.to_string());

    // 重新生成 answerwqbid（排除被跳过的题目）
    let active_ids: Vec<&str> = questions_data
        .questions
        .iter()
        .filter(|q| {
            answer_source_map.get(&q.id).map(|s| s.as_str()) != Some("skip")
        })
        .map(|q| q.id.as_str())
        .collect();
    let answerwqbid = if active_ids.is_empty() {
        String::new()
    } else {
        format!("{},", active_ids.join(","))
    };
    form.insert("answerwqbid".to_string(), answerwqbid);

    // 添加每题的答案
    for q in &questions_data.questions {
        let source = answer_source_map
            .get(&q.id)
            .map(|s| s.as_str())
            .unwrap_or("random");

        // 跳过被标记为 skip 的题目（填空/简答 AI 失败）
        if source == "skip" {
            tracing::info!("跳过题目（未写入提交表单）: {}", q.title);
            continue;
        }

        let answer = answer_map.get(&q.id).cloned().unwrap_or_default();

        // 获取 answertype
        let answer_type = q
            .answer_field
            .get(&format!("answertype{}", q.id))
            .cloned()
            .unwrap_or_default();

        // 填空 / 简答：用 <p>...</p> 包裹答案以匹配 UEditor 富文本格式
        // 否则超星后台解析后会显示为空（参考 cxmooc-tools 修复 commit afe12b0）
        let needs_html_wrap = matches!(
            q.question_type,
            QuestionType::Completion | QuestionType::ShortAnswer
        );
        let format_answer = |raw: String| -> String {
            if !needs_html_wrap || raw.is_empty() {
                return raw;
            }
            // 多空答案以 \n 分隔，每空独立 <p>
            raw.split('\n')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(html_escape)
                .map(|s| format!("<p>{}</p>", s))
                .collect::<Vec<_>>()
                .join("")
        };

        if py_flag == "1" {
            // 保存模式：仅保存题库查到的答案，随机答案留空
            let final_answer = if source == "cover" {
                format_answer(answer)
            } else {
                String::new()
            };
            form.insert(format!("answer{}", q.id), final_answer);
        } else {
            // 提交模式：所有答案都填入
            form.insert(format!("answer{}", q.id), format_answer(answer));
        }

        form.insert(format!("answertype{}", q.id), answer_type);
    }

    form
}

/// 转义 HTML 特殊字符，避免破坏 UEditor 富文本结构
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// 提交答案
///
/// 对应 Python 最后的 POST 提交
async fn submit_work(
    client: &HttpClient,
    form_data: &HashMap<String, String>,
    py_flag: &str,
) -> Result<StudyResult, AppError> {
    let url = "https://mooc1.chaoxing.com/mooc-ans/work/addStudentWorkNew";
    let mode_name = if py_flag.is_empty() { "提交" } else { "保存" };

    let resp = client
        .client
        .post(url)
        .header("X-Requested-With", "XMLHttpRequest")
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .header("Origin", "https://mooc1.chaoxing.com")
        .header(
            "Accept",
            "application/json, text/javascript, */*; q=0.01",
        )
        .header("Sec-Fetch-Site", "same-origin")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Dest", "empty")
        .form(form_data)
        .send()
        .await?;

    if resp.status().as_u16() != 200 {
        let text = resp.text().await.unwrap_or_default();
        tracing::error!("{}答题失败 -> {}", mode_name, text);
        return Ok(StudyResult::Error);
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::Parse(e.to_string()))?;

    let status = json["status"].as_bool().unwrap_or(false);
    let msg = json["msg"].as_str().unwrap_or("未知");

    if status {
        tracing::info!("{}答题成功 -> {}", mode_name, msg);
        Ok(StudyResult::Success)
    } else {
        tracing::error!("{}答题失败 -> {}", mode_name, msg);
        Ok(StudyResult::Error)
    }
}

/// 将 QuestionType 转为字符串
fn question_type_to_str(qt: &QuestionType) -> &str {
    match qt {
        QuestionType::Single => "single",
        QuestionType::Multiple => "multiple",
        QuestionType::Completion => "completion",
        QuestionType::Judgement => "judgement",
        QuestionType::ShortAnswer => "shortanswer",
        QuestionType::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_subsequence() {
        assert!(is_subsequence("abc", "aXbYcZ"));
        assert!(is_subsequence("ac", "abc"));
        assert!(!is_subsequence("cb", "abc"));
        assert!(is_subsequence("", "abc"));
        assert!(is_subsequence("a", "a"));
    }

    #[test]
    fn test_clean_res() {
        let items = vec!["A选项内容".to_string(), "B".to_string()];
        let result = clean_res(&items);
        assert_eq!(result[0], "选项内容");
        // 单字符不清理
        assert_eq!(result[1], "B");
    }

    #[test]
    fn test_match_single_answer() {
        let options = "A 选项一\nB 选项二\nC 选项三";
        let answer = "选项二";
        let result = match_single_answer(answer, options);
        assert_eq!(result, "B");
    }

    #[test]
    fn test_match_multiple_answer() {
        let options = "A 选项一\nB 选项二\nC 选项三";
        let answer = "选项一\n选项三";
        let result = match_multiple_answer(answer, options);
        assert_eq!(result, "AC");
    }

    #[test]
    fn test_random_answer_single() {
        let options = "A 选项一\nB 选项二";
        let answer = random_answer("single", options);
        assert!(answer == "A" || answer == "B");
    }

    #[test]
    fn test_random_answer_judgement() {
        let answer = random_answer("judgement", "");
        assert!(answer == "true" || answer == "false");
    }

    #[test]
    fn test_determine_py_flag_submit() {
        let config = crate::models::config::AppConfig {
            tiku_submit: true,
            tiku_cover_rate: 0.8,
            ..Default::default()
        };
        let manager = TikuManager::from_config(&config);
        // submit=true 意味着直接提交（flag=""）
        assert_eq!(determine_py_flag(&manager, 90.0), "");
    }

    #[test]
    fn test_determine_py_flag_save_only() {
        let config = crate::models::config::AppConfig {
            tiku_submit: false,
            tiku_cover_rate: 0.8,
            ..Default::default()
        };
        let manager = TikuManager::from_config(&config);
        // submit=false 意味着仅保存（flag="1"）
        assert_eq!(determine_py_flag(&manager, 90.0), "1");
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("<p>x</p>"), "&lt;p&gt;x&lt;/p&gt;");
        assert_eq!(html_escape("纯文本"), "纯文本");
        assert_eq!(html_escape("a\"b'c"), "a&quot;b&#39;c");
    }
}
