import { useState } from "react";
import { Card, Tabs, Form, Input, Button, Alert, Typography } from "antd";
import { UserOutlined, LockOutlined } from "@ant-design/icons";
import { useNavigate } from "react-router-dom";
import { useAuthStore } from "../stores/authStore";

const { Title } = Typography;
const { TextArea } = Input;

/** 登录页面 */
export function LoginPage() {
  const navigate = useNavigate();
  const { isLoading, error, login, loginWithCookies, clearError } =
    useAuthStore();
  const [activeTab, setActiveTab] = useState("password");

  /** 账号密码登录 */
  const handlePasswordLogin = async (values: {
    username: string;
    password: string;
  }) => {
    await login(values.username, values.password);
    // 登录成功后 store 会更新 isLoggedIn
    if (useAuthStore.getState().isLoggedIn) {
      navigate("/courses");
    }
  };

  /** Cookie 登录 */
  const handleCookieLogin = async (values: { cookies: string }) => {
    await loginWithCookies(values.cookies);
    if (useAuthStore.getState().isLoggedIn) {
      navigate("/courses");
    }
  };

  /** 切换 Tab 时清除错误 */
  const handleTabChange = (key: string) => {
    setActiveTab(key);
    clearError();
  };

  return (
    <div
      style={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        minHeight: "100vh",
        background: "#f0f2f5",
      }}
    >
      <Card style={{ width: 420 }}>
        <Title level={3} style={{ textAlign: "center", marginBottom: 24 }}>
          超星学习通
        </Title>

        {error && (
          <Alert
            message={error}
            type="error"
            closable
            onClose={clearError}
            style={{ marginBottom: 16 }}
          />
        )}

        <Tabs
          activeKey={activeTab}
          onChange={handleTabChange}
          items={[
            {
              key: "password",
              label: "账号密码",
              children: (
                <Form onFinish={handlePasswordLogin} autoComplete="off">
                  <Form.Item
                    name="username"
                    rules={[{ required: true, message: "请输入手机号" }]}
                  >
                    <Input
                      prefix={<UserOutlined />}
                      placeholder="手机号"
                      size="large"
                    />
                  </Form.Item>
                  <Form.Item
                    name="password"
                    rules={[{ required: true, message: "请输入密码" }]}
                  >
                    <Input.Password
                      prefix={<LockOutlined />}
                      placeholder="密码"
                      size="large"
                    />
                  </Form.Item>
                  <Form.Item>
                    <Button
                      type="primary"
                      htmlType="submit"
                      loading={isLoading}
                      block
                      size="large"
                    >
                      登录
                    </Button>
                  </Form.Item>
                </Form>
              ),
            },
            {
              key: "cookie",
              label: "Cookie 登录",
              children: (
                <Form onFinish={handleCookieLogin} autoComplete="off">
                  <Form.Item
                    name="cookies"
                    rules={[{ required: true, message: "请输入 Cookie" }]}
                  >
                    <TextArea
                      placeholder="粘贴 Cookie 内容，支持多行"
                      rows={6}
                    />
                  </Form.Item>
                  <Form.Item>
                    <Button
                      type="primary"
                      htmlType="submit"
                      loading={isLoading}
                      block
                      size="large"
                    >
                      登录
                    </Button>
                  </Form.Item>
                </Form>
              ),
            },
          ]}
        />
      </Card>
    </div>
  );
}
