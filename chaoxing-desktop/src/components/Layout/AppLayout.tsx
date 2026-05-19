import { useMemo, useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Outlet, useNavigate, useLocation } from "react-router-dom";
import { useAuthStore } from "../../stores/authStore";
import { useCourseStore } from "../../stores/courseStore";
import { useTaskStore } from "../../stores/taskStore";

const navLinks = [
  { key: "/courses", label: "课程" },
  { key: "/tasks", label: "任务" },
  { key: "/settings", label: "设置" },
];

interface PageMeta {
  title: string;
  description?: string;
}

function getPageMeta(pathname: string): PageMeta {
  if (pathname.startsWith("/courses/") && pathname.endsWith("/tasks")) {
    return { title: "任务配置", description: "按类型筛选并勾选学习范围" };
  }
  if (pathname.startsWith("/courses")) {
    return { title: "课程" };
  }
  if (pathname.startsWith("/tasks")) {
    return { title: "任务" };
  }
  return { title: "设置" };
}

export function AppLayout() {
  const navigate = useNavigate();
  const location = useLocation();
  const username = useAuthStore((s) => s.username);
  const logout = useAuthStore((s) => s.logout);
  const courseReset = useCourseStore((s) => s.reset);
  const taskClearAll = useTaskStore((s) => s.clearAll);
  const isRunning = useTaskStore((s) => s.isRunning);

  const pageMeta = useMemo(() => getPageMeta(location.pathname), [location.pathname]);
  const activeKey = useMemo(() => {
    if (location.pathname.startsWith("/courses")) return "/courses";
    if (location.pathname.startsWith("/tasks")) return "/tasks";
    if (location.pathname.startsWith("/settings")) return "/settings";
    return "";
  }, [location.pathname]);

  const [accountMenuOpen, setAccountMenuOpen] = useState(false);
  const accountMenuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!accountMenuOpen) return;
    const handler = (event: MouseEvent) => {
      if (accountMenuRef.current && !accountMenuRef.current.contains(event.target as Node)) {
        setAccountMenuOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [accountMenuOpen]);

  const handleSwitchAccount = async () => {
    setAccountMenuOpen(false);
    if (isRunning) return;
    try {
      await invoke("cancel_tasks");
    } catch { /* 无运行中任务时忽略 */ }
    try {
      await logout();
    } finally {
      courseReset();
      taskClearAll();
      navigate("/login", { replace: true });
    }
  };

  return (
    <div className="apple-app">
      {/* 白色顶栏 — 合并导航 + 页面标题 */}
      <header
        style={{
          position: "sticky",
          top: 0,
          zIndex: 100,
          background: "rgba(255, 255, 255, 0.84)",
          backdropFilter: "saturate(180%) blur(20px)",
          WebkitBackdropFilter: "saturate(180%) blur(20px)",
          borderBottom: "1px solid rgba(0, 0, 0, 0.08)",
        }}
      >
        <div
          style={{
            maxWidth: 1024,
            margin: "0 auto",
            padding: "0 22px",
            height: 44,
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            gap: 24,
          }}
        >
          {/* 左侧：应用名 + 页面标题 */}
          <div style={{ display: "flex", alignItems: "baseline", gap: 20 }}>
            <span
              style={{
                fontFamily: "var(--apple-font-display)",
                fontSize: 17,
                fontWeight: 600,
                letterSpacing: "-0.374px",
                color: "var(--apple-color-ink)",
              }}
            >
              超星助手
            </span>
            <h1
              style={{
                margin: 0,
                fontFamily: "var(--apple-font-display)",
                fontSize: 17,
                fontWeight: 600,
                letterSpacing: "-0.374px",
                color: "var(--apple-color-ink-muted-80)",
              }}
            >
              {pageMeta.title}
            </h1>
          </div>

          {/* 右侧：导航链接 + 账号 */}
          <div style={{ display: "flex", alignItems: "center", gap: 20 }}>
            {navLinks.map((link) => (
              <button
                key={link.key}
                type="button"
                onClick={() => navigate(link.key)}
                style={{
                  background: "none",
                  border: "none",
                  padding: 0,
                  fontFamily: "var(--apple-font-text)",
                  fontSize: 14,
                  fontWeight: 400,
                  letterSpacing: "-0.224px",
                  color: activeKey === link.key ? "var(--apple-color-ink)" : "var(--apple-color-ink-muted-48)",
                  cursor: "pointer",
                  transition: "color 160ms ease",
                }}
              >
                {link.label}
              </button>
            ))}

            <div ref={accountMenuRef} style={{ position: "relative" }}>
              <button
                type="button"
                onClick={() => setAccountMenuOpen((value) => !value)}
                aria-haspopup="menu"
                aria-expanded={accountMenuOpen}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 6,
                  background: "var(--apple-color-canvas-parchment)",
                  border: "1px solid var(--apple-color-hairline)",
                  borderRadius: 9999,
                  padding: "4px 14px",
                  fontFamily: "var(--apple-font-text)",
                  fontSize: 13,
                  letterSpacing: "-0.12px",
                  color: "var(--apple-color-ink-muted-80)",
                  cursor: "pointer",
                }}
              >
                <span
                  style={{
                    display: "inline-block",
                    width: 6,
                    height: 6,
                    borderRadius: "50%",
                    background: isRunning ? "#a85d00" : "#1e7e34",
                  }}
                />
                {username || "账号"}
              </button>
              {accountMenuOpen ? (
                <div
                  role="menu"
                  style={{
                    position: "absolute",
                    top: "calc(100% + 6px)",
                    right: 0,
                    minWidth: 200,
                    background: "var(--apple-color-canvas)",
                    border: "1px solid var(--apple-color-hairline)",
                    borderRadius: 11,
                    padding: 10,
                    boxShadow: "0 10px 32px rgba(0,0,0,0.12)",
                    zIndex: 200,
                  }}
                >
                  <div
                    style={{
                      padding: "8px 10px 10px",
                      borderBottom: "1px solid var(--apple-color-divider-soft)",
                      marginBottom: 6,
                    }}
                  >
                    <div
                      style={{
                        fontFamily: "var(--apple-font-text)",
                        fontSize: 14,
                        fontWeight: 600,
                        letterSpacing: "-0.224px",
                        color: "var(--apple-color-ink)",
                      }}
                    >
                      {username || "当前账号"}
                    </div>
                    <div
                      style={{
                        fontFamily: "var(--apple-font-text)",
                        fontSize: 12,
                        letterSpacing: "-0.12px",
                        color: "var(--apple-color-ink-muted-48)",
                        marginTop: 2,
                      }}
                    >
                      {isRunning ? "任务执行中" : "已登录"}
                    </div>
                  </div>
                  <button
                    type="button"
                    role="menuitem"
                    disabled={isRunning}
                    onClick={() => void handleSwitchAccount()}
                    style={{
                      width: "100%",
                      textAlign: "left",
                      padding: "8px 10px",
                      borderRadius: 8,
                      background: "transparent",
                      border: "none",
                      color: isRunning ? "var(--apple-color-ink-muted-48)" : "var(--apple-color-primary)",
                      fontFamily: "var(--apple-font-text)",
                      fontSize: 14,
                      letterSpacing: "-0.224px",
                      cursor: isRunning ? "not-allowed" : "pointer",
                    }}
                  >
                    {isRunning ? "任务执行中，暂不可切换" : "切换账号"}
                  </button>
                </div>
              ) : null}
            </div>
          </div>
        </div>
      </header>

      <main style={{ flex: 1, display: "flex", flexDirection: "column" }}>
        <Outlet />
      </main>
    </div>
  );
}
