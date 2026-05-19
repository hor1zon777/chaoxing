import { useEffect, useState } from "react";
import { Alert, Form, Input, Popconfirm, Empty, Grid, message } from "antd";
import { useNavigate } from "react-router-dom";
import { useAuthStore } from "../stores/authStore";
import { useCourseStore } from "../stores/courseStore";
import { useTaskStore } from "../stores/taskStore";
import { Card, Chip, Eyebrow, PillButton, Tag } from "../components/ui/appleUI";

const { TextArea } = Input;
const { useBreakpoint } = Grid;

type LoginMode = "password" | "cookie";

export function LoginPage() {
  const screens = useBreakpoint();
  const navigate = useNavigate();
  const courseReset = useCourseStore((s) => s.reset);
  const taskClearAll = useTaskStore((s) => s.clearAll);
  const {
    isLoading,
    isLoggedIn,
    error,
    notice,
    savedAccounts,
    login,
    loginWithCookies,
    loginWithSavedAccount,
    deleteSavedAccount,
    clearError,
    setNotice,
  } = useAuthStore();

  const [mode, setMode] = useState<LoginMode>("password");
  const [pendingAccountId, setPendingAccountId] = useState<string | null>(null);
  const [deletingAccountId, setDeletingAccountId] = useState<string | null>(null);

  const isBusy = isLoading || pendingAccountId !== null || deletingAccountId !== null;
  const isFormSubmitting = isLoading && pendingAccountId === null && deletingAccountId === null;
  const isWide = screens.lg;

  const savedAccountListMaxHeight = screens.lg ? "calc(100vh - 280px)" : "none";

  useEffect(() => {
    if (isLoggedIn) navigate("/courses", { replace: true });
  }, [isLoggedIn, navigate]);

  const resetTaskState = () => {
    courseReset();
    taskClearAll();
  };

  const handlePasswordLogin = async (values: { username: string; password: string }) => {
    await login(values.username, values.password);
    resetTaskState();
  };

  const handleCookieLogin = async (values: { cookies: string }) => {
    await loginWithCookies(values.cookies);
    resetTaskState();
  };

  const handleSavedAccountLogin = async (accountId: string) => {
    setPendingAccountId(accountId);
    try {
      await loginWithSavedAccount(accountId);
      resetTaskState();
    } finally {
      setPendingAccountId(null);
    }
  };

  const handleDeleteSavedAccount = async (accountId: string) => {
    setDeletingAccountId(accountId);
    try {
      await deleteSavedAccount(accountId);
    } catch (err) {
      const errMessage = err instanceof Error ? err.message : String(err);
      message.error(`删除账号失败：${errMessage}`);
    } finally {
      setDeletingAccountId(null);
    }
  };

  const switchMode = (next: LoginMode) => {
    if (isBusy) return;
    setMode(next);
    clearError();
  };

  return (
    <div
      style={{
        minHeight: "100vh",
        display: "flex",
        background: "var(--apple-color-canvas)",
      }}
    >
      {/* 左侧：品牌 + 已保存账号 */}
      <div
        style={{
          flex: isWide ? "0 0 52%" : "1 1 auto",
          display: "flex",
          flexDirection: "column",
          background: "var(--apple-color-canvas-parchment)",
          borderRight: isWide ? "1px solid var(--apple-color-divider-soft)" : "none",
          minWidth: 0,
        }}
      >
        {/* 品牌区 */}
        <div style={{ padding: isWide ? "48px 48px 0" : "32px 24px 0" }}>
          <Eyebrow>超星学习通助手</Eyebrow>
          <h1
            style={{
              fontFamily: "var(--apple-font-display)",
              fontSize: isWide ? 56 : 40,
              fontWeight: 600,
              letterSpacing: isWide ? "-0.28px" : 0,
              lineHeight: 1.07,
              margin: "12px 0 0",
              color: "var(--apple-color-ink)",
            }}
          >
            登录，
            <br />
            开始学习。
          </h1>
          <p
            style={{
              fontFamily: "var(--apple-font-display)",
              fontSize: isWide ? 24 : 17,
              fontWeight: 300,
              letterSpacing: 0,
              lineHeight: 1.5,
              color: "var(--apple-color-ink-muted-80)",
              margin: "18px 0 0",
              maxWidth: 460,
            }}
          >
            一键恢复已保存账号，或新增登录方式。
          </p>
        </div>

        {/* 已保存账号 */}
        <div style={{ padding: isWide ? "36px 48px 24px" : "28px 24px 16px", flex: 1, display: "flex", flexDirection: "column", minHeight: 0 }}>
          <div style={{ marginBottom: 16 }}>
            <span
              style={{
                fontFamily: "var(--apple-font-text)",
                fontSize: 14,
                fontWeight: 600,
                letterSpacing: "-0.224px",
                color: "var(--apple-color-ink-muted-80)",
              }}
            >
              已保存账号 · {savedAccounts.length}
            </span>
          </div>

          {notice ? (
            <Alert
              message={notice}
              type="success"
              closable
              onClose={() => setNotice(null)}
              style={{ borderRadius: 11, marginBottom: 14, fontSize: 14 }}
            />
          ) : null}

          {savedAccounts.length > 0 ? (
            <div
              style={{
                flex: 1,
                overflowY: "auto",
                display: "grid",
                gridTemplateColumns: isWide ? (savedAccounts.length === 1 ? "1fr" : "1fr 1fr") : "1fr",
                gap: 14,
                alignContent: "start",
                maxHeight: savedAccountListMaxHeight,
                paddingRight: 4,
              }}
            >
              {savedAccounts.map((account) => {
                const loginDisabled = isBusy && pendingAccountId !== account.accountId;
                const deleteDisabled = isBusy && deletingAccountId !== account.accountId;

                return (
                  <Card
                    key={account.accountId}
                    padding={isWide ? 20 : 16}
                    selected={pendingAccountId === account.accountId}
                  >
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: 8, marginBottom: 8 }}>
                      <div
                        style={{
                          fontFamily: "var(--apple-font-display)",
                          fontSize: isWide ? 17 : 15,
                          fontWeight: 600,
                          letterSpacing: "-0.374px",
                          color: "var(--apple-color-ink)",
                          wordBreak: "break-all",
                          lineHeight: 1.3,
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          display: "-webkit-box",
                          WebkitBoxOrient: "vertical",
                          WebkitLineClamp: 1,
                        }}
                      >
                        {account.displayName}
                      </div>
                      <Tag tone={account.loginType === "cookie" ? "warning" : "default"}>
                        {account.loginType === "cookie" ? "Cookie" : "密码"}
                      </Tag>
                    </div>

                    <div
                      style={{
                        fontFamily: "var(--apple-font-text)",
                        fontSize: 12,
                        letterSpacing: "-0.12px",
                        color: "var(--apple-color-ink-muted-48)",
                        marginBottom: 14,
                      }}
                    >
                      最近使用 · {account.lastUsedAt}
                      {account.uid ? ` · UID ${account.uid}` : ""}
                    </div>

                    <div style={{ display: "flex", gap: 10, flexWrap: "wrap" }}>
                      <PillButton
                        onClick={() => void handleSavedAccountLogin(account.accountId)}
                        disabled={loginDisabled}
                      >
                        {pendingAccountId === account.accountId ? "登录中…" : "登录"}
                      </PillButton>
                      <Popconfirm
                        title="确认删除该账号吗？"
                        description="删除后需要重新登录。"
                        okText="删除"
                        cancelText="取消"
                        okButtonProps={{ danger: true }}
                        disabled={deleteDisabled}
                        onConfirm={() => void handleDeleteSavedAccount(account.accountId)}
                      >
                        <PillButton variant="ghost" disabled={deleteDisabled}>
                          {deletingAccountId === account.accountId ? "…" : "删除"}
                        </PillButton>
                      </Popconfirm>
                    </div>
                  </Card>
                );
              })}
            </div>
          ) : (
            <div style={{ flex: 1, display: "flex", alignItems: "center" }}>
              <Card padding={24} style={{ width: "100%" }}>
                <Empty
                  image={Empty.PRESENTED_IMAGE_SIMPLE}
                  description="暂无保存的账号，使用右侧表单登录后将自动保存"
                />
              </Card>
            </div>
          )}
        </div>
      </div>

      {/* 右侧：登录表单 */}
      <div
        style={{
          flex: isWide ? "0 0 48%" : "1 1 auto",
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          background: "var(--apple-color-canvas)",
          padding: isWide ? "48px 48px" : "32px 24px",
          borderTop: isWide ? "none" : "1px solid var(--apple-color-divider-soft)",
          minWidth: 0,
        }}
      >
        <div style={{ maxWidth: 440, width: "100%", margin: "0 auto" }}>
          <div style={{ marginBottom: 28 }}>
            <span
              style={{
                fontFamily: "var(--apple-font-text)",
                fontSize: 14,
                fontWeight: 600,
                letterSpacing: "-0.224px",
                color: "var(--apple-color-ink-muted-80)",
                display: "block",
                marginBottom: 16,
              }}
            >
              新增账号
            </span>

            {/* Mode chips */}
            <div style={{ display: "flex", gap: 10, flexWrap: "wrap", marginBottom: 28 }}>
              <Chip selected={mode === "password"} onClick={() => switchMode("password")} disabled={isBusy}>
                账号密码
              </Chip>
              <Chip selected={mode === "cookie"} onClick={() => switchMode("cookie")} disabled={isBusy}>
                Cookie 登录
              </Chip>
            </div>

            {error ? (
              <Alert
                message={error}
                type="error"
                closable
                onClose={clearError}
                style={{ borderRadius: 11, marginBottom: 20, fontSize: 14 }}
              />
            ) : null}
          </div>

          {mode === "password" ? (
            <Form layout="vertical" onFinish={handlePasswordLogin} autoComplete="off" key="password" size="large">
              <Form.Item
                label="手机号"
                name="username"
                rules={[{ required: true, message: "请输入手机号" }]}
              >
                <Input
                  placeholder="请输入手机号"
                  disabled={isBusy}
                  style={{
                    borderRadius: 11,
                    fontSize: isWide ? 17 : 16,
                    height: isWide ? 48 : 44,
                  }}
                />
              </Form.Item>
              <Form.Item
                label="密码"
                name="password"
                rules={[{ required: true, message: "请输入密码" }]}
              >
                <Input.Password
                  placeholder="请输入密码"
                  disabled={isBusy}
                  style={{
                    borderRadius: 11,
                    fontSize: isWide ? 17 : 16,
                    height: isWide ? 48 : 44,
                  }}
                />
              </Form.Item>

              <div style={{ marginTop: 24 }}>
                <PillButton htmlType="submit" large={isWide} fullWidth disabled={isBusy}>
                  {isFormSubmitting ? "登录中…" : "登录并保存"}
                </PillButton>
              </div>
              <p
                style={{
                  fontFamily: "var(--apple-font-text)",
                  fontSize: 12,
                  letterSpacing: "-0.12px",
                  color: "var(--apple-color-ink-muted-48)",
                  marginTop: 14,
                  lineHeight: 1.5,
                }}
              >
                适合日常使用，登录成功后自动保存账号。
              </p>
            </Form>
          ) : (
            <Form layout="vertical" onFinish={handleCookieLogin} autoComplete="off" key="cookie" size="large">
              <Form.Item
                label="Cookie"
                name="cookies"
                rules={[{ required: true, message: "请输入 Cookie" }]}
              >
                <TextArea
                  placeholder="粘贴完整 Cookie，支持多行"
                  rows={isWide ? 10 : 7}
                  disabled={isBusy}
                  style={{
                    borderRadius: 11,
                    fontFamily: "var(--apple-font-text)",
                    fontSize: isWide ? 15 : 14,
                    resize: "vertical",
                  }}
                />
              </Form.Item>

              <div style={{ marginTop: 24 }}>
                <PillButton htmlType="submit" large={isWide} fullWidth disabled={isBusy}>
                  {isFormSubmitting ? "登录中…" : "登录并保存"}
                </PillButton>
              </div>
              <p
                style={{
                  fontFamily: "var(--apple-font-text)",
                  fontSize: 12,
                  letterSpacing: "-0.12px",
                  color: "var(--apple-color-ink-muted-48)",
                  marginTop: 14,
                  lineHeight: 1.5,
                }}
              >
                适合已有浏览器会话的情况，粘贴 Cookie 后登录保存。
              </p>
            </Form>
          )}
        </div>
      </div>
    </div>
  );
}
