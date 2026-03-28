import { useEffect, type ReactNode } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { ConfigProvider, Result, Button, Spin } from "antd";
import { useAuthStore } from "./stores/authStore";
import { AppLayout } from "./components/Layout/AppLayout";
import { LoginPage } from "./routes/LoginPage";
import { CoursesPage } from "./routes/CoursesPage";
import { CourseTaskSelectPage } from "./routes/CourseTaskSelectPage";
import { TaskPage } from "./routes/TaskPage";
import { SettingsPage } from "./routes/SettingsPage";

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
          background: "#f0f2f5",
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

/** 404 页面 */
function NotFoundPage() {
  return (
    <Result
      status="404"
      title="404"
      subTitle="页面不存在"
      extra={
        <Button type="primary" onClick={() => window.history.back()}>
          返回
        </Button>
      }
    />
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
      theme={{
        token: {
          colorPrimary: "#1677ff",
          colorInfo: "#1677ff",
          colorSuccess: "#16a34a",
          colorWarning: "#d97706",
          colorError: "#dc2626",
          borderRadius: 12,
          colorBgLayout: "#f0f2f5",
          colorBgContainer: "#ffffff",
        },
      }}
    >
      <BrowserRouter>
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
          </Route>
          <Route path="*" element={<NotFoundPage />} />
        </Routes>
      </BrowserRouter>
    </ConfigProvider>
  );
}

export default App;
