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

export const surfaceCardStyle = {
  borderRadius: 18,
  boxShadow: "0 10px 28px rgba(15,23,42,0.05)",
  border: "1px solid #eef2f6",
};

export const panelCardStyle = {
  borderRadius: 16,
  border: "1px solid #edf2fa",
  background: "#fafcff",
};

export const panelStyle = {
  ...panelCardStyle,
  padding: "14px 16px",
};

export const primaryActionButtonStyle = {
  minWidth: 132,
  height: 42,
  borderRadius: 12,
  paddingInline: 20,
};

export const workflowCopy = {
  login: "先选择账号，再登录进入课程页。",
  courses: "先选课程，再进入任务配置。",
  courseTasks: "按任务类型勾选学习范围。",
  tasks: "确认范围后开始任务并查看进度。",
  settings: "在这里调整默认参数、题库和通知。",
};

export function GuideBanner({ text }: { text: string }) {
  return (
    <div
      style={{
        padding: "16px 18px",
        borderRadius: 18,
        border: "1px solid #dbeafe",
        background: "linear-gradient(135deg, #eef4ff 0%, #f8fbff 100%)",
        boxShadow: "0 8px 24px rgba(22,119,255,0.06)",
      }}
    >
      <Text
        style={{
          display: "block",
          fontSize: 16,
          fontWeight: 600,
          lineHeight: 1.7,
          color: "#1f2937",
        }}
      >
        {text}
      </Text>
    </div>
  );
}

export function GuideLabel({ text }: { text: string }) {
  return (
    <Text
      style={{
        display: "block",
        fontSize: 12,
        fontWeight: 600,
        color: "#1677ff",
        marginBottom: 8,
      }}
    >
      {text}
    </Text>
  );
}

export function GuideText({ text }: { text: string }) {
  return (
    <Text
      type="secondary"
      style={{
        display: "block",
        fontSize: 13,
        lineHeight: 1.75,
      }}
    >
      {text}
    </Text>
  );
}

export const workflowBannerCopy = {
  login: "选择历史账号或新增登录方式，登录成功后即可进入课程页。",
  courses: "先从课程中挑选要学习的内容，再进入任务配置页细化范围。",
  courseTasks: "先筛选任务类型，再勾选章节与任务点，选择会自动保存。",
  tasks: "确认学习范围后开始任务，在这里查看进度、播放状态和运行日志。",
  settings: "在这里统一调整默认参数、题库连接与通知方式。",
};

export const pageHintText = {
  coursesAction: "点击课程卡片可直接进入任务配置",
};

export const footerCopy = {
  courses: "完成任务配置后即可前往任务执行页开始学习",
  courseTasks: "选择会自动保存，可直接返回课程列表，后续再前往任务执行页开始学习",
  settings: "配置文件保存在应用数据目录下的 config.json，可先导入 INI 再按当前界面进行调整。",
};

export const summaryLabelCopy = {
  courseFilter: "当前筛选",
  courseTaskFilter: "当前筛选摘要",
  settingsForm: "当前表单摘要",
};

export const actionBarCardStyle = {
  ...surfaceCardStyle,
  boxShadow: "0 -4px 18px rgba(15,23,42,0.04)",
};

export const stickyActionBarStyle = {
  ...actionBarCardStyle,
  position: "sticky" as const,
  bottom: 0,
};

export const actionBarBodyStyle = {
  padding: "16px 20px",
};

export const pageSectionGapStyle = {
  display: "flex",
  flexDirection: "column" as const,
  gap: 16,
};

export const pageLeadTitleStyle = {
  margin: 0,
};

export const pageLeadTextStyle = {
  margin: "8px 0 0",
  lineHeight: 1.7,
};

export const summaryGridStyle = (isWide: boolean) => ({
  display: "grid",
  gridTemplateColumns: isWide ? "repeat(4, minmax(0, 1fr))" : "repeat(2, minmax(0, 1fr))",
  gap: 12,
  marginTop: 20,
});

export const compactGuidePanelStyle = {
  ...panelStyle,
  padding: "10px 14px",
};

export const pageTopGroupStyle = {
  display: "flex",
  flexDirection: "column" as const,
  gap: 16,
};

export const sectionStackStyle = {
  display: "flex",
  flexDirection: "column" as const,
  gap: 16,
};

