//! 讨论区相关数据模型（面向前端，camelCase）

use serde::{Deserialize, Serialize};

/// 发帖成功后的返回信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTopicResult {
    /// 新建话题 ID
    pub topic_id: i64,
    /// 话题分享链接
    pub share_url: String,
    /// 所属圈子 ID
    pub circle_id: i64,
    /// 所属圈子名称
    pub circle_name: String,
    /// 客户端生成的话题 uuid（与分享链接对应）
    pub uuid: String,
}

/// 讨论区话题列表项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicListItem {
    /// 话题 ID
    pub id: i64,
    /// 话题 uuid
    pub uuid: String,
    /// 标题
    pub title: String,
    /// 正文（纯文本概要）
    pub content: String,
    /// 发布者昵称
    pub creator_name: String,
    /// 创建时间（毫秒时间戳）
    pub create_time: i64,
    /// 回复数
    pub reply_count: i64,
    /// 阅读数
    pub read_count: i64,
    /// 点赞数
    pub praise_count: i64,
    /// 分享链接
    pub share_url: String,
}
