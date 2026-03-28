import { useEffect, useMemo, useState } from "react";
import {
  Card,
  Tabs,
  Form,
  Input,
  Button,
  Alert,
  Typography,
  message,
  Space,
  List,
  Tag,
  Popconfirm,
  Empty,
  Grid,
} from "antd";
import {
  UserOutlined,
  LockOutlined,
  DeleteOutlined,
  LoginOutlined,
  ClockCircleOutlined,
} from "@ant-design/icons";
import { useNavigate } from "react-router-dom";
import { useAuthStore } from "../stores/authStore";
import type { SavedAccountSummary } from "../stores/authStore";
import { useCourseStore } from "../stores/courseStore";
import { useTaskStore } from "../stores/taskStore";
import {
  GuideBanner,
  pageTopBannerTexts,
  panelStyle,
  primaryActionButtonStyle,
  SectionTitle,
  SummaryMetric,
  surfaceCardStyle,
} from "../components/Layout/AppLayout";

const { Title, Text, Paragraph } = Typography;
const { TextArea } = Input;
const { useBreakpoint } = Grid;

function getLoginTypeTagColor(loginType: string) {
  return loginType === "cookie" ? "gold" : "blue";
}

function renderSavedAccountMeta(account: SavedAccountSummary) {
  return (
    <Space size={[8, 8]} wrap>
      <Tag color={getLoginTypeTagColor(account.loginType)} style={{ marginInlineEnd: 0 }}>
        {account.loginType === "cookie" ? "Cookie" : "账号密码"}
      </Tag>
      {account.uid ? <Tag style={{ marginInlineEnd: 0 }}>{account.uid}</Tag> : null}
      <Text type="secondary">
        <ClockCircleOutlined style={{ marginRight: 6 }} />
        最近使用 {account.lastUsedAt}
      </Text>
    </Space>
  );
}

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
  const [activeTab, setActiveTab] = useState("password");
  const [pendingAccountId, setPendingAccountId] = useState<string | null>(null);
  const [deletingAccountId, setDeletingAccountId] = useState<string | null>(null);

  const isSavedAccountActionRunning = pendingAccountId !== null || deletingAccountId !== null;
  const isBusy = isLoading || isSavedAccountActionRunning;
  const isFormSubmitting = isLoading && !isSavedAccountActionRunning;

  const loginSummary = useMemo(
    () => [
      {
        label: "已保存账号",
        value: `${savedAccounts.length}`,
        hint: "可直接恢复历史登录会话",
      },
      {
        label: "当前方式",
        value: activeTab === "cookie" ? "Cookie" : "账号密码",
        hint: "登录方式可随时切换",
      },
      {
        label: "当前状态",
        value: isBusy ? "处理中" : "待登录",
        hint: isBusy ? "请稍候，正在处理账号操作" : "登录后进入课程页配置任务",
      },
    ],
    [savedAccounts.length, activeTab, isBusy],
  );

  useEffect(() => {
    if (isLoggedIn) {
      navigate("/courses", { replace: true });
    }
  }, [isLoggedIn, navigate]);

  const resetTaskRelatedState = () => {
    courseReset();
    taskClearAll();
  };

  const handlePasswordLogin = async (values: { username: string; password: string }) => {
    await login(values.username, values.password);
  };

  const handleCookieLogin = async (values: { cookies: string }) => {
    await loginWithCookies(values.cookies);
  };

  const handleSavedAccountLogin = async (accountId: string) => {
    setPendingAccountId(accountId);
    try {
      await loginWithSavedAccount(accountId);
      resetTaskRelatedState();
    } finally {
      setPendingAccountId(null);
    }
  };

  const handleDeleteSavedAccount = async (accountId: string) => {
    setDeletingAccountId(accountId);
    try {
      await deleteSavedAccount(accountId);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setNotice(null);
      clearError();
      message.error(`删除账号失败：${errorMessage}`);
    } finally {
      setDeletingAccountId(null);
    }
  };

  const handleTabChange = (key: string) => {
    if (isBusy) {
      return;
    }
    setActiveTab(key);
    clearError();
  };

  const handleNoticeClose = () => {
    setNotice(null);
  };

  return (
    <div
      style={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        minHeight: "100vh",
        padding: 24,
        background:
          "radial-gradient(circle at top, rgba(22,119,255,0.08), transparent 32%), #f3f6fb",
      }}
    >
      <Card
        style={{
          ...surfaceCardStyle,
          width: screens.lg ? 940 : 620,
          maxWidth: "100%",
        }}
        styles={{ body: { padding: screens.md ? 28 : 20 } }}
      >
        <div
          style={{
            display: "grid",
            gridTemplateColumns: screens.lg ? "minmax(0, 0.92fr) minmax(0, 1.08fr)" : "1fr",
            gap: 24,
          }}
        >
          <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
            <div>
              <Title level={2} style={{ margin: 0 }}>
                超星学习通
              </Title>
              <Paragraph type="secondary" style={{ margin: "8px 0 0", lineHeight: 1.75 }}>
                选择账号后登录，进入课程页继续配置学习内容。
              </Paragraph>
            </div>

            <GuideBanner text={pageTopBannerTexts.login} />

            <div
              style={{
                display: "grid",
                gridTemplateColumns: screens.sm ? "repeat(3, minmax(0, 1fr))" : "1fr",
                gap: 12,
              }}
            >
              {loginSummary.map((item) => (
                <SummaryMetric key={item.label} label={item.label} value={item.value} hint={item.hint} />
              ))}
            </div>

            {notice ? (
              <Alert
                message={notice}
                type="success"
                closable
                onClose={handleNoticeClose}
                style={{ borderRadius: 12 }}
              />
            ) : null}

            {error ? (
              <Alert
                message={error}
                type="error"
                closable
                onClose={clearError}
                style={{ borderRadius: 12 }}
              />
            ) : null}

            <Card
              size="small"
              style={{
                ...surfaceCardStyle,
                background: "linear-gradient(180deg, #ffffff 0%, #f8fbff 100%)",
                border: "1px solid #e8f1ff",
              }}
              styles={{ body: { padding: 18 } }}
            >
              <Space direction="vertical" size={12} style={{ width: "100%" }}>
                <Space direction="vertical" size={2} style={{ width: "100%" }}>
                  <Text strong>已保存账号</Text>
                  <Text type="secondary">可直接恢复历史登录会话，也可以删除失效账号。</Text>
                </Space>

                {savedAccounts.length > 0 ? (
                  <List
                    dataSource={savedAccounts}
                    split={false}
                    renderItem={(account) => {
                      const loginDisabled = isBusy && pendingAccountId !== account.accountId;
                      const deleteDisabled = isBusy && deletingAccountId !== account.accountId;

                      return (
                        <List.Item style={{ padding: 0, marginBottom: 12 }}>
                          <div
                            style={{
                              ...panelStyle,
                              width: "100%",
                              background: "#fff",
                              display: "flex",
                              justifyContent: "space-between",
                              alignItems: "flex-start",
                              gap: 16,
                              flexWrap: "wrap",
                            }}
                          >
                            <div style={{ flex: 1, minWidth: 220 }}>
                              <Text strong style={{ display: "block", fontSize: 15, marginBottom: 8 }}>
                                {account.displayName}
                              </Text>
                              {renderSavedAccountMeta(account)}
                            </div>
                            <Space wrap>
                              <Button
                                type="primary"
                                icon={<LoginOutlined />}
                                loading={pendingAccountId === account.accountId}
                                disabled={loginDisabled}
                                onClick={() => void handleSavedAccountLogin(account.accountId)}
                                style={{ borderRadius: 12 }}
                              >
                                登录
                              </Button>
                              <Popconfirm
                                title="确认删除该账号吗？"
                                description="删除后需要重新登录才能再次保存。"
                                okText="删除"
                                cancelText="取消"
                                okButtonProps={{ danger: true }}
                                disabled={deleteDisabled}
                                onConfirm={() => void handleDeleteSavedAccount(account.accountId)}
                              >
                                <Button
                                  danger
                                  icon={<DeleteOutlined />}
                                  loading={deletingAccountId === account.accountId}
                                  disabled={deleteDisabled}
                                  style={{ borderRadius: 12 }}
                                >
                                  删除
                                </Button>
                              </Popconfirm>
                            </Space>
                          </div>
                        </List.Item>
                      );
                    }}
                  />
                ) : (
                  <Empty
                    image={Empty.PRESENTED_IMAGE_SIMPLE}
                    description={screens.lg ? "暂无已保存账号，请先使用右侧方式登录" : "暂无已保存账号，请先使用下方方式登录"}
                  />
                )}
              </Space>
            </Card>
          </div>

          <Card
            style={{
              borderRadius: 18,
              border: "1px solid #eef2f6",
              boxShadow: "0 10px 28px rgba(15,23,42,0.05)",
            }}
            styles={{ body: { padding: 20 } }}
          >
            <div style={{ marginBottom: 16 }}>
              <SectionTitle
                title="新增账号"
                subtitle="选择登录方式并提交，登录成功后会自动保存账号，方便下次快速进入。"
              />
            </div>

            <Tabs
              activeKey={activeTab}
              onChange={handleTabChange}
              items={[
                {
                  key: "password",
                  label: "账号密码",
                  disabled: isBusy,
                  children: (
                    <Form onFinish={handlePasswordLogin} autoComplete="off" layout="vertical">
                      <Form.Item
                        label="手机号"
                        name="username"
                        rules={[{ required: true, message: "请输入手机号" }]}
                      >
                        <Input
                          prefix={<UserOutlined />}
                          placeholder="请输入手机号"
                          size="large"
                          disabled={isBusy}
                        />
                      </Form.Item>
                      <Form.Item
                        label="密码"
                        name="password"
                        rules={[{ required: true, message: "请输入密码" }]}
                      >
                        <Input.Password
                          prefix={<LockOutlined />}
                          placeholder="请输入密码"
                          size="large"
                          disabled={isBusy}
                        />
                      </Form.Item>
                      <div style={{ ...panelStyle, marginBottom: 16 }}>
                        <Text strong style={{ display: "block", marginBottom: 4 }}>
                          登录说明
                        </Text>
                        <Text type="secondary" style={{ fontSize: 12, lineHeight: 1.7 }}>
                          适合日常使用，登录成功后会自动保存当前账号，后续可在左侧历史账号区直接恢复。
                        </Text>
                      </div>
                      <Form.Item style={{ marginBottom: 0 }}>
                        <Button
                          type="primary"
                          htmlType="submit"
                          loading={isFormSubmitting}
                          disabled={isBusy}
                          block
                          size="large"
                          style={{ ...primaryActionButtonStyle, height: 44 }}
                        >
                          登录并保存账号
                        </Button>
                      </Form.Item>
                    </Form>
                  ),
                },
                {
                  key: "cookie",
                  label: "Cookie 登录",
                  disabled: isBusy,
                  children: (
                    <Form onFinish={handleCookieLogin} autoComplete="off" layout="vertical">
                      <Form.Item
                        label="Cookie"
                        name="cookies"
                        rules={[{ required: true, message: "请输入 Cookie" }]}
                      >
                        <TextArea
                          placeholder="粘贴完整 Cookie，支持多行"
                          rows={8}
                          disabled={isBusy}
                        />
                      </Form.Item>
                      <div style={{ ...panelStyle, marginBottom: 16 }}>
                        <Text strong style={{ display: "block", marginBottom: 4 }}>
                          登录说明
                        </Text>
                        <Text type="secondary" style={{ fontSize: 12, lineHeight: 1.7 }}>
                          适合已有浏览器会话的情况。粘贴完整 Cookie 后登录，系统会自动保存为可复用账号。
                        </Text>
                      </div>
                      <Form.Item style={{ marginBottom: 0 }}>
                        <Button
                          type="primary"
                          htmlType="submit"
                          loading={isFormSubmitting}
                          disabled={isBusy}
                          block
                          size="large"
                          style={{ ...primaryActionButtonStyle, height: 44 }}
                        >
                          登录并保存账号
                        </Button>
                      </Form.Item>
                    </Form>
                  ),
                },
              ]}
            />
          </Card>
        </div>
      </Card>
    </div>
  );
}

