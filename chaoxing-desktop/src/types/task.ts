/** 任务事件类型 */
export type TaskEventType =
  | "courseStarted"
  | "courseCompleted"
  | "courseError"
  | "chapterStarted"
  | "chapterCompleted"
  | "chapterSkipped"
  | "chapterRetrying"
  | "jobStarted"
  | "jobCompleted"
  | "jobFailed"
  | "videoProgress"
  | "liveProgress"
  | "workQuestionAnswered"
  | "workSubmitted"
  | "allTasksCompleted"
  | "log";

/** 基础任务事件 */
export interface BaseTaskEvent {
  type: TaskEventType;
}

/** 课程开始事件 */
export interface CourseStartedEvent extends BaseTaskEvent {
  type: "courseStarted";
  courseId: string;
  courseTitle: string;
  totalChapters: number;
}

/** 课程完成事件 */
export interface CourseCompletedEvent extends BaseTaskEvent {
  type: "courseCompleted";
  courseId: string;
  courseTitle: string;
}

/** 课程错误事件 */
export interface CourseErrorEvent extends BaseTaskEvent {
  type: "courseError";
  courseId: string;
  courseTitle: string;
  error: string;
}

/** 章节开始事件 */
export interface ChapterStartedEvent extends BaseTaskEvent {
  type: "chapterStarted";
  courseId: string;
  chapterId: string;
  chapterTitle: string;
  jobCount: number;
}

/** 章节完成事件 */
export interface ChapterCompletedEvent extends BaseTaskEvent {
  type: "chapterCompleted";
  courseId: string;
  chapterId: string;
  chapterTitle: string;
}

/** 章节跳过事件 */
export interface ChapterSkippedEvent extends BaseTaskEvent {
  type: "chapterSkipped";
  courseId: string;
  chapterId: string;
  chapterTitle: string;
  reason: string;
}

/** 章节重试事件 */
export interface ChapterRetryingEvent extends BaseTaskEvent {
  type: "chapterRetrying";
  courseId: string;
  chapterId: string;
  chapterTitle: string;
  attempt: number;
  maxAttempts: number;
}

/** 任务开始事件 */
export interface JobStartedEvent extends BaseTaskEvent {
  type: "jobStarted";
  courseId: string;
  chapterId: string;
  jobId: string;
  jobName: string;
  jobType: string;
}

/** 任务完成事件 */
export interface JobCompletedEvent extends BaseTaskEvent {
  type: "jobCompleted";
  courseId: string;
  chapterId: string;
  jobId: string;
  jobName: string;
  jobType: string;
}

/** 任务失败事件 */
export interface JobFailedEvent extends BaseTaskEvent {
  type: "jobFailed";
  courseId: string;
  chapterId: string;
  jobId: string;
  jobName: string;
  error: string;
}

/** 视频进度事件 */
export interface VideoProgressEvent extends BaseTaskEvent {
  type: "videoProgress";
  courseId: string;
  jobId: string;
  jobName: string;
  currentTime: number;
  totalDuration: number;
}

/** 直播进度事件 */
export interface LiveProgressEvent extends BaseTaskEvent {
  type: "liveProgress";
  courseId: string;
  jobId: string;
  jobName: string;
  currentMinute: number;
  totalMinutes: number;
}

/** 答题事件 */
export interface WorkQuestionAnsweredEvent extends BaseTaskEvent {
  type: "workQuestionAnswered";
  courseId: string;
  questionTitle: string;
  answer: string;
  source: string;
}

/** 作业提交事件 */
export interface WorkSubmittedEvent extends BaseTaskEvent {
  type: "workSubmitted";
  courseId: string;
  chapterTitle: string;
  coverRate: number;
  submitted: boolean;
}

/** 日志事件 */
export interface LogEvent extends BaseTaskEvent {
  type: "log";
  level: string;
  message: string;
  timestamp: string;
}

/** 所有任务完成事件 */
export interface AllTasksCompletedEvent extends BaseTaskEvent {
  type: "allTasksCompleted";
}

/** 任务事件联合类型 */
export type TaskEvent =
  | CourseStartedEvent
  | CourseCompletedEvent
  | CourseErrorEvent
  | ChapterStartedEvent
  | ChapterCompletedEvent
  | ChapterSkippedEvent
  | ChapterRetryingEvent
  | JobStartedEvent
  | JobCompletedEvent
  | JobFailedEvent
  | VideoProgressEvent
  | LiveProgressEvent
  | WorkQuestionAnsweredEvent
  | WorkSubmittedEvent
  | LogEvent
  | AllTasksCompletedEvent;

/** 日志条目 */
export interface LogEntry {
  level: string;
  message: string;
  timestamp: string;
}

/** 课程进度 */
export interface CourseProgress {
  courseId: string;
  courseTitle: string;
  completedChapters: number;
  totalChapters: number;
  /** 已完成的 job 总数（容错用：当 chapterCompleted 漏发时仍能推进 UI 进度） */
  completedJobs: number;
  /** 已观察到的 job 总数（通过 chapterStarted.jobCount 累加） */
  totalJobs: number;
  status: "pending" | "running" | "completed" | "error";
}

/** 视频播放进度信息 */
export interface VideoProgressInfo {
  jobId: string;
  jobName: string;
  currentTime: number;
  totalDuration: number;
}
