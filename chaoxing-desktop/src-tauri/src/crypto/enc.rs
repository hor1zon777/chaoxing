use md5::{Digest, Md5};

/// 生成视频进度上报的 enc 签名
/// 对应 Python: Chaoxing.get_enc()
pub fn get_video_enc(
    clazz_id: &str,
    jobid: &str,
    object_id: &str,
    playing_time: u64,
    duration: u64,
    userid: &str,
) -> String {
    let input = format!(
        "[{}][{}][{}][{}][{}][d_yHJ!$pdA~5][{}][0_{}]",
        clazz_id,
        userid,
        jobid,
        object_id,
        playing_time * 1000,
        duration * 1000,
        duration,
    );
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enc_format() {
        let enc = get_video_enc("12345", "job1", "obj1", 60, 300, "user1");
        // MD5 哈希为 32 位十六进制字符串
        assert_eq!(enc.len(), 32);
        assert!(enc.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_enc_deterministic() {
        let r1 = get_video_enc("12345", "job1", "obj1", 60, 300, "user1");
        let r2 = get_video_enc("12345", "job1", "obj1", 60, 300, "user1");
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_enc_different_inputs() {
        let r1 = get_video_enc("12345", "job1", "obj1", 60, 300, "user1");
        let r2 = get_video_enc("12345", "job1", "obj1", 120, 300, "user1");
        assert_ne!(r1, r2);
    }
}