export const wrapBetweenStyle = {
  display: "flex",
  justifyContent: "space-between",
  alignItems: "flex-start",
  gap: 16,
  flexWrap: "wrap" as const,
};

export const inlineActionWrapStyle = {
  display: "flex",
  flexWrap: "wrap" as const,
  gap: 10,
};

export const cardBodyPadding24 = { body: { padding: 24 } };
export const cardBodyPadding20 = { body: { padding: 20 } };
export const cardBodyPadding18 = { body: { padding: 18 } };
export const actionCardBodyStyle = { body: actionBarBodyStyle };

export const primaryButtonWideStyle = {
  ...primaryActionButtonStyle,
  minWidth: 168,
};

export const primaryButtonCompactStyle = {
  ...primaryActionButtonStyle,
  minWidth: 120,
};

export const secondaryActionButtonStyle = {
  ...primaryActionButtonStyle,
  minWidth: 110,
};

export const narrowPanelMinWidth = 200;
export const mediumPanelMinWidth = 240;
export const filterInputMinWidth = 260;

export const helperTextStyle = {
  fontSize: 12,
};

export const headlineGuideTextStyle = {
  fontSize: 13,
};

export const pageContentMaxWidthStyle = {
  maxWidth: 780,
};

export const pageContentWideMaxWidthStyle = {
  maxWidth: 840,
};

export const leadTextStyle = {
  lineHeight: 1.7,
};

export const courseCardShadow = "0 10px 24px rgba(15,23,42,0.05)";
export const selectedCourseCardShadow = "0 14px 28px rgba(22,119,255,0.10)";

export const summaryAccentPanelStyle = {
  ...panelStyle,
  background: "linear-gradient(180deg, #f8fbff 0%, #ffffff 100%)",
};

export const savedAccountCardStyle = {
  ...surfaceCardStyle,
  background: "linear-gradient(180deg, #ffffff 0%, #f8fbff 100%)",
  border: "1px solid #e8f1ff",
};

export const accountItemPanelStyle = {
  ...panelStyle,
  background: "#fff",
};

export const topSummaryPanelStyle = {
  ...panelStyle,
  border: "1px solid #dceafe",
  background: "linear-gradient(180deg, #f8fbff 0%, #ffffff 100%)",
};

export const actionPrimaryText = {
  login: "登录并保存账号",
  courses: "开始学习",
  courseTasks: "返回课程列表",
  settings: "保存配置",
};

export const actionSecondaryText = {
  settingsImport: "导入 INI 配置",
};

export const pageGuideLabel = "操作引导";
export const pageSummaryLabel = "当前概览";
export const pageFormSummaryLabel = "当前表单";
export const pageSavedAccountsLabel = "已保存账号";
export const pageNewAccountLabel = "新增账号";
export const loginHelpLabel = "登录说明";
export const pageSelectionLabel = "当前筛选";
export const pageSelectionSummaryLabel = "当前筛选摘要";
export const settingsSummaryGuideText = "这里会实时反映表单中的最新输入，便于在保存前快速确认关键配置。";
export const savedAccountHelperText = "可直接恢复历史登录会话，也可以删除失效账号。";
export const newAccountHelperText = "选择登录方式并提交，登录成功后会自动保存账号，方便下次快速进入。";
export const passwordLoginHelpText = "适合日常使用，登录成功后会自动保存当前账号，后续可在左侧历史账号区直接恢复。";
export const cookieLoginHelpText = "适合已有浏览器会话的情况。粘贴完整 Cookie 后登录，系统会自动保存为可复用账号。";
export const courseTaskSummaryGuideText = "快捷选择会直接按类型重置当前课程的任务勾选结果。";
export const settingsSummaryGuideLabel = "当前表单摘要";
export const courseSelectionSummaryLabel = "当前筛选摘要";
export const settingsFileLocationLabel = "配置文件位置";
export const settingsFileLocationText = "配置文件保存在应用数据目录下的 config.json，可先导入 INI 再按当前界面进行调整。";
export const taskNoSelectionText = "尚未配置课程任务，请先前往课程页进入任务配置";
export const loginEmptyAccountsText = "暂无已保存账号，请先使用下方方式登录";
export const loginEmptyAccountsWideText = "暂无已保存账号，请先使用右侧方式登录";
export const savedAccountsTitle = "已保存账号";
export const newAccountTitle = "新增账号";
export const appSettingsTitle = "应用设置";
export const learningTaskConsoleTitle = "学习任务控制台";
export const courseSelectTitle = "选择课程并进入任务配置";
export const pageTaskConfigTitle = "任务配置";
export const coursesLeadTitle = "选择课程并进入任务配置";
export const settingsLeadTitle = "应用设置";
export const taskLeadTitle = "学习任务控制台";
export const loginLeadTitle = "超星学习通";
export const loginSavedAccountButtonText = "登录";
export const loginDeleteAccountButtonText = "删除";
export const settingsTestNotificationText = "发送测试通知";
export const settingsTestTikuText = "测试题库连接";
export const quickGuideTitle = "操作引导";
export const summaryGuideTitle = "当前概览";

