import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "../types/config";

interface ConfigState {
  config: AppConfig;
  loading: boolean;
  error: string | null;
  loadConfig: () => Promise<void>;
  saveConfig: (config: AppConfig) => Promise<void>;
  updateConfig: (partial: Partial<AppConfig>) => void;
  importIni: (path: string) => Promise<void>;
}

const defaultConfig: AppConfig = {
  speed: 1.0,
  jobs: 4,
  tasksPerChapter: 1,
  chaptersPerCourse: 1,
  notopenAction: "retry",
  tikuProvider: "",
  tikuTokens: "",
  tikuSubmit: false,
  tikuCoverRate: 0.9,
  tikuDelay: 1.0,
  aiEndpoint: "",
  aiKey: "",
  aiModel: "",
  aiProxy: "",
  aiMinInterval: 3,
  siliconflowKey: "",
  siliconflowModel: "deepseek-ai/DeepSeek-R1",
  siliconflowEndpoint: "https://api.siliconflow.cn/v1/chat/completions",
  likeSearch: false,
  likeVision: true,
  likeModel: "glm-4.5-air",
  likeRetry: true,
  likeRetryTimes: 3,
  tikuAdapterUrl: "",
  trueList: "正确,对,√,是",
  falseList: "错误,错,×,否,不对,不正确",
  notificationProvider: "",
  notificationUrl: "",
  notificationTgChatId: "",
  checkLlmConnection: true,
};

export const useConfigStore = create<ConfigState>((set, get) => ({
  config: { ...defaultConfig },
  loading: false,
  error: null,

  loadConfig: async () => {
    set({ loading: true, error: null });
    try {
      const config = await invoke<AppConfig>("load_config");
      set({ config, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  saveConfig: async (config: AppConfig) => {
    set({ loading: true, error: null });
    try {
      await invoke("save_config", { config });
      set({ config, loading: false });
    } catch (e) {
      const errorMessage = String(e);
      set({ error: errorMessage, loading: false });
      throw new Error(errorMessage);
    }
  },

  updateConfig: (partial: Partial<AppConfig>) => {
    const current = get().config;
    set({ config: { ...current, ...partial } });
  },

  importIni: async (path: string) => {
    set({ loading: true, error: null });
    try {
      const config = await invoke<AppConfig>("import_ini", { path });
      set({ config, loading: false });
    } catch (e) {
      const errorMessage = String(e);
      set({ error: errorMessage, loading: false });
      throw new Error(errorMessage);
    }
  },
}));
