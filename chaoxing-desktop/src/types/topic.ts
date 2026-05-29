// 讨论区相关类型，严格对齐后端 src-tauri/src/models/topic.rs 的
// #[serde(rename_all = "camelCase")] 序列化产物。
//
// 注意：Rust 的 i64 经 serde 序列化为 JSON number，TS 侧统一用 number。
// 不要把 topicId / id / createTime 写成 string。

/** 发帖成功返回（对齐后端 AddTopicResult） */
export interface AddTopicResult {
  /** 新建话题 ID（i64） */
  topicId: number;
  /** 话题分享链接 */
  shareUrl: string;
  /** 所属圈子 ID（i64） */
  circleId: number;
  /** 所属圈子名称 */
  circleName: string;
  /** 客户端生成的话题 uuid（与分享链接对应） */
  uuid: string;
}

/** 讨论区话题列表项（对齐后端 TopicListItem） */
export interface TopicListItem {
  /** 话题 ID（i64）—— 列表渲染 key 用此字段，而非 topicId */
  id: number;
  /** 话题 uuid */
  uuid: string;
  /** 标题 */
  title: string;
  /** 正文（纯文本概要） */
  content: string;
  /** 发布者昵称 */
  creatorName: string;
  /** 创建时间（毫秒时间戳 i64）→ new Date(createTime) */
  createTime: number;
  /** 回复数 */
  replyCount: number;
  /** 阅读数 */
  readCount: number;
  /** 点赞数 */
  praiseCount: number;
  /** 分享链接 */
  shareUrl: string;
}
