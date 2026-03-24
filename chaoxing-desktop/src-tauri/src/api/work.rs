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
use crate::models::job::{Job, JobInfo};
use crate::models::video::StudyResult;
use crate::models::work::QuestionType;
use crate::parser::questions::{parse_questions_info, QuestionsFormData};
use crate::tiku::answer_check::cut;
use crate::tiku::TikuManager;

/// 选项字母前缀清理正则
static CLEAN_RE: OnceLock<Regex> = OnceLock::new();

fn clean_re() -> &'static Regex {
    CLEAN_RE.get_or_init(|| Regex::new(r"^[A-Za-z]|[.,!?;:，。！？；：]").unwrap())
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
) -> Result<StudyResult, AppError> {
    if tiku.disabled {
        tracing::info!("题库未启用，跳过章节检测");
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

    tracing::info!("章节检测: 共 {} 道题目", total_questions);

    // 搜题并匹配答案
    let mut found_answers = 0u32;
    let mut answer_map: HashMap<String, String> = HashMap::new();
    let mut answer_source_map: HashMap<String, String> = HashMap::new();

    for q in &questions_data.questions {
        let q_type_str = question_type_to_str(&q.question_type);

        // 搜题延迟
        if tiku.delay > 0.0 {
            tokio::time::sleep(std::time::Duration::from_secs_f64(tiku.delay)).await;
        }

        let query_result = tiku.query(&q.title, q_type_str, &q.options).await;

        let (answer, source) = if let Some(ref res) = query_result.answer {
            // 根据题目类型匹配答案到选项
            let matched = match q_type_str {
                "multiple" => match_multiple_answer(res, &q.options),
                "single" => match_single_answer(res, &q.options),
                "judgement" => {
                    let is_true = tiku.judgement_select(res);
                    if is_true {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                "completion" => res.clone(),
                // 简答题等其他类型直接使用答案
                _ => res.clone(),
            };

            if matched.is_empty() {
                tracing::warn!("找到答案但未能匹配选项 -> {} ，随机选择答案", res);
                (random_answer(q_type_str, &q.options), "random")
            } else {
                tracing::info!("成功获取到答案: {}", matched);
                found_answers += 1;
                (matched, "cover")
            }
        } else {
            // 随机答题
            (random_answer(q_type_str, &q.options), "random")
        };

        answer_source_map.insert(q.id.clone(), source.to_string());
        answer_map.insert(q.id.clone(), answer.clone());
        tracing::info!("{} 填写答案为 {}", q.title, answer);
    }

    let cover_rate = (found_answers as f64 / total_questions as f64) * 100.0;
    tracing::info!("章节检测题库覆盖率: {:.0}%", cover_rate);

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
    submit_work(client, &form_data, &py_flag).await
}

/// 获取题目页面并解析
///
/// 对应 Python fetch_response()（带重试）
async fn fetch_questions(
    client: &HttpClient,
    course_id: &str,
    clazz_id: &str,
    work_id: &str,
    jobid: &str,
    job_info: &JobInfo,
    enc: &str,
) -> Result<QuestionsFormData, AppError> {
    let url = "https://mooc1.chaoxing.com/mooc-ans/api/work";

    let params = [
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

        let resp = client
            .client
            .get(url)
            .query(&params)
            .send()
            .await;

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

        if resp.status().as_u16() != 200 {
            tracing::warn!(
                "无效响应 (Code: {})，重试中... ({}/3)",
                resp.status(),
                attempt + 1
            );
            let delay = std::time::Duration::from_secs(2u64.pow(attempt + 1));
            tokio::time::sleep(delay).await;
            continue;
        }

        let html = resp.text().await.map_err(|e| AppError::Parse(e.to_string()))?;

        // 检查是否教师未创建完成
        if html.contains("教师未创建完成该测验") {
            return Err(AppError::Other("教师未创建完成该测验".to_string()));
        }

        let questions = parse_questions_info(&html, None)?;

        if !questions.questions.is_empty() {
            return Ok(questions);
        }

        tracing::warn!("未解析到题目，重试中... ({}/3)", attempt + 1);
        let delay = std::time::Duration::from_secs(2u64.pow(attempt + 1));
        tokio::time::sleep(delay).await;
    }

    Err(last_err.unwrap_or_else(|| AppError::Other("获取题目超过最大重试次数".to_string())))
}

/// 匹配多选题答案到选项
///
/// 对应 Python study_work() 中的多选匹配逻辑
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
    let mut result = String::new();

    for a in &cleaned_answers {
        for o in &options_list {
            if is_subsequence(a, o) {
                // 取首字为答案（例如 A 或 B）
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

/// 匹配单选题答案到选项
fn match_single_answer(answer: &str, options: &str) -> String {
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
        "1".to_string()
    } else if cover_rate >= tiku.cover_rate * 100.0 {
        // 覆盖率达标，直接提交
        String::new()
    } else {
        tracing::info!(
            "章节检测题库覆盖率低于 {:.0}%，不予提交",
            tiku.cover_rate * 100.0
        );
        "1".to_string()
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

    // 添加 answerwqbid
    form.insert("answerwqbid".to_string(), questions_data.answerwqbid.clone());

    // 添加 pyFlag
    form.insert("pyFlag".to_string(), py_flag.to_string());

    // 添加每题的答案
    for q in &questions_data.questions {
        let source = answer_source_map
            .get(&q.id)
            .map(|s| s.as_str())
            .unwrap_or("random");

        let answer = answer_map.get(&q.id).cloned().unwrap_or_default();

        // 获取 answertype
        let answer_type = q
            .answer_field
            .get(&format!("answertype{}", q.id))
            .cloned()
            .unwrap_or_default();

        if py_flag == "1" {
            // 保存模式：仅保存题库查到的答案，随机答案留空
            let final_answer = if source == "cover" {
                answer
            } else {
                String::new()
            };
            form.insert(format!("answer{}", q.id), final_answer);
        } else {
            // 提交模式：所有答案都填入
            form.insert(format!("answer{}", q.id), answer);
        }

        form.insert(format!("answertype{}", q.id), answer_type);
    }

    form
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
}
