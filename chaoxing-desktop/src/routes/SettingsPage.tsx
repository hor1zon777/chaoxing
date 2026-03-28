import { useEffect, useMemo, useState } from "react";
import {
  Tabs,
  Form,
  Input,
  InputNumber,
  Switch,
  Select,
  Button,
  Space,
  Card,
  message,
  Slider,
  Typography,
  Grid,
} from "antd";
import {
  SettingOutlined,
  BookOutlined,
  BellOutlined,
  ImportOutlined,
} from "@ant-design/icons";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useConfigStore } from "../stores/configStore";
import type { AppConfig } from "../types/config";
import {
  GuideBanner,
  pageTopBannerTexts,
  panelCardStyle,
  primaryActionButtonStyle,
  SectionTitle,
  SummaryMetric,
  surfaceCardStyle,
} from "../components/Layout/AppLayout";

const { Text, Title, Paragraph } = Typography;
const { useBreakpoint } = Grid;

export function SettingsPage() {
  const screens = useBreakpoint();
  const { config, loading, loadConfig, saveConfig, importIni } =
    useConfigStore();
  const [form] = Form.useForm<AppConfig>();
  const [msgApi, contextHolder] = message.useMessage();
  const [testingNotification, setTestingNotification] = useState(false);
  const [testingTiku, setTestingTiku] = useState(false);
  const formValues = Form.useWatch([], form) as Partial<AppConfig> | undefined;

  const summaryConfig = {
    ...config,
    ...formValues,
  } as AppConfig;

  useEffect(() => {
    void loadConfig();
  }, [loadConfig]);

  useEffect(() => {
    form.setFieldsValue(config);
  }, [config, form]);

  const settingsSummary = useMemo(
    () => [
      {
        label: "播放速度",
        value: `${summaryConfig.speed}x`,
        hint: "任务执行默认倍速",
      },
      {
        label: "并发任务",
        value: `${summaryConfig.jobs}`,
        hint: "同时执行的任务线程数",
      },
      {
        label: "题库提供者",
        value: summaryConfig.tikuProvider || "未启用",
        hint: "当前答题来源",
      },
      {
        label: "通知服务",
        value: summaryConfig.notificationProvider || "未启用",
        hint: "任务完成后的消息推送",
      },
    ],
    [summaryConfig],
  );

  const notificationFieldMeta = useMemo(() => {
    switch (summaryConfig.notificationProvider) {
      case "serverchan":
        return {
          primaryLabel: "Server酱 SendKey / 推送地址",
          primaryPlaceholder: "输入 Server酱 SendKey 或完整推送地址",
          helperText: "Server酱通常只需要 SendKey 或完整推送 URL。",
          showPrimaryField: true,
          showChatIdField: false,
        };
      case "qmsg":
        return {
          primaryLabel: "Qmsg Key / 推送地址",
          primaryPlaceholder: "输入 Qmsg Key 或完整推送地址",
          helperText: "Qmsg 通常只需要推送 Key 或完整推送 URL。",
          showPrimaryField: true,
          showChatIdField: false,
        };
      case "bark":
        return {
          primaryLabel: "Bark 推送地址",
          primaryPlaceholder: "输入 Bark 推送 URL，例如 https://api.day.app/你的key",
          helperText: "Bark 只需要推送地址，无需额外 Chat ID。",
          showPrimaryField: true,
          showChatIdField: false,
        };
      case "telegram":
        return {
          primaryLabel: "Telegram Bot Token / API 地址",
          primaryPlaceholder: "输入 Bot Token 或 Telegram Bot API 地址",
          helperText: "Telegram 需要同时填写 Bot Token 或 API 地址，以及 Chat ID。",
          showPrimaryField: true,
          showChatIdField: true,
        };
      default:
        return {
          primaryLabel: "通知 URL / Token",
          primaryPlaceholder: "选择通知服务后显示对应参数",
          helperText: "请先选择通知渠道，再填写对应的配置参数。",
          showPrimaryField: false,
          showChatIdField: false,
        };
    }
  }, [summaryConfig.notificationProvider]);

  const notificationProviderLabel =
    summaryConfig.notificationProvider || "未启用";

  const notificationDisabled = !summaryConfig.notificationProvider;

  const handleSave = async () => {
    try {
      const values = await form.validateFields();
      await saveConfig(values);
      msgApi.success("配置已保存");
    } catch {
      msgApi.error("保存失败");
    }
  };

  const handleImportIni = async () => {
    const selected = await open({
      filters: [{ name: "INI 配置文件", extensions: ["ini"] }],
    });
    if (selected) {
      try {
        await importIni(selected as string);
        msgApi.success("INI 配置导入成功");
      } catch {
        msgApi.error("INI 导入失败");
      }
    }
  };

  const handleTestNotification = async () => {
    setTestingNotification(true);
    try {
      const values = form.getFieldsValue();
      const ok = await invoke<boolean>("test_notification", {
        provider: values.notificationProvider,
        url: values.notificationUrl,
        tgChatId: values.notificationTgChatId,
      });
      if (ok) {
        msgApi.success("测试通知发送成功");
      } else {
        msgApi.error("测试通知发送失败");
      }
    } catch {
      msgApi.error("测试通知异常");
    } finally {
      setTestingNotification(false);
    }
  };

  const handleTestTiku = async () => {
    setTestingTiku(true);
    try {
      const values = form.getFieldsValue();
      await saveConfig(values);
      const ok = await invoke<boolean>("test_tiku_connection");
      if (ok) {
        msgApi.success("题库连接正常");
      } else {
        msgApi.error("题库连接失败");
      }
    } catch (e) {
      msgApi.error(`题库测试异常: ${e}`);
    } finally {
      setTestingTiku(false);
    }
  };

  const generalTab = (
    <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
      <SettingsSection
        title="任务执行默认参数"
        description="这些配置会作为课程任务执行时的默认参数。"
      >
        <Form.Item label="播放速度" name="speed">
          <Slider
            min={0.5}
            max={16}
            step={0.5}
            marks={{ 1: "1x", 2: "2x", 4: "4x", 8: "8x", 16: "16x" }}
          />
        </Form.Item>
        <Form.Item label="并发任务数" name="jobs">
          <InputNumber min={1} max={16} style={{ width: "100%" }} />
        </Form.Item>
        <Form.Item label="未开放章节处理" name="notopenAction">
          <Select
            options={[
              { value: "retry", label: "重试 (retry)" },
              { value: "continue", label: "跳过 (continue)" },
            ]}
          />
        </Form.Item>
      </SettingsSection>
    </div>
  );

  const tikuTab = (
    <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
      <SettingsSection
        title="题库基础设置"
        description="配置题库来源、令牌、自动提交和查询策略。"
      >
        <Form.Item label="题库提供者" name="tikuProvider">
          <Select
            allowClear
            placeholder="选择题库"
            options={[
              { value: "", label: "不使用题库" },
              { value: "yanxi", label: "言溪题库" },
              { value: "like", label: "LIKE 知识库" },
              { value: "adapter", label: "TikuAdapter" },
              { value: "ai", label: "AI 大模型" },
              { value: "siliconflow", label: "硅基流动" },
            ]}
          />
        </Form.Item>

        <Form.Item label="题库令牌" name="tikuTokens">
          <Input.TextArea rows={2} placeholder="多个用逗号分隔" />
        </Form.Item>

        <Form.Item label="自动提交" name="tikuSubmit" valuePropName="checked">
          <Switch />
        </Form.Item>

        <Form.Item label="答案覆盖率" name="tikuCoverRate">
          <Slider
            min={0}
            max={1}
            step={0.05}
            marks={{ 0.6: "60%", 0.8: "80%", 0.9: "90%", 1: "100%" }}
          />
        </Form.Item>

        <Form.Item label="查询延迟 (秒)" name="tikuDelay">
          <InputNumber min={0} max={30} step={0.5} style={{ width: "100%" }} />
        </Form.Item>
      </SettingsSection>

      <SettingsSection
        title="AI 大模型设置"
        description="用于 AI 题库提供者时的接口、模型和代理配置。"
      >
        <Form.Item label="API 端点" name="aiEndpoint">
          <Input placeholder="https://api.openai.com/v1" />
        </Form.Item>
        <Form.Item label="API Key" name="aiKey">
          <Input.Password placeholder="sk-..." />
        </Form.Item>
        <Form.Item label="模型名称" name="aiModel">
          <Input placeholder="gpt-4o-mini" />
        </Form.Item>
        <Form.Item label="代理地址" name="aiProxy">
          <Input placeholder="http://127.0.0.1:7890" />
        </Form.Item>
        <Form.Item label="最小请求间隔 (秒)" name="aiMinInterval">
          <InputNumber min={0} max={60} style={{ width: "100%" }} />
        </Form.Item>
        <Form.Item label="启动时检查 LLM 连接" name="checkLlmConnection" valuePropName="checked">
          <Switch />
        </Form.Item>
      </SettingsSection>

      <SettingsSection
        title="硅基流动与 LIKE 设置"
        description="在对应题库模式下填写专属配置。"
      >
        <Form.Item label="SiliconFlow Key" name="siliconflowKey">
          <Input.Password />
        </Form.Item>
        <Form.Item label="SiliconFlow 模型" name="siliconflowModel">
          <Input placeholder="Qwen/Qwen2.5-7B-Instruct" />
        </Form.Item>
        <Form.Item label="SiliconFlow 端点" name="siliconflowEndpoint">
          <Input placeholder="https://api.siliconflow.cn/v1" />
        </Form.Item>
        <Form.Item label="模糊搜索" name="likeSearch" valuePropName="checked">
          <Switch />
        </Form.Item>
        <Form.Item label="视觉识别" name="likeVision" valuePropName="checked">
          <Switch />
        </Form.Item>
        <Form.Item label="LIKE 模型" name="likeModel">
          <Input />
        </Form.Item>
        <Form.Item label="搜索重试" name="likeRetry" valuePropName="checked">
          <Switch />
        </Form.Item>
        <Form.Item label="重试次数" name="likeRetryTimes">
          <InputNumber min={1} max={10} style={{ width: "100%" }} />
        </Form.Item>
      </SettingsSection>

      <SettingsSection
        title="适配器与判断题关键词"
        description="适用于 TikuAdapter 与判断题答案纠偏。"
      >
        <Form.Item label="适配器 URL" name="tikuAdapterUrl">
          <Input placeholder="http://localhost:8080" />
        </Form.Item>
        <Form.Item label="判断为「对」的关键词" name="trueList">
          <Input placeholder="正确,对,是,YES,对的,√" />
        </Form.Item>
        <Form.Item label="判断为「错」的关键词" name="falseList">
          <Input placeholder="错误,错,否,NO,错的,×" />
        </Form.Item>
        <Form.Item style={{ marginBottom: 0 }}>
          <Button type="primary" onClick={handleTestTiku} loading={testingTiku}>
            测试题库连接
          </Button>
        </Form.Item>
      </SettingsSection>
    </div>
  );

  const notificationTab = (
    <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
      <SettingsSection
        title="通知渠道设置"
        description="配置任务完成后的消息推送服务，并发送测试通知确认联通性。"
      >
        <Form.Item label="通知服务" name="notificationProvider">
          <Select
            allowClear
            placeholder="选择通知服务"
            options={[
              { value: "", label: "不启用通知" },
              { value: "serverchan", label: "Server酱" },
              { value: "qmsg", label: "Qmsg酱" },
              { value: "bark", label: "Bark" },
              { value: "telegram", label: "Telegram" },
            ]}
          />
        </Form.Item>

        <div style={{ marginBottom: 16, ...panelCardStyle, padding: "12px 14px" }}>
          <Text strong style={{ display: "block", marginBottom: 4 }}>
            当前渠道：{notificationProviderLabel}
          </Text>
          <Text type="secondary" style={{ fontSize: 12, lineHeight: 1.7 }}>
            {notificationFieldMeta.helperText}
          </Text>
        </div>

        {notificationFieldMeta.showPrimaryField ? (
          <Form.Item label={notificationFieldMeta.primaryLabel} name="notificationUrl">
            <Input placeholder={notificationFieldMeta.primaryPlaceholder} />
          </Form.Item>
        ) : null}

        {notificationFieldMeta.showChatIdField ? (
          <Form.Item label="Telegram Chat ID" name="notificationTgChatId">
            <Input placeholder="例如 123456789 或群组 Chat ID" />
          </Form.Item>
        ) : null}

        <Form.Item style={{ marginBottom: 0 }}>
          <Button onClick={handleTestNotification} loading={testingNotification} disabled={notificationDisabled}>
            发送测试通知
          </Button>
        </Form.Item>
      </SettingsSection>
    </div>
  );

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
      {contextHolder}
      <GuideBanner text={pageTopBannerTexts.settings} />

      <Card style={surfaceCardStyle} styles={{ body: { padding: 24 } }}>
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "flex-start",
            gap: 16,
            flexWrap: "wrap",
          }}
        >
          <div style={{ maxWidth: 780 }}>
            <Title level={4} style={{ margin: 0 }}>
              应用设置
            </Title>
            <Paragraph type="secondary" style={{ margin: "8px 0 0", lineHeight: 1.7 }}>
              可在此查看当前表单摘要，并按分组调整默认参数、题库连接和通知方式。
            </Paragraph>
          </div>
          <div
            style={{
              padding: "12px 16px",
              borderRadius: 16,
              border: "1px solid #dceafe",
              background: "linear-gradient(180deg, #f8fbff 0%, #ffffff 100%)",
              minWidth: screens.md ? 250 : "100%",
            }}
          >
            <Text strong style={{ display: "block", marginBottom: 6 }}>
              当前表单摘要
            </Text>
            <Text type="secondary" style={{ fontSize: 12, lineHeight: 1.7 }}>
              这里会实时反映表单中的最新输入，便于在保存前快速确认关键配置。
            </Text>
          </div>
        </div>

        <div
          style={{
            display: "grid",
            gridTemplateColumns: screens.md ? "repeat(4, minmax(0, 1fr))" : "repeat(2, minmax(0, 1fr))",
            gap: 12,
            marginTop: 20,
          }}
        >
          {settingsSummary.map((item) => (
            <SummaryMetric key={item.label} label={item.label} value={item.value} hint={item.hint} />
          ))}
        </div>
      </Card>

      <Card style={surfaceCardStyle} styles={{ body: { padding: 20 } }}>
        <Form form={form} layout="vertical" initialValues={config}>
          <Tabs
            defaultActiveKey="general"
            items={[
              {
                key: "general",
                label: (
                  <span>
                    <SettingOutlined /> 通用设置
                  </span>
                ),
                children: generalTab,
              },
              {
                key: "tiku",
                label: (
                  <span>
                    <BookOutlined /> 题库设置
                  </span>
                ),
                children: tikuTab,
              },
              {
                key: "notification",
                label: (
                  <span>
                    <BellOutlined /> 通知设置
                  </span>
                ),
                children: notificationTab,
              },
            ]}
          />
        </Form>
      </Card>

      <Card style={surfaceCardStyle} styles={{ body: { padding: "16px 20px" } }}>
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            gap: 16,
            flexWrap: "wrap",
          }}
        >
          <div>
            <Text strong style={{ display: "block" }}>
              配置文件位置
            </Text>
            <Text type="secondary" style={{ fontSize: 12 }}>
              配置文件保存在应用数据目录下的 config.json，可先导入 INI 再按当前界面进行调整。
            </Text>
          </div>
          <Space wrap>
            <Button icon={<ImportOutlined />} onClick={handleImportIni} style={{ ...primaryActionButtonStyle, minWidth: 120 }}>
              导入 INI 配置
            </Button>
            <Button type="primary" onClick={handleSave} loading={loading} style={{ ...primaryActionButtonStyle, minWidth: 120 }}>
              保存配置
            </Button>
          </Space>
        </div>
      </Card>
    </div>
  );
}

function SettingsSection({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: React.ReactNode;
}) {
  return (
    <Card style={panelCardStyle} styles={{ body: { padding: 18 } }}>
      <div style={{ marginBottom: 16 }}>
        <SectionTitle title={title} subtitle={description} />
      </div>
      {children}
    </Card>
  );
}