export const topGuideSpacing = {
  marginBottom: 4,
};

export const emphasisTextStyle = {
  fontSize: 16,
  lineHeight: 1.7,
};

export const titleSpacingStyle = {
  margin: "12px 0 6px",
};

export const compactSummaryTextStyle = {
  display: "block",
  fontSize: 12,
  lineHeight: 1.7,
};

export const routePageGapStyle = {
  display: "flex",
  flexDirection: "column" as const,
  gap: 16,
};

export const guideBannerSpacingStyle = {
  marginBottom: 0,
};

export const pageGuideTitleText = "操作引导";
export const overviewGuideTitleText = "当前概览";

export const pageTopBannerStyle = {
  marginBottom: 0,
};

export const pageTitleWrapStyle = {
  maxWidth: 760,
};

export const pageWideTitleWrapStyle = {
  maxWidth: 840,
};

export const settingsSummaryPanelMinWidth = 250;

export const actionButtonBaseStyle = {
  borderRadius: 12,
};

export const savedAccountActionButtonStyle = {
  borderRadius: 12,
};

export const helperPanelMarginBottom = 16;

export const loginShellCardStyle = {
  ...surfaceCardStyle,
  borderRadius: 22,
  boxShadow: "0 20px 56px rgba(22,119,255,0.12)",
};

export const helperBannerPanelStyle = {
  ...panelStyle,
  marginBottom: 16,
};

export const contentParagraphStyle = {
  margin: 0,
  lineHeight: 1.7,
};

export const loginLeadTextStyle = {
  margin: "8px 0 0",
  lineHeight: 1.75,
};

export const hintPanelTextStyle = {
  fontSize: 13,
};

export const summaryPanelMinWidth = 240;

export const flowGuideVariantText = {
  login: "进入课程页前先完成账号选择或新增登录。",
  courses: "先完成课程选择，再去配置任务范围。",
  courseTasks: "先筛选，再勾选需要学习的任务点。",
  tasks: "开始任务后在这里查看进度、日志和播放状态。",
  settings: "在这里集中维护应用运行所需的全部配置。",
};

export const pageTopBannerTexts = {
  login: "先选账号，再登录进入课程页。",
  courses: "先选课程，再进入任务配置。",
  courseTasks: "先筛选任务，再勾选学习范围。",
  tasks: "确认范围后开始任务并查看实时进度。",
  settings: "统一调整默认参数、题库连接和通知。",
};

export const pageTopGuideStyle = {
  marginBottom: 16,
};

export const pageSummaryTitleStyle = {
  display: "block",
  marginBottom: 8,
};

export const pageSummaryTextStyle = {
  display: "block",
  fontSize: 12,
  lineHeight: 1.7,
};

export const subtlePanelStyle = {
  ...panelStyle,
  padding: "10px 14px",
};

export const calloutPanelStyle = {
  ...panelStyle,
  border: "1px solid #dceafe",
  background: "linear-gradient(180deg, #f8fbff 0%, #ffffff 100%)",
};

export const actionStickyCardStyle = stickyActionBarStyle;

export const pageGridSummaryStyle = summaryGridStyle;

export const headerSubtitleTextStyle = {
  fontSize: 12,
};

export const accountDropdownButtonStyle = {
  height: "auto",
  padding: "10px 14px",
  borderRadius: 16,
  border: "1px solid #e6ebf2",
  background: "#ffffff",
  boxShadow: "0 6px 18px rgba(15,23,42,0.04)",
};

export const topBannerHeadingText = "操作引导";
export const formSummaryHeadingText = "当前表单摘要";

export const pageSummaryHintStyle = {
  fontSize: 12,
  lineHeight: 1.7,
};

