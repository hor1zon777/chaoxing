import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { AddTopicResult, TopicListItem } from "../types/topic";

/** 批量发帖结果：results 为已成功发布的话题（按发布顺序），error 为遇到的首个错误（停在此处） */
export interface BatchSendResult {
  results: AddTopicResult[];
  error: string | null;
}

/** 批量发送时每条之间的间隔（毫秒），降低被平台风控的概率 */
const BATCH_DELAY_MS = 500;

const delay = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

// 列表拉取的单调递增请求序号：仅最新一次拉取的响应可写入 topics，
// 丢弃过期响应（如快速切换范围 / 刷新时乱序返回），避免旧数据覆盖当前视图。
let topicFetchSeq = 0;

/**
 * 讨论区状态。
 *
 * 话题为动态数据：每次进入 / 切换范围都重新拉取，不做跨页缓存
 * （离开页面调用 reset 清空；fetchTopics 起始即清空，避免切换时串显旧数据）。
 */
interface TopicState {
  /** 当前范围下的话题列表 */
  topics: TopicListItem[];
  /** 列表加载中 */
  isLoading: boolean;
  /** 发帖提交中（锁按钮、防重复提交） */
  submitting: boolean;
  /** 错误信息（仅供列表加载失败的页面级 Alert；写操作错误归弹窗 toast） */
  error: string | null;
  /**
   * 拉取话题列表。
   * mine=true 走 list_my_topics（groupweb getMyTopicList，服务端按登录用户过滤）；
   * mine=false 走 list_course_topics（groupyd，课程全部话题）。
   */
  fetchTopics: (courseId: string, clazzId: string, mine: boolean) => Promise<void>;
  /**
   * 批量发布话题（真实写操作）。顺序发布 count 条相同内容，每条之间间隔 BATCH_DELAY_MS，
   * 遇到第一个错误即停止。返回已成功发布的列表与首个错误（若有）。
   */
  sendTopicBatch: (
    courseId: string,
    clazzId: string,
    title: string,
    content: string,
    count: number,
    onProgress?: (done: number) => void,
  ) => Promise<BatchSendResult>;
  /** 将若干新话题乐观插入列表顶部（传入顺序即最终顺序，最新者应在数组前部） */
  prependTopics: (items: TopicListItem[]) => void;
  /** 清空状态（离开页面 / 切换课程 / 退出登录时调用） */
  reset: () => void;
}

export const useTopicStore = create<TopicState>((set) => ({
  topics: [],
  isLoading: false,
  submitting: false,
  error: null,

  fetchTopics: async (courseId, clazzId, mine) => {
    // 起始即清空旧 topics：切换课程 / 切换范围重新拉取时，避免加载期间继续渲染旧数据。
    const seq = (topicFetchSeq += 1);
    set({ topics: [], isLoading: true, error: null });
    try {
      const topics = mine
        ? await invoke<TopicListItem[]>("list_my_topics", { courseId })
        : await invoke<TopicListItem[]>("list_course_topics", { courseId, clazzId });
      // 仅当本次仍是最新拉取时才落库，丢弃被更晚一次拉取取代的过期响应
      if (seq !== topicFetchSeq) return;
      set({ topics, isLoading: false });
    } catch (err) {
      if (seq !== topicFetchSeq) return;
      const message = err instanceof Error ? err.message : String(err);
      set({ isLoading: false, error: message });
    }
  },

  sendTopicBatch: async (courseId, clazzId, title, content, count, onProgress) => {
    set({ submitting: true, error: null });
    const results: AddTopicResult[] = [];
    for (let i = 0; i < count; i += 1) {
      try {
        const result = await invoke<AddTopicResult>("send_topic", {
          courseId,
          clazzId,
          title,
          content,
        });
        results.push(result);
        onProgress?.(results.length);
      } catch (err) {
        // 遇错即停：写操作错误专由弹窗 toast 呈现，不写入 store.error（避免页面残留横幅）
        set({ submitting: false });
        return { results, error: err instanceof Error ? err.message : String(err) };
      }
      // 最后一条之后无需再等待
      if (i < count - 1) {
        await delay(BATCH_DELAY_MS);
      }
    }
    set({ submitting: false });
    return { results, error: null };
  },

  prependTopics: (items) =>
    set((state) => (items.length === 0 ? state : { topics: [...items, ...state.topics] })),

  reset: () => set({ topics: [], isLoading: false, submitting: false, error: null }),
}));
