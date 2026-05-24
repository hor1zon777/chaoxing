import { useEffect, type ReactNode } from "react";
import { HashRouter, Routes, Route, Navigate } from "react-router-dom";
import { ConfigProvider, Spin } from "antd";
import zhCN from "antd/locale/zh_CN";
import { useAuthStore } from "./stores/authStore";
import { AppLayout } from "./components/Layout/AppLayout";
import { LoginPage } from "./routes/LoginPage";
import { CoursesPage } from "./routes/CoursesPage";
import { CourseTaskSelectPage } from "./routes/CourseTaskSelectPage";
import { TaskPage } from "./routes/TaskPage";
import { SettingsPage } from "./routes/SettingsPage";
import { AboutPage } from "./routes/AboutPage";
import { PillButton } from "./components/ui/appleUI";

/** 路由守卫：未登录则跳转登录页 */
function ProtectedRoute({ children }: { children: ReactNode }) {
  const isLoggedIn = useAuthStore((s) => s.isLoggedIn);
  const isLoading = useAuthStore((s) => s.isLoading);
  const hasInitialized = useAuthStore((s) => s.hasInitialized);

  if (!hasInitialized || isLoading) {
    return (
      <div
        style={{
          minHeight: "100vh",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          background: "var(--apple-color-canvas)",
        }}
      >
        <Spin size="large" tip="正在加载已保存账号..." />
      </div>
    );
  }

  if (!isLoggedIn) {
    return <Navigate to="/login" replace />;
  }
  return <>{children}</>;
}

/** 404 页面 - Apple 风格 hero tile */
function NotFoundPage() {
  return (
    <div
      style={{
        minHeight: "100vh",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "var(--apple-color-canvas)",
        padding: 22,
        flexDirection: "column",
        textAlign: "center",
        gap: 30,
      }}
    >
      <span className="apple-eyebrow">404</span>
      <h1
        style={{
          fontFamily: "var(--apple-font-display)",
          fontSize: 56,
          fontWeight: 600,
          letterSpacing: "-0.28px",
          lineHeight: 1.07,
          margin: 0,
        }}
      >
        页面不存在。
      </h1>
      <p
        style={{
          fontFamily: "var(--apple-font-display)",
          fontSize: 28,
          fontWeight: 400,
          letterSpacing: "0.196px",
          lineHeight: 1.14,
          color: "var(--apple-color-ink-muted-80)",
          margin: 0,
          maxWidth: 520,
        }}
      >
        我们没能在这里找到任何内容。
      </p>
      <PillButton onClick={() => window.history.back()}>返回上一页</PillButton>
    </div>
  );
}

function App() {
  const fetchSavedAccounts = useAuthStore((s) => s.fetchSavedAccounts);
  const validateSession = useAuthStore((s) => s.validateSession);

  useEffect(() => {
    void (async () => {
      await validateSession();
      await fetchSavedAccounts();
    })();
  }, [fetchSavedAccounts, validateSession]);

  return (
    <ConfigProvider
      locale={zhCN}
      theme={{
        token: {
          colorPrimary: "#0066cc",
          colorInfo: "#0066cc",
          colorSuccess: "#1e7e34",
          colorWarning: "#a85d00",
          colorError: "#c8261d",
          colorLink: "#0066cc",
          colorLinkHover: "#0071e3",
          colorLinkActive: "#0066cc",
          borderRadius: 11,
          borderRadiusLG: 18,
          borderRadiusSM: 8,
          colorBgLayout: "#ffffff",
          colorBgContainer: "#ffffff",
          colorBgElevated: "#ffffff",
          colorBorder: "#e0e0e0",
          colorBorderSecondary: "#f0f0f0",
          colorText: "#1d1d1f",
          colorTextSecondary: "#7a7a7a",
          colorTextTertiary: "#7a7a7a",
          fontFamily:
            '"SF Pro Text", system-ui, -apple-system, "PingFang SC", "Microsoft YaHei", sans-serif',
          fontSize: 17,
          fontSizeLG: 17,
          fontSizeSM: 14,
          fontWeightStrong: 600,
          controlHeight: 40,
          controlHeightLG: 44,
          lineHeight: 1.47,
        },
        components: {
          Button: {
            borderRadius: 9999,
            controlHeight: 40,
            controlHeightLG: 44,
            fontWeight: 400,
            primaryShadow: "none",
            defaultShadow: "none",
            defaultBg: "#ffffff",
            defaultBorderColor: "#e0e0e0",
          },
          Card: {
            borderRadiusLG: 18,
            boxShadowTertiary: "none",
            colorBorderSecondary: "#e0e0e0",
            headerBg: "transparent",
            headerFontSize: 17,
          },
          Input: {
            borderRadius: 11,
            controlHeight: 44,
            activeBorderColor: "#0071e3",
            hoverBorderColor: "#0071e3",
          },
          Select: {
            borderRadius: 11,
            controlHeight: 44,
          },
          Tabs: {
            inkBarColor: "#0066cc",
            itemSelectedColor: "#0066cc",
            itemActiveColor: "#0066cc",
            itemHoverColor: "#0071e3",
            titleFontSize: 17,
          },
          Form: {
            labelFontSize: 14,
            verticalLabelPadding: "0 0 6px",
          },
          Switch: {
            colorPrimary: "#0066cc",
          },
          Tag: {
            borderRadiusSM: 9999,
          },
          Segmented: {
            borderRadius: 9999,
            trackBg: "rgba(0, 0, 0, 0.06)",
            itemSelectedBg: "#ffffff",
            itemSelectedColor: "#1d1d1f",
          },
          Slider: {
            railBg: "rgba(0, 0, 0, 0.10)",
            railHoverBg: "rgba(0, 0, 0, 0.18)",
            trackBg: "#0066cc",
            trackHoverBg: "#0071e3",
            handleColor: "#0066cc",
            handleActiveColor: "#0071e3",
          },
          Empty: {
            colorTextDescription: "#7a7a7a",
          },
          Progress: {
            defaultColor: "#0066cc",
          },
          Alert: {
            borderRadiusLG: 11,
          },
          Modal: {
            borderRadiusLG: 18,
          },
        },
      }}
    >
      <HashRouter>
        <Routes>
          <Route path="/login" element={<LoginPage />} />
          <Route
            path="/"
            element={
              <ProtectedRoute>
                <AppLayout />
              </ProtectedRoute>
            }
          >
            <Route index element={<Navigate to="/courses" replace />} />
            <Route path="courses" element={<CoursesPage />} />
            <Route path="courses/:courseId/tasks" element={<CourseTaskSelectPage />} />
            <Route path="tasks" element={<TaskPage />} />
            <Route path="settings" element={<SettingsPage />} />
            <Route path="about" element={<AboutPage />} />
          </Route>
          <Route path="*" element={<NotFoundPage />} />
        </Routes>
      </HashRouter>
    </ConfigProvider>
  );
}

export default App;