export const sectionTitleMarginStyle = {
  marginBottom: 16,
};

export const pageLeadParagraphStyle = {
  margin: "8px 0 0",
  lineHeight: 1.7,
};

export const pageTopIntroStyle = {
  display: "flex",
  flexDirection: "column" as const,
  gap: 20,
};

export const guideCompactTextStyle = {
  display: "block",
  fontSize: 13,
};

export const pageMetricsGridStyle = summaryGridStyle;

export const pageTitleTextStyle = {
  margin: 0,
};

export const settingsSummaryText = "这里会实时反映表单中的最新输入，便于在保存前快速确认关键配置。";

export const pageHelperTexts = {
  savedAccounts: "可直接恢复历史登录会话，也可以删除失效账号。",
  newAccount: "选择登录方式并提交，登录成功后会自动保存账号，方便下次快速进入。",
  coursesSelect: "点击课程卡片可直接进入任务配置",
};

export const pageFooterTexts = {
  courses: "完成任务配置后即可前往任务执行页开始学习",
  courseTasks: "选择会自动保存，可直接返回课程列表，后续再前往任务执行页开始学习",
  settings: "配置文件保存在应用数据目录下的 config.json，可先导入 INI 再按当前界面进行调整。",
};

export const loginHelpTexts = {
  password: "适合日常使用，登录成功后会自动保存当前账号，后续可在左侧历史账号区直接恢复。",
  cookie: "适合已有浏览器会话的情况。粘贴完整 Cookie 后登录，系统会自动保存为可复用账号。",
};

export const topBannerTexts = workflowBannerCopy;

export const guidePanelTitle = "操作引导";
export const summaryPanelTitle = "当前概览";
export const formSummaryPanelTitle = "当前表单摘要";
export const selectionSummaryPanelTitle = "当前筛选摘要";
export const filterSummaryTitle = "当前筛选";
export const savedAccountsPanelTitle = "已保存账号";

export const bannerTextEmphasisStyle = {
  display: "block",
  fontSize: 16,
  fontWeight: 600,
  lineHeight: 1.7,
  color: "#1f2937",
};

export const bannerTextLabelStyle = {
  display: "block",
  fontSize: 12,
  fontWeight: 600,
  color: "#1677ff",
  marginBottom: 8,
};

export const bannerPanelStyle = {
  padding: "16px 18px",
  borderRadius: 18,
  border: "1px solid #dbeafe",
  background: "linear-gradient(135deg, #eef4ff 0%, #f8fbff 100%)",
  boxShadow: "0 8px 24px rgba(22,119,255,0.06)",
};

export function SectionTitle({ title, subtitle }: { title: string; subtitle: string }) {
  return (
    <div>
      <Text strong style={{ display: "block", fontSize: 14 }}>
        {title}
      </Text>
      <Text type="secondary" style={{ fontSize: 12 }}>
        {subtitle}
      </Text>
    </div>
  );
}

export function SummaryMetric({
  label,
  value,
  hint,
}: {
  label: string;
  value: string;
  hint: string;
}) {
  return (
    <div style={panelStyle}>
      <Text type="secondary" style={{ display: "block", fontSize: 12, marginBottom: 6 }}>
        {label}
      </Text>
      <Text strong style={{ display: "block", fontSize: 18, lineHeight: 1.4 }}>
        {value}
      </Text>
      <Text type="secondary" style={{ fontSize: 12 }}>
        {hint}
      </Text>
    </div>
  );
}

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
        style={{ borderRight: "1px solid #e9eef5", background: "#ffffff" }}
      >
        <div
          style={{
            padding: "20px 20px 16px",
            borderBottom: "1px solid #eef1f5",
            background: "#ffffff",
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
          style={{ borderRight: 0, padding: "16px 12px", background: "#ffffff" }}
        />
      </Sider>

      <Layout>
        <Header
          style={{
            height: 88,
            padding: "0 24px",
            background: "#f5f7fb",
            borderBottom: "1px solid #e9eef5",
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
                border: "1px solid #e6ebf2",
                background: "#ffffff",
                boxShadow: "0 6px 18px rgba(15,23,42,0.04)",
              }}
            >
              <Space size={12} align="center">
                <div
                  style={{
                    width: 36,
                    height: 36,
                    borderRadius: 12,
                    background: "#eef4ff",
                    color: "#1677ff",
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
            background: "#f5f7fb",
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
