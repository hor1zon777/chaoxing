/** 应用配置 */
export interface AppConfig {
  /** 播放速度 */
  speed: number;
  /** 任务数量 */
  jobs: number;
  /** 单门课程内章节里同时并行的任务数 */
  tasksPerChapter: number;
  /** 未开放操作: 重试或继续 */
  notopenAction: "retry" | "continue";
  /** 题库提供者 */
  tikuProvider: string;
  /** 题库令牌 */
  tikuTokens: string;
  /** 是否自动提交答案 */
  tikuSubmit: boolean;
  /** 题库覆盖率 */
  tikuCoverRate: number;
  /** 题库延迟 (毫秒) */
  tikuDelay: number;
  /** AI 端点 */
  aiEndpoint: string;
  /** AI 密钥 */
  aiKey: string;
  /** AI 模型名称 */
  aiModel: string;
  /** AI 代理地址 */
  aiProxy: string;
  /** AI 最小请求间隔 (毫秒) */
  aiMinInterval: number;
  /** SiliconFlow 密钥 */
  siliconflowKey: string;
  /** SiliconFlow 模型 */
  siliconflowModel: string;
  /** SiliconFlow 端点 */
  siliconflowEndpoint: string;
  /** 是否模糊搜索 */
  likeSearch: boolean;
  /** 是否启用视觉识别 */
  likeVision: boolean;
  /** 模糊搜索模型 */
  likeModel: string;
  /** 是否重试 */
  likeRetry: boolean;
  /** 重试次数 */
  likeRetryTimes: number;
  /** 题库适配器 URL */
  tikuAdapterUrl: string;
  /** 判断为 "对" 的关键词列表 */
  trueList: string;
  /** 判断为 "错" 的关键词列表 */
  falseList: string;
  /** 通知提供者 */
  notificationProvider: string;
  /** 通知 URL */
  notificationUrl: string;
  /** Telegram 聊天 ID */
  notificationTgChatId: string;
  /** 是否检查 LLM 连接 */
  checkLlmConnection: boolean;
}
