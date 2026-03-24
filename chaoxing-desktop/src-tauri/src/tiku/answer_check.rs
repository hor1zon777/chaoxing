//! 答案检查与分割
//!
//! 精确复刻 Python answer_check.py 的逻辑：
//! - cut(): 使用 19 种分隔符依次尝试分割答案
//! - check_answer(): 根据题目类型验证答案有效性

/// 分割答案文本
///
/// 对应 Python: cut()
/// 按 19 种分隔符依次尝试，找到第一个能成功分割（产生多于 1 个非空部分）的分隔符即返回。
/// 若所有分隔符都无法产生多段，则返回原始答案作为单元素列表（trim 后为空则返回 None）。
pub fn cut(answer: &str) -> Option<Vec<String>> {
    let separators = [
        "\n", ",", "\u{ff0c}", // ，
        "|", "\r", "\t", "#", "*", "-", "_", "+", "@", "~", "/", "\\", ".", "&", " ",
        "\u{3001}", // 、
    ];

    for sep in &separators {
        if !answer.contains(sep) {
            continue;
        }
        let parts: Vec<String> = answer
            .split(sep)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !parts.is_empty() {
            return Some(parts);
        }
    }

    let trimmed = answer.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(vec![trimmed])
    }
}

/// 检查答案是否有效
///
/// 对应 Python: check_answer()
/// 根据题目类型使用不同的验证规则
pub fn check_answer(
    answer: &str,
    q_type: &str,
    true_list: &[String],
    false_list: &[String],
) -> bool {
    if answer.is_empty() {
        return false;
    }
    match q_type {
        "single" => {
            check_single(answer) && check_judgement(answer, true_list, false_list) == -1
        }
        "multiple" => {
            check_multiple(answer) && check_judgement(answer, true_list, false_list) == -1
        }
        "completion" => check_completion(answer),
        "judgement" => check_judgement(answer, true_list, false_list) != -1,
        // 未知类型不做匹配检查
        _ => true,
    }
}

/// 检查单选题答案
///
/// 对应 Python: check_single()
/// 要求 cut 后恰好只有 1 个部分
fn check_single(answer: &str) -> bool {
    match cut(answer) {
        Some(parts) if parts.len() == 1 => true,
        _ => false,
    }
}

/// 检查多选题答案
///
/// 对应 Python: check_multiple()
/// 要求 cut 后至少有 1 个部分
fn check_multiple(answer: &str) -> bool {
    match cut(answer) {
        Some(parts) if !parts.is_empty() => true,
        _ => false,
    }
}

/// 检查填空题答案
///
/// 对应 Python: check_completion()
/// 答案非空即有效
fn check_completion(answer: &str) -> bool {
    !answer.is_empty()
}

