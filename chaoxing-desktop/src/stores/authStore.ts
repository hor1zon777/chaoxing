import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

interface LoginResult {
  success: boolean;
  message: string;
  uid?: string | null;
}

export interface SavedAccountSummary {
  accountId: string;
  displayName: string;
  uid?: string | null;
  loginType: string;
  lastUsedAt: string;
}

/** 认证状态 */
interface AuthState {
  /** 当前认证请求代次 */
  authVersion: number;
  /** 是否已登录 */
  isLoggedIn: boolean;
  /** 用户名 */
  username: string;
  /** 是否加载中 */
  isLoading: boolean;
  /** 是否已完成启动认证初始化 */
  hasInitialized: boolean;
  /** 错误信息 */
  error: string | null;
  /** 提示信息 */
  notice: string | null;
  /** 已保存账号列表 */
  savedAccounts: SavedAccountSummary[];
  /** 账号密码登录 */
  login: (username: string, password: string) => Promise<void>;
  /** Cookie 登录 */
  loginWithCookies: (cookiesText: string) => Promise<void>;
  /** 使用已保存账户登录 */
  loginWithSavedAccount: (accountId: string) => Promise<void>;
  /** 获取已保存账号列表 */
  fetchSavedAccounts: () => Promise<void>;
  /** 删除已保存账户 */
  deleteSavedAccount: (accountId: string) => Promise<void>;
  /** 登出 */
  logout: () => Promise<void>;
  /** 验证当前活动会话是否有效 */
  validateSession: () => Promise<void>;
  /** 设置提示信息 */
  setNotice: (notice: string | null) => void;
  /** 清除错误信息 */
  clearError: () => void;
  /** 重置认证状态 */
  reset: () => void;
}

function resolveDisplayName(result: LoginResult, fallback: string) {
  return result.message || (result.uid ? `${fallback} (${result.uid})` : fallback);
}

function updateSavedAccounts(
  savedAccounts: SavedAccountSummary[],
  accountId: string,
): SavedAccountSummary[] {
  return savedAccounts.filter((account) => account.accountId !== accountId);
}

export const useAuthStore = create<AuthState>((set) => ({
  isLoggedIn: false,
  username: "",
  isLoading: false,
  hasInitialized: false,
  error: null,
  notice: null,
  authVersion: 0,
  savedAccounts: [],

  login: async (username: string, password: string) => {
    const nextVersion = useAuthStore.getState().authVersion + 1;
    set({ isLoading: true, error: null, notice: null, authVersion: nextVersion });
    try {
      const result = await invoke<LoginResult>("login", { username, password });
      if (useAuthStore.getState().authVersion !== nextVersion) {
        return;
      }
      if (!result.success) {
        set({ isLoading: false, isLoggedIn: false, error: result.message });
        return;
      }
      const savedAccounts = await invoke<SavedAccountSummary[]>("list_saved_accounts");
      set({
        isLoggedIn: true,
        username: resolveDisplayName(result, username),
        isLoading: false,
        hasInitialized: true,
        savedAccounts,
      });
    } catch (err) {
      if (useAuthStore.getState().authVersion !== nextVersion) {
        return;
      }
      const message = err instanceof Error ? err.message : String(err);
      set({ isLoading: false, error: message, isLoggedIn: false });
    }
  },

  loginWithCookies: async (cookiesText: string) => {
    const nextVersion = useAuthStore.getState().authVersion + 1;
    set({ isLoading: true, error: null, notice: null, authVersion: nextVersion });
    try {
      const result = await invoke<LoginResult>("login_with_cookies", { cookiesText });
      if (useAuthStore.getState().authVersion !== nextVersion) {
        return;
      }
      if (!result.success) {
        set({ isLoading: false, isLoggedIn: false, error: result.message });
        return;
      }
      const savedAccounts = await invoke<SavedAccountSummary[]>("list_saved_accounts");
      set({
        isLoggedIn: true,
        username: resolveDisplayName(result, "Cookie 登录"),
        isLoading: false,
        hasInitialized: true,
        savedAccounts,
      });
    } catch (err) {
      if (useAuthStore.getState().authVersion !== nextVersion) {
        return;
      }
      const message = err instanceof Error ? err.message : String(err);
      set({ isLoading: false, error: message, isLoggedIn: false });
    }
  },

  loginWithSavedAccount: async (accountId: string) => {
    const nextVersion = useAuthStore.getState().authVersion + 1;
    set({ isLoading: true, error: null, notice: null, authVersion: nextVersion });
    try {
      const result = await invoke<LoginResult>("login_with_saved_account", { accountId });
      if (useAuthStore.getState().authVersion !== nextVersion) {
        return;
      }
      if (!result.success) {
        set({ isLoading: false, isLoggedIn: false, error: result.message });
        return;
      }
      const savedAccounts = await invoke<SavedAccountSummary[]>("list_saved_accounts");
      set({
        isLoggedIn: true,
        username: resolveDisplayName(result, "已保存账户"),
        isLoading: false,
        hasInitialized: true,
        savedAccounts,
      });
    } catch (err) {
      if (useAuthStore.getState().authVersion !== nextVersion) {
        return;
      }
      const message = err instanceof Error ? err.message : String(err);
      set({ isLoading: false, error: message, isLoggedIn: false });
    }
  },

  fetchSavedAccounts: async () => {
    set({ isLoading: true, error: null });
    try {
      const savedAccounts = await invoke<SavedAccountSummary[]>("list_saved_accounts");
      set({
        savedAccounts,
        isLoading: false,
        hasInitialized: true,
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      set({
        savedAccounts: [],
        isLoading: false,
        hasInitialized: true,
        error: message,
      });
    }
  },

  deleteSavedAccount: async (accountId: string) => {
    try {
      const { savedAccounts, validateSession } = useAuthStore.getState();
      await invoke("delete_saved_account", { accountId });
      const nextAccounts = updateSavedAccounts(savedAccounts, accountId);
      set({ savedAccounts: nextAccounts });
      await validateSession();
      if (!useAuthStore.getState().isLoggedIn) {
        set({ notice: "已删除当前账号，请重新选择要登录的账户" });
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      set({ error: `删除账号失败: ${message}` });
    }
  },

  logout: async () => {
    try {
      await invoke("logout");
    } finally {
      set({
        isLoggedIn: false,
        username: "",
        error: null,
        notice: "已退出当前账号，请选择要登录的账户",
      });
      await useAuthStore.getState().fetchSavedAccounts();
    }
  },

  validateSession: async () => {
    const nextVersion = useAuthStore.getState().authVersion + 1;
    set({ isLoading: true, authVersion: nextVersion });
    try {
      const valid = await invoke<boolean>("validate_session");
      if (useAuthStore.getState().authVersion !== nextVersion) {
        return;
      }
      const currentUsername = useAuthStore.getState().username;
      set({
        isLoggedIn: valid,
        isLoading: false,
        hasInitialized: true,
        username: valid ? currentUsername || "当前活动会话" : "",
      });
    } catch {
      if (useAuthStore.getState().authVersion !== nextVersion) {
        return;
      }
      set({ isLoggedIn: false, isLoading: false, hasInitialized: true, username: "" });
    }
  },

  setNotice: (notice) => set({ notice }),

  clearError: () => set({ error: null }),

  reset: () =>
    set({
      isLoggedIn: false,
      username: "",
      isLoading: false,
      hasInitialized: true,
      error: null,
      notice: null,
      savedAccounts: [],
    }),
}));
