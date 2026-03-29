import type { CSSProperties } from "react";
import { Typography } from "antd";

const { Text } = Typography;

export const surfaceCardStyle = {
  borderRadius: 20,
  boxShadow: "0 18px 40px rgba(37,99,235,0.08), 0 10px 24px rgba(15,23,42,0.05)",
  border: "1px solid #d8e6f8",
  background: "linear-gradient(180deg, #ffffff 0%, #f8fbff 100%)",
};

export const panelCardStyle = {
  borderRadius: 16,
  border: "1px solid #dbe7f6",
  background: "linear-gradient(180deg, #fcfdff 0%, #f6f9ff 100%)",
  boxShadow: "inset 0 1px 0 rgba(255,255,255,0.75)",
};

export const panelStyle = {
  ...panelCardStyle,
  padding: "14px 16px",
  background: "linear-gradient(180deg, #f4f8ff 0%, #edf4ff 100%)",
  border: "1px solid #cfe0ff",
  boxShadow: "0 8px 20px rgba(59,130,246,0.08)",
};

export const primaryActionButtonStyle = {
  minWidth: 132,
  height: 42,
  borderRadius: 12,
  paddingInline: 20,
  fontWeight: 600,
  boxShadow: "0 10px 24px rgba(37,99,235,0.16)",
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
        padding: "18px 20px",
        borderRadius: 20,
        border: "1px solid #c7dbff",
        background:
          "linear-gradient(135deg, rgba(37,99,235,0.14) 0%, rgba(59,130,246,0.08) 34%, rgba(245,158,11,0.10) 100%)",
        boxShadow: "0 14px 30px rgba(37,99,235,0.10)",
      }}
    >
      <Text
        style={{
          display: "block",
          fontSize: 16,
          fontWeight: 700,
          lineHeight: 1.75,
          color: "#1e3a8a",
          letterSpacing: 0.2,
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
        color: "#2563eb",
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
export const selectedCourseCardShadow = "0 18px 34px rgba(37,99,235,0.12)";

export const summaryAccentPanelStyle = {
  ...panelStyle,
  background: "linear-gradient(180deg, #eff6ff 0%, #ffffff 100%)",
  border: "1px solid #bfd4ff",
};

export const savedAccountCardStyle = {
  ...surfaceCardStyle,
  background: "linear-gradient(180deg, #ffffff 0%, #f3f8ff 100%)",
  border: "1px solid #d8e7fb",
};

export const accountItemPanelStyle = {
  ...panelStyle,
  background: "linear-gradient(180deg, #ffffff 0%, #f8fbff 100%)",
  border: "1px solid #d6e3f8",
};

export const topSummaryPanelStyle = {
  ...panelStyle,
  border: "1px solid #bfd4ff",
  background: "linear-gradient(180deg, #eef5ff 0%, #ffffff 100%)",
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
  boxShadow: "0 24px 60px rgba(37,99,235,0.14), 0 14px 36px rgba(15,23,42,0.06)",
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
  color: "#2563eb",
  marginBottom: 8,
};

export const bannerPanelStyle = {
  padding: "18px 20px",
  borderRadius: 20,
  border: "1px solid #c7dbff",
  background:
    "linear-gradient(135deg, rgba(37,99,235,0.14) 0%, rgba(59,130,246,0.08) 34%, rgba(245,158,11,0.10) 100%)",
  boxShadow: "0 14px 30px rgba(37,99,235,0.10)",
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
  style,
}: {
  label: string;
  value: string;
  hint: string;
  style?: CSSProperties;
}) {
  return (
    <div
      style={{
        ...panelStyle,
        background: "linear-gradient(180deg, #ffffff 0%, #edf4ff 100%)",
        border: "1px solid #cfe0ff",
        boxShadow: "0 10px 22px rgba(37,99,235,0.08)",
        ...style,
      }}
    >
      <Text type="secondary" style={{ display: "block", fontSize: 12, marginBottom: 6, color: "#475569" }}>
        {label}
      </Text>
      <Text strong style={{ display: "block", fontSize: 18, lineHeight: 1.4, color: "#1e3a8a" }}>
        {value}
      </Text>
      <Text type="secondary" style={{ fontSize: 12, color: "#64748b" }}>
        {hint}
      </Text>
    </div>
  );
}