/// 检查判断题答案
///
/// 对应 Python: check_judgement()
/// 返回: 1=true, 0=false, -1=无法判断
pub fn check_judgement(answer: &str, true_list: &[String], false_list: &[String]) -> i32 {
    let a = answer.trim();
    if true_list.iter().any(|t| t == a) {
        return 1;
    }
    if false_list.iter().any(|f| f == a) {
        return 0;
    }
    -1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_true_list() -> Vec<String> {
        vec![
            "正确".to_string(),
            "对".to_string(),
            "\u{221a}".to_string(), // √
            "是".to_string(),
        ]
    }

    fn default_false_list() -> Vec<String> {
        vec![
            "错误".to_string(),
            "错".to_string(),
            "\u{00d7}".to_string(), // ×
            "否".to_string(),
            "不对".to_string(),
            "不正确".to_string(),
        ]
    }

    // ========== cut() 测试 ==========

    #[test]
    fn test_cut_newline() {
        let result = cut("答案A\n答案B").unwrap();
        assert_eq!(result, vec!["答案A", "答案B"]);
    }

    #[test]
    fn test_cut_comma() {
        let result = cut("答案A,答案B,答案C").unwrap();
        assert_eq!(result, vec!["答案A", "答案B", "答案C"]);
    }

    #[test]
    fn test_cut_chinese_comma() {
        let result = cut("答案A\u{ff0c}答案B").unwrap();
        assert_eq!(result, vec!["答案A", "答案B"]);
    }

    #[test]
    fn test_cut_pipe() {
        let result = cut("A|B|C").unwrap();
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_cut_tab() {
        let result = cut("答案A\t答案B").unwrap();
        assert_eq!(result, vec!["答案A", "答案B"]);
    }

    #[test]
    fn test_cut_hash() {
        let result = cut("A#B#C").unwrap();
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_cut_star() {
        let result = cut("A*B").unwrap();
        assert_eq!(result, vec!["A", "B"]);
    }

    #[test]
    fn test_cut_dash() {
        let result = cut("A-B-C").unwrap();
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_cut_underscore() {
        let result = cut("A_B").unwrap();
        assert_eq!(result, vec!["A", "B"]);
    }

    #[test]
    fn test_cut_plus() {
        let result = cut("A+B").unwrap();
        assert_eq!(result, vec!["A", "B"]);
    }

    #[test]
    fn test_cut_at() {
        let result = cut("A@B").unwrap();
        assert_eq!(result, vec!["A", "B"]);
    }

    #[test]
    fn test_cut_tilde() {
        let result = cut("A~B").unwrap();
        assert_eq!(result, vec!["A", "B"]);
    }

    #[test]
    fn test_cut_slash() {
        let result = cut("A/B").unwrap();
        assert_eq!(result, vec!["A", "B"]);
    }

    #[test]
    fn test_cut_backslash() {
        let result = cut("A\\B").unwrap();
        assert_eq!(result, vec!["A", "B"]);
    }

    #[test]
    fn test_cut_dot() {
        let result = cut("A.B").unwrap();
        assert_eq!(result, vec!["A", "B"]);
    }

    #[test]
    fn test_cut_ampersand() {
        let result = cut("A&B").unwrap();
        assert_eq!(result, vec!["A", "B"]);
    }

    #[test]
    fn test_cut_space() {
        let result = cut("A B C").unwrap();
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_cut_chinese_dunhao() {
        let result = cut("A\u{3001}B\u{3001}C").unwrap();
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_cut_single_answer() {
        let result = cut("单个答案").unwrap();
        assert_eq!(result, vec!["单个答案"]);
    }

    #[test]
    fn test_cut_none_for_empty() {
        assert!(cut("").is_none());
    }

    #[test]
    fn test_cut_none_for_whitespace() {
        assert!(cut("   ").is_none());
    }

    #[test]
    fn test_cut_trims_parts() {
        let result = cut(" A , B , C ").unwrap();
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_cut_priority_newline_over_comma() {
        // 包含 \n 和 , 时，优先按 \n 分割
        let result = cut("A,1\nB,2").unwrap();
        assert_eq!(result, vec!["A,1", "B,2"]);
    }

    // ========== check_answer() 测试 ==========

    #[test]
    fn test_check_answer_empty() {
        assert!(!check_answer("", "single", &default_true_list(), &default_false_list()));
    }

    #[test]
    fn test_check_single_valid() {
        assert!(check_answer(
            "选项A的内容",
            "single",
            &default_true_list(),
            &default_false_list()
        ));
    }

    #[test]
    fn test_check_single_rejects_judgement_answer() {
        // 单选题答案如果恰好是判断题的关键词，应该被拒绝
        assert!(!check_answer(
            "正确",
            "single",
            &default_true_list(),
            &default_false_list()
        ));
    }

    #[test]
    fn test_check_single_rejects_multiple_parts() {
        // cut("A\nB") 会产生 2 个部分，不是单选
        assert!(!check_answer(
            "A\nB",
            "single",
            &default_true_list(),
            &default_false_list()
        ));
    }

    #[test]
    fn test_check_multiple_valid() {
        assert!(check_answer(
            "选项A\n选项B",
            "multiple",
            &default_true_list(),
            &default_false_list()
        ));
    }

    #[test]
    fn test_check_multiple_rejects_judgement_answer() {
        assert!(!check_answer(
            "正确",
            "multiple",
            &default_true_list(),
            &default_false_list()
        ));
    }

    #[test]
    fn test_check_judgement_true() {
        assert!(check_answer(
            "正确",
            "judgement",
            &default_true_list(),
            &default_false_list()
        ));
    }

    #[test]
    fn test_check_judgement_false() {
        assert!(check_answer(
            "错误",
            "judgement",
            &default_true_list(),
            &default_false_list()
        ));
    }

    #[test]
    fn test_check_judgement_unknown() {
        assert!(!check_answer(
            "不知道",
            "judgement",
            &default_true_list(),
            &default_false_list()
        ));
    }

    #[test]
    fn test_check_completion_valid() {
        assert!(check_answer(
            "填空内容",
            "completion",
            &default_true_list(),
            &default_false_list()
        ));
    }

    #[test]
    fn test_check_unknown_type_always_valid() {
        assert!(check_answer(
            "任意答案",
            "shortanswer",
            &default_true_list(),
            &default_false_list()
        ));
    }

    // ========== check_judgement() 直接测试 ==========

    #[test]
    fn test_check_judgement_returns_1_for_true() {
        assert_eq!(check_judgement("对", &default_true_list(), &default_false_list()), 1);
    }

    #[test]
    fn test_check_judgement_returns_0_for_false() {
        assert_eq!(check_judgement("错", &default_true_list(), &default_false_list()), 0);
    }

    #[test]
    fn test_check_judgement_returns_neg1_for_unknown() {
        assert_eq!(
            check_judgement("随便", &default_true_list(), &default_false_list()),
            -1
        );
    }

    #[test]
    fn test_check_judgement_trims_whitespace() {
        assert_eq!(
            check_judgement(" 正确 ", &default_true_list(), &default_false_list()),
            1
        );
    }
}
