import { Layout, Menu } from "antd";
import {
  BookOutlined,
  PlayCircleOutlined,
  SettingOutlined,
} from "@ant-design/icons";
import { Outlet, useNavigate, useLocation } from "react-router-dom";

const { Sider, Content } = Layout;

/** 侧边栏菜单项 */
const menuItems = [
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
  {
    key: "/settings",
    icon: <SettingOutlined />,
    label: "设置",
  },
];

/** 应用主布局：侧边栏导航 + 内容区 */
export function AppLayout() {
  const navigate = useNavigate();
  const location = useLocation();

  return (
    <Layout style={{ minHeight: "100vh" }}>
      <Sider width={200} theme="light">
        <div
          style={{
            height: 48,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontWeight: 600,
            fontSize: 16,
            borderBottom: "1px solid #f0f0f0",
          }}
        >
          超星助手
        </div>
        <Menu
          mode="inline"
          selectedKeys={[location.pathname]}
          items={menuItems}
          onClick={({ key }) => navigate(key)}
          style={{ borderRight: 0 }}
        />
      </Sider>
      <Layout>
        <Content style={{ padding: 24, background: "#f5f5f5" }}>
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  );
}
