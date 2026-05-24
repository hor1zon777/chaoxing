import { useEffect, useMemo, useState } from "react";
import { Form, Input, InputNumber, Select, Slider, Switch, message } from "antd";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useConfigStore } from "../stores/configStore";
import type { AppConfig } from "../types/config";
import { Card, Chip, Eyebrow, Headline, Metric, PillButton, UtilityButton } from "../components/ui/appleUI";

type SettingsTab = "general" | "tiku" | "notification";

export function SettingsPage() {
  const { config, loading, loadConfig, saveConfig, importIni } = useConfigStore();
  const [form] = Form.useForm<AppConfig>();
  const [msgApi, contextHolder] = message.useMessage();
  const [testingNotification, setTestingNotification] = useState(false);
  const [testingTiku, setTestingTiku] = useState(false);
  const [activeTab, setActiveTab] = useState<SettingsTab>("general");
  const formValues = Form.useWatch([], form) as Partial<AppConfig> | undefined;

  const summaryConfig = { ...config, ...formValues } as AppConfig;

  // 仅对比已注册（已渲染）的字段：若任意字段与 store 中 config 不同则视为 dirty
  const isDirty = useMemo(() => {
    if (!formValues) return false;
    return Object.entries(formValues).some(([key, value]) => {
      const original = (config as unknown as Record<string, unknown>)[key];
      return JSON.stringify(value) !== JSON.stringify(original);
    });
  }, [formValues, config]);

  useEffect(() => {
    void loadConfig();
  }, [loadConfig]);

  useEffect(() => {
    form.setFieldsValue(config);
  }, [config, form]);

  const settingsSummary = useMemo(
    () => [
      { label: "播放速度", value: `${summaryConfig.speed}x`, hint: "默认任务倍速" },
      { label: "并发任务", value: `${summaryConfig.jobs}`, hint: "同时执行线程" },
      {
        label: "题库提供者",
        value: summaryConfig.tikuProvider || "未启用",
        hint: "当前答题来源",
      },
      {
        label: "通知服务",
        value: summaryConfig.notificationProvider || "未启用",
        hint: "任务完成后推送",
      },
    ],
    [summaryConfig],
  );

  const notificationFieldMeta = useMemo(() => {
    switch (summaryConfig.notificationProvider) {
      case "serverchan":
        return {
          primaryLabel: "Server酱 SendKey / 推送地址",
          primaryPlaceholder: "输入 SendKey，或完整发送地址",
          helperText: "支持直接填写 SendKey，系统会自动补全发送地址。",
          showPrimaryField: true,
          showChatIdField: false,
        };
      case "qmsg":
        return {
          primaryLabel: "Qmsg Key / 推送地址",
          primaryPlaceholder: "输入 Qmsg Key，或完整发送地址",
          helperText: "支持直接填写 Qmsg Key，系统会自动补全。",
          showPrimaryField: true,
          showChatIdField: false,
        };
      case "bark":
        return {
          primaryLabel: "Bark Key / 推送地址",
          primaryPlaceholder: "输入 Bark Key，或完整推送地址",
          helperText: "支持直接填写 Bark Key，将自动拼接为 https://api.day.app/...",
          showPrimaryField: true,
          showChatIdField: false,
        };
      case "telegram":
        return {
          primaryLabel: "Telegram Bot Token / API 地址",
          primaryPlaceholder: "输入 Bot Token，或完整 sendMessage API",
          helperText: "支持直接填写 Bot Token，需要同时填写 Chat ID。",
          showPrimaryField: true,
          showChatIdField: true,
        };
      default:
        return {
          primaryLabel: "通知 URL / Token",
          primaryPlaceholder: "选择通知服务后显示对应参数",
          helperText: "请先选择通知渠道，再填写对应参数。",
          showPrimaryField: false,
          showChatIdField: false,
        };
    }
  }, [summaryConfig.notificationProvider]);

  const notificationDisabled = !summaryConfig.notificationProvider;

  const handleSave = async () => {
    try {
      // 先校验当前 Tab 注册字段
      await form.validateFields();
      // 取 ALL 字段（getFieldsValue(true) 包含 antd Form preserve=true 保留的
      // 已切走 Tab 中的字段值），避免"在 A Tab 改后切到 B Tab 保存"丢失 A 的修改
      const allValues = form.getFieldsValue(true);
      const merged: AppConfig = { ...config, ...allValues };
      await saveConfig(merged);
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
        provider: values.notificationProvider ?? "",
        url: values.notificationUrl ?? "",
        tgChatId: values.notificationTgChatId ?? "",
      });
      if (ok) msgApi.success("测试通知发送成功");
      else msgApi.error("测试通知发送失败");
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
      const merged: AppConfig = { ...config, ...values };
      await saveConfig(merged);
      const ok = await invoke<boolean>("test_tiku_connection");
      if (ok) msgApi.success("题库连接正常");
      else msgApi.error("题库连接失败");
    } catch (e) {
      msgApi.error(`题库测试异常: ${e}`);
    } finally {
      setTestingTiku(false);
    }
  };

  return (
    <div style={{ background: "var(--apple-color-canvas)" }}>
      {contextHolder}

      {/* Hero */}
      <section style={{ padding: "48px 22px 24px" }}>
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "flex-end",
              gap: 24,
              flexWrap: "wrap",
            }}
          >
            <div style={{ minWidth: 0, flex: "1 1 360px" }}>
              <Eyebrow>设置</Eyebrow>
              <Headline level="lg" style={{ marginTop: 8 }}>
                定制你的学习节奏。
              </Headline>
              <p
                style={{
                  fontFamily: "var(--apple-font-text)",
                  fontSize: 17,
                  color: "var(--apple-color-ink-muted-80)",
                  lineHeight: 1.47,
                  letterSpacing: "-0.374px",
                  margin: "14px 0 0",
                  maxWidth: 640,
                }}
              >
                调整默认参数、题库连接和通知方式。配置保存在应用数据目录的 config.json。
              </p>
            </div>
            <div style={{ display: "flex", gap: 12, flexWrap: "wrap" }}>
              <UtilityButton light onClick={() => void handleImportIni()}>
                导入 INI
              </UtilityButton>
              <PillButton onClick={() => void handleSave()} disabled={loading}>
                {loading ? "保存中…" : "保存配置"}
              </PillButton>
            </div>
          </div>

          {/* Metric row */}
          <div
            style={{
              marginTop: 32,
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))",
              gap: 16,
            }}
          >
            {settingsSummary.map((item) => (
              <Metric key={item.label} label={item.label} value={item.value} hint={item.hint} />
            ))}
          </div>
        </div>
      </section>

      {/* Tabs row */}
      <section
        style={{
          padding: "20px 22px 32px",
          borderTop: "1px solid var(--apple-color-divider-soft)",
        }}
      >
        <div
          style={{
            maxWidth: 1024,
            margin: "0 auto",
            display: "flex",
            justifyContent: "center",
            gap: 10,
            flexWrap: "wrap",
          }}
        >
          <Chip selected={activeTab === "general"} onClick={() => setActiveTab("general")}>
            通用
          </Chip>
          <Chip selected={activeTab === "tiku"} onClick={() => setActiveTab("tiku")}>
            题库
          </Chip>
          <Chip selected={activeTab === "notification"} onClick={() => setActiveTab("notification")}>
            通知
          </Chip>
        </div>
      </section>

      <section style={{ padding: "0 22px 80px" }}>
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          <Form form={form} layout="vertical" initialValues={config} preserve>
            {activeTab === "general" ? (
              <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
                <Card title="任务执行默认参数" subtitle="影响所有课程任务的默认行为">
                  <div style={{ marginTop: 16, display: "flex", flexDirection: "column", gap: 4 }}>
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
                    <Form.Item
                      label="单课程章节内任务并发"
                      name="tasksPerChapter"
                      tooltip="同一章节里最多并行处理几个任务点（视频/作业等）。设为 1 即原有串行行为；增大可加速但 API 压力更大。"
                    >
                      <InputNumber min={1} max={8} style={{ width: "100%" }} />
                    </Form.Item>
                    <Form.Item
                      label="单课程章节并发"
                      name="chaptersPerCourse"
                      tooltip="同一门课程里最多同时处理几个章节。设为 1 保持顺序解锁；增大可显著加速，但某些课程要求前章节完成才能解锁后章节，此时后章节会触发未开放重试/跳过。"
                    >
                      <InputNumber min={1} max={8} style={{ width: "100%" }} />
                    </Form.Item>
                    <Form.Item label="未开放章节处理" name="notopenAction">
                      <Select
                        options={[
                          { value: "retry", label: "重试 (retry)" },
                          { value: "continue", label: "跳过 (continue)" },
                        ]}
                      />
                    </Form.Item>
                  </div>
                </Card>
              </div>
            ) : null}

            {activeTab === "tiku" ? (
              <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
                <Card title="题库基础设置" subtitle="选择题库来源并配置查询策略">
                  <div style={{ marginTop: 16 }}>
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
                  </div>
                </Card>

                <Card title="AI 大模型设置" subtitle="用于 AI 题库的接口、模型和代理配置">
                  <div style={{ marginTop: 16 }}>
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
                  </div>
                </Card>

                <Card title="硅基流动 & LIKE 设置" subtitle="对应题库模式下的专属配置">
                  <div style={{ marginTop: 16 }}>
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
                  </div>
                </Card>

                <Card title="适配器与判断题关键词" subtitle="适用于 TikuAdapter 与判断题答案纠偏">
                  <div style={{ marginTop: 16 }}>
                    <Form.Item label="适配器 URL" name="tikuAdapterUrl">
                      <Input placeholder="http://localhost:8080" />
                    </Form.Item>
                    <Form.Item label="判断为「对」的关键词" name="trueList">
                      <Input placeholder="正确,对,是,YES,对的,√" />
                    </Form.Item>
                    <Form.Item label="判断为「错」的关键词" name="falseList">
                      <Input placeholder="错误,错,否,NO,错的,×" />
                    </Form.Item>
                    <Form.Item style={{ marginBottom: 0, marginTop: 12 }}>
                      <PillButton
                        variant="ghost"
                        onClick={() => void handleTestTiku()}
                        disabled={testingTiku}
                      >
                        {testingTiku ? "测试中…" : "测试题库连接"}
                      </PillButton>
                    </Form.Item>
                  </div>
                </Card>
              </div>
            ) : null}

            {activeTab === "notification" ? (
              <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
                <Card title="通知渠道" subtitle="配置任务完成后的消息推送服务">
                  <div style={{ marginTop: 16 }}>
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

                    <div
                      style={{
                        padding: "14px 16px",
                        borderRadius: 11,
                        background: "var(--apple-color-canvas-parchment)",
                        border: "1px solid var(--apple-color-divider-soft)",
                        marginBottom: 20,
                      }}
                    >
                      <div
                        style={{
                          fontFamily: "var(--apple-font-text)",
                          fontSize: 14,
                          fontWeight: 600,
                          letterSpacing: "-0.224px",
                          color: "var(--apple-color-ink)",
                          marginBottom: 4,
                        }}
                      >
                        当前渠道 · {summaryConfig.notificationProvider || "未启用"}
                      </div>
                      <div
                        style={{
                          fontFamily: "var(--apple-font-text)",
                          fontSize: 12,
                          letterSpacing: "-0.12px",
                          color: "var(--apple-color-ink-muted-48)",
                          lineHeight: 1.5,
                        }}
                      >
                        {notificationFieldMeta.helperText}
                      </div>
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
                      <PillButton
                        variant="ghost"
                        onClick={() => void handleTestNotification()}
                        disabled={notificationDisabled || testingNotification}
                      >
                        {testingNotification ? "发送中…" : "发送测试通知"}
                      </PillButton>
                    </Form.Item>
                  </div>
                </Card>
              </div>
            ) : null}
          </Form>
        </div>
      </section>

      {/* Sticky bottom save */}
      <div className="apple-sticky-bar">
        <div
          style={{
            maxWidth: 1024,
            margin: "0 auto",
            width: "100%",
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            gap: 16,
            flexWrap: "wrap",
          }}
        >
          <div>
            <div
              style={{
                fontFamily: "var(--apple-font-text)",
                fontSize: 17,
                fontWeight: 600,
                letterSpacing: "-0.374px",
                color: isDirty
                  ? "var(--apple-color-ink)"
                  : "var(--apple-color-ink-muted-48)",
              }}
            >
              {isDirty ? "配置变更未保存" : "配置已是最新"}
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
              {isDirty ? "点击保存以持久化到 config.json" : "全部更改已写入 config.json"}
            </div>
          </div>
          <div style={{ display: "flex", gap: 12, flexWrap: "wrap" }}>
            <UtilityButton light onClick={() => void handleImportIni()}>
              导入 INI
            </UtilityButton>
            <PillButton onClick={() => void handleSave()} disabled={loading}>
              {loading ? "保存中…" : "保存配置"}
            </PillButton>
          </div>
        </div>
      </div>
    </div>
  );
}
