import { Layout, Menu, Button, Dropdown, Space, Typography, type MenuProps } from "antd";
import {
  BookOutlined,
  DownOutlined,
  PlayCircleOutlined,
  SettingOutlined,
  SwapOutlined,
  UserOutlined,
} from "@ant-design/icons";
import { Outlet, useNavigate, useLocation } from "react-router-dom";
import { useAuthStore } from "../../stores/authStore";
import { useCourseStore } from "../../stores/courseStore";
import { useTaskStore } from "../../stores/taskStore";

const { Sider, Content, Header } = Layout;
const { Title, Text } = Typography;

const menuItems: MenuProps["items"] = [
  {
    type: "group",
    label: "学习中心",
    children: [
      {
        key: "/courses",
        icon: <BookOutlined />,
        label: "课程列表",
      },
      {
        key: "/tasks",
        icon: <PlayCircleOutlined />,
        label: "任务执行",
      },
    ],
  },
  {
    type: "group",
    label: "系统设置",
    children: [
      {
        key: "/settings",
        icon: <SettingOutlined />,
        label: "设置",
      },
    ],
  },
];

function getPageTitle(pathname: string) {
  if (pathname.startsWith("/courses/") && pathname.endsWith("/tasks")) {
    return {
      title: "任务配置",
      subtitle: "按任务类型筛选、勾选学习范围并返回课程列表。",
    };
  }

  if (pathname.startsWith("/courses")) {
    return {
      title: "课程列表",
      subtitle: "",
    };
  }

  if (pathname.startsWith("/tasks")) {
    return {
      title: "任务执行",
      subtitle: "",
    };
  }

  return {
    title: "设置",
    subtitle: "",
  };
}

export function AppLayout() {
  const navigate = useNavigate();
  const location = useLocation();
  const username = useAuthStore((s) => s.username);
  const logout = useAuthStore((s) => s.logout);
  const courseReset = useCourseStore((s) => s.reset);
  const taskClearAll = useTaskStore((s) => s.clearAll);
  const isRunning = useTaskStore((s) => s.isRunning);

  const pageMeta = getPageTitle(location.pathname);

  const handleSwitchAccount = async () => {
    try {
      await logout();
    } finally {
      courseReset();
      taskClearAll();
      navigate("/login", { replace: true });
    }
  };

  const accountMenuItems: MenuProps["items"] = [
    {
      key: "status",
      disabled: true,
      label: (
        <Space size={8}>
          <span
            style={{
              width: 8,
              height: 8,
              borderRadius: "50%",
              background: "#52c41a",
              display: "inline-block",
            }}
          />
          <span>已登录</span>
        </Space>
      ),
    },
    {
      type: "divider",
    },
    {
      key: "switch-account",
      icon: <SwapOutlined />,
      label: isRunning ? "任务执行中，暂不可切换账号" : "切换账号",
      disabled: isRunning,
    },
  ];

  return (
    <Layout style={{ minHeight: "100vh" }}>
      <Sider
        width={236}
        theme="light"
        style={{
          borderRight: "1px solid #c8d8ee",
          background: "linear-gradient(180deg, #eaf2fb 0%, #e2edf9 100%)",
          boxShadow: "inset -1px 0 0 rgba(255,255,255,0.5)",
        }}
      >
        <div
          style={{
            padding: "20px 20px 16px",
            borderBottom: "1px solid #d2dfef",
            background: "linear-gradient(180deg, #eef5ff 0%, #e6effb 100%)",
          }}
        >
          <Title level={4} style={{ margin: 0, fontSize: 20 }}>
            超星助手
          </Title>
          <Text type="secondary" style={{ fontSize: 12 }}>
            课程配置与任务学习
          </Text>
        </div>

        <Menu
          mode="inline"
          selectedKeys={[
            location.pathname.startsWith("/courses/") ? "/courses" : location.pathname,
          ]}
          items={menuItems}
          onClick={({ key }) => navigate(key)}
          style={{ borderRight: 0, padding: "16px 12px", background: "transparent" }}
        />

        <div
          style={{
            margin: "0 12px 16px",
            height: 1,
            background: "linear-gradient(90deg, transparent 0%, #b7c7dd 18%, #b7c7dd 82%, transparent 100%)",
          }}
        />
      </Sider>

      <Layout>
        <Header
          style={{
            height: 88,
            padding: "0 24px",
            background: "linear-gradient(180deg, #f3f7fd 0%, #edf3fb 100%)",
            borderBottom: "1px solid #d6e2f0",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            gap: 16,
          }}
        >
          <div style={{ transform: "translateY(-8px)" }}>
            <Title level={3} style={{ margin: 0 }}>
              {pageMeta.title}
            </Title>
            {pageMeta.subtitle ? <Text type="secondary">{pageMeta.subtitle}</Text> : null}
          </div>

          <Dropdown
            trigger={["click"]}
            menu={{
              items: accountMenuItems,
              onClick: ({ key }) => {
                if (key === "switch-account") {
                  void handleSwitchAccount();
                }
              },
            }}
          >
            <Button
              type="text"
              style={{
                height: "auto",
                padding: "10px 14px",
                borderRadius: 16,
                border: "1px solid #d6e3f5",
                background: "linear-gradient(180deg, #ffffff 0%, #f7fbff 100%)",
                boxShadow: "0 12px 24px rgba(37,99,235,0.08)",
              }}
            >
              <Space size={12} align="center">
                <div
                  style={{
                    width: 36,
                    height: 36,
                    borderRadius: 12,
                    background: "linear-gradient(180deg, #dbeafe 0%, #bfdbfe 100%)",
                    color: "#1d4ed8",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    flexShrink: 0,
                  }}
                >
                  <UserOutlined />
                </div>
                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "flex-start",
                    lineHeight: 1.4,
                  }}
                >
                  <Text strong>{username || "当前账号"}</Text>
                  <Text type="secondary" style={{ fontSize: 12 }}>
                    {isRunning ? "任务执行中，暂不可切换账号" : "已登录"}
                  </Text>
                </div>
                <DownOutlined style={{ fontSize: 12, color: "#8c8c8c" }} />
              </Space>
            </Button>
          </Dropdown>
        </Header>

        <Content
          style={{
            padding: 24,
            background: "#edf3fb",
            overflow: "auto",
          }}
        >
          <div style={{ maxWidth: 1400, margin: "0 auto" }}>
            <Outlet />
          </div>
        </Content>
      </Layout>
    </Layout>
  );
}
