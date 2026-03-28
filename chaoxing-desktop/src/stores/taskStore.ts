import { create } from "zustand";
import type {
  TaskEvent,
  LogEntry,
  CourseProgress,
  VideoProgressInfo,
} from "../types/task";

/** 日志上限，超过后截断前半部分 */
const MAX_LOGS = 5000;

/** 任务状态 */
interface TaskState {
  /** 是否正在运行 */
  isRunning: boolean;
  /** 是否已暂停 */
  isPaused: boolean;
  /** 各课程进度 (courseId -> CourseProgress) */
  courseProgress: Record<string, CourseProgress>;
  /** 各视频播放进度 (jobId -> VideoProgressInfo) */
  videoProgress: Record<string, VideoProgressInfo>;
  /** 日志列表 */
  logs: LogEntry[];
  /** 日志过滤级别 */
  logFilter: "all" | "info" | "warn" | "error";

  /** 处理后端推送的任务事件 */
  handleTaskEvent: (event: TaskEvent) => void;
  /** 添加一条日志 */
  addLog: (level: string, message: string) => void;
  /** 设置日志过滤级别 */
  setLogFilter: (filter: "all" | "info" | "warn" | "error") => void;
  /** 清空日志 */
  clearLogs: () => void;
  /** 设置运行状态 */
  setRunning: (running: boolean) => void;
  /** 设置暂停状态 */
  setPaused: (paused: boolean) => void;
  /** 重置全部状态 */
  reset: () => void;
  /** 彻底清空任务状态 */
  clearAll: () => void;
}

export const useTaskStore = create<TaskState>((set, get) => ({
  isRunning: false,
  isPaused: false,
  courseProgress: {},
  videoProgress: {},
  logs: [],
  logFilter: "all",

  handleTaskEvent: (event: TaskEvent) => {
    const { courseProgress, addLog } = get();

    switch (event.type) {
      case "courseStarted": {
        set({
          courseProgress: {
            ...courseProgress,
            [event.courseId]: {
              courseId: event.courseId,
              courseTitle: event.courseTitle,
              completedChapters: 0,
              totalChapters: event.totalChapters,
              status: "running",
            },
          },
        });
        addLog(
          "info",
          `开始学习: ${event.courseTitle} (${event.totalChapters} 章节)`,
        );
        break;
      }

      case "courseCompleted": {
        const prev = courseProgress[event.courseId];
        if (prev) {
          set({
            courseProgress: {
              ...courseProgress,
              [event.courseId]: {
                ...prev,
                status: "completed",
                completedChapters: prev.totalChapters,
              },
            },
          });
        }
        addLog("info", `课程完成: ${event.courseTitle}`);
        break;
      }

      case "courseError": {
        const prev = courseProgress[event.courseId];
        if (prev) {
          set({
            courseProgress: {
              ...courseProgress,
              [event.courseId]: { ...prev, status: "error" },
            },
          });
        }
        addLog("error", `课程出错: ${event.courseTitle} - ${event.error}`);
        break;
      }

      case "chapterStarted": {
        addLog(
          "info",
          `章节开始: ${event.chapterTitle} (${event.jobCount} 个任务)`,
        );
        break;
      }

      case "chapterCompleted": {
        const prev = courseProgress[event.courseId];
        if (prev) {
          set({
            courseProgress: {
              ...courseProgress,
              [event.courseId]: {
                ...prev,
                completedChapters: prev.completedChapters + 1,
              },
            },
          });
        }
        addLog("info", `章节完成: ${event.chapterTitle}`);
        break;
      }

      case "chapterSkipped": {
        const prev = courseProgress[event.courseId];
        if (prev) {
          set({
            courseProgress: {
              ...courseProgress,
              [event.courseId]: {
                ...prev,
                completedChapters: prev.completedChapters + 1,
              },
            },
          });
        }
        addLog("warn", `章节跳过: ${event.chapterTitle} (${event.reason})`);
        break;
      }

      case "chapterRetrying": {
        addLog("warn", `章节重试: ${event.chapterTitle}`);
        break;
      }

      case "jobStarted": {
        addLog("info", `任务开始: ${event.jobName}`);
        break;
      }

      case "jobCompleted": {
        // 任务完成后移除对应的视频进度
        const { videoProgress } = get();
        const { [event.jobId]: _removed, ...restVideoProgress } =
          videoProgress;
        set({ videoProgress: restVideoProgress });
        addLog("info", `任务完成: ${event.jobName}`);
        break;
      }

      case "jobFailed": {
        addLog("error", `任务失败: ${event.jobName} - ${event.error}`);
        break;
      }

      case "videoProgress": {
        const { videoProgress } = get();
        set({
          videoProgress: {
            ...videoProgress,
            [event.jobId]: {
              jobId: event.jobId,
              jobName: event.jobName,
              currentTime: event.currentTime,
              totalDuration: event.totalDuration,
            },
          },
        });
        break;
      }

      case "liveProgress": {
        const { videoProgress } = get();
        set({
          videoProgress: {
            ...videoProgress,
            [event.jobId]: {
              jobId: event.jobId,
              jobName: event.jobName,
              currentTime: event.currentMinute,
              totalDuration: event.totalMinutes,
            },
          },
        });
        break;
      }

      case "workQuestionAnswered": {
        addLog("info", `答题: ${event.questionTitle} -> ${event.answer}`);
        break;
      }

      case "workSubmitted": {
        addLog(
          "info",
          `作业提交: ${event.chapterTitle} (覆盖率: ${event.coverRate.toFixed(0)}%, ${event.submitted ? "已提交" : "已保存"})`,
        );
        break;
      }

      case "log": {
        addLog(event.level, event.message);
        break;
      }

      case "allTasksCompleted": {
        set({ isRunning: false });
        addLog("info", "所有课程学习任务已完成");
        break;
      }
    }
  },

  addLog: (level: string, message: string) => {
    const timestamp = new Date().toLocaleTimeString("zh-CN", {
      hour12: false,
    });
    set((state) => {
      const newLogs = [...state.logs, { level, message, timestamp }];
      if (newLogs.length > MAX_LOGS) {
        return { logs: newLogs.slice(newLogs.length - MAX_LOGS) };
      }
      return { logs: newLogs };
    });
  },

  setLogFilter: (filter) => set({ logFilter: filter }),

  clearLogs: () => set({ logs: [] }),

  setRunning: (running) =>
    set((state) => ({
      isRunning: running,
      videoProgress: running ? state.videoProgress : {},
    })),

  setPaused: (paused) => set({ isPaused: paused }),

  reset: () =>
    set({
      isRunning: false,
      isPaused: false,
      courseProgress: {},
      videoProgress: {},
      logs: [],
    }),

  clearAll: () =>
    set({
      isRunning: false,
      isPaused: false,
      courseProgress: {},
      videoProgress: {},
      logs: [],
      logFilter: "all",
    }),
}));
