import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

/** 认证状态 */
interface AuthState {
  /** 是否已登录 */
  isLoggedIn: boolean;
  /** 用户名 */
  username: string;
  /** 是否加载中 */
  isLoading: boolean;
  /** 错误信息 */
  error: string | null;
  /** 账号密码登录 */
  login: (username: string, password: string) => Promise<void>;
  /** Cookie 登录 */
  loginWithCookies: (cookiesText: string) => Promise<void>;
  /** 登出 */
  logout: () => Promise<void>;
  /** 验证会话有效性 */
  validateSession: () => Promise<void>;
  /** 清除错误信息 */
  clearError: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  isLoggedIn: false,
  username: "",
  isLoading: false,
  error: null,

  login: async (username: string, password: string) => {
    set({ isLoading: true, error: null });
    try {
      await invoke("login", { username, password });
      set({ isLoggedIn: true, username, isLoading: false });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      set({ isLoading: false, error: message });
    }
  },

  loginWithCookies: async (cookiesText: string) => {
    set({ isLoading: true, error: null });
    try {
      await invoke("login_with_cookies", { cookiesText });
      set({ isLoggedIn: true, username: "Cookie 用户", isLoading: false });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      set({ isLoading: false, error: message });
    }
  },

  logout: async () => {
    try {
      await invoke("logout");
    } finally {
      set({ isLoggedIn: false, username: "", error: null });
    }
  },

  validateSession: async () => {
    set({ isLoading: true });
    try {
      const valid = await invoke<boolean>("validate_session");
      set({ isLoggedIn: valid, isLoading: false });
    } catch {
      set({ isLoggedIn: false, isLoading: false });
    }
  },

  clearError: () => set({ error: null }),
}));
