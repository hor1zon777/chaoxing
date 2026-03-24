import { useEffect, useState } from "react";
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
  Divider,
  Typography,
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

const { Text } = Typography;

export function SettingsPage() {
  const { config, loading, loadConfig, saveConfig, importIni } =
    useConfigStore();
  const [form] = Form.useForm<AppConfig>();
  const [msgApi, contextHolder] = message.useMessage();
  const [testingNotification, setTestingNotification] = useState(false);
  const [testingTiku, setTestingTiku] = useState(false);

  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  useEffect(() => {
    form.setFieldsValue(config);
  }, [config, form]);

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
    <Card bordered={false}>
      <Form.Item label="播放速度" name="speed">
        <Slider min={0.5} max={16} step={0.5} marks={{ 1: "1x", 2: "2x", 4: "4x", 8: "8x", 16: "16x" }} />
      </Form.Item>
      <Form.Item label="并发任务数" name="jobs">
        <InputNumber min={1} max={16} />
      </Form.Item>
      <Form.Item label="未开放章节处理" name="notopenAction">
        <Select
          options={[
            { value: "retry", label: "重试 (retry)" },
            { value: "continue", label: "跳过 (continue)" },
          ]}
        />
      </Form.Item>
    </Card>
  );

  const tikuTab = (
    <Card bordered={false}>
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
        <Slider min={0} max={1} step={0.05} marks={{ 0.6: "60%", 0.8: "80%", 0.9: "90%", 1: "100%" }} />
      </Form.Item>

      <Form.Item label="查询延迟 (秒)" name="tikuDelay">
        <InputNumber min={0} max={30} step={0.5} />
      </Form.Item>

      <Divider orientation="left">AI 大模型设置</Divider>

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
        <InputNumber min={0} max={60} />
      </Form.Item>
      <Form.Item label="启动时检查 LLM 连接" name="checkLlmConnection" valuePropName="checked">
        <Switch />
      </Form.Item>

      <Divider orientation="left">硅基流动设置</Divider>

      <Form.Item label="SiliconFlow Key" name="siliconflowKey">
        <Input.Password />
      </Form.Item>
      <Form.Item label="SiliconFlow 模型" name="siliconflowModel">
        <Input placeholder="Qwen/Qwen2.5-7B-Instruct" />
      </Form.Item>
      <Form.Item label="SiliconFlow 端点" name="siliconflowEndpoint">
        <Input placeholder="https://api.siliconflow.cn/v1" />
      </Form.Item>

      <Divider orientation="left">LIKE 知识库设置</Divider>

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
        <InputNumber min={1} max={10} />
      </Form.Item>

      <Divider orientation="left">TikuAdapter 设置</Divider>

      <Form.Item label="适配器 URL" name="tikuAdapterUrl">
        <Input placeholder="http://localhost:8080" />
      </Form.Item>

      <Divider orientation="left">判断题关键词</Divider>

      <Form.Item label="判断为「对」的关键词" name="trueList">
        <Input placeholder="正确,对,是,YES,对的,√" />
      </Form.Item>
      <Form.Item label="判断为「错」的关键词" name="falseList">
        <Input placeholder="错误,错,否,NO,错的,×" />
      </Form.Item>

      <Form.Item>
        <Button type="primary" onClick={handleTestTiku} loading={testingTiku}>
          测试题库连接
        </Button>
      </Form.Item>
    </Card>
  );

  const notificationTab = (
    <Card bordered={false}>
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

      <Form.Item label="通知 URL / Token" name="notificationUrl">
        <Input placeholder="各服务的推送 URL 或 Token" />
      </Form.Item>

      <Form.Item label="Telegram Chat ID" name="notificationTgChatId">
        <Input placeholder="仅 Telegram 需要" />
      </Form.Item>

      <Form.Item>
        <Button onClick={handleTestNotification} loading={testingNotification}>
          发送测试通知
        </Button>
      </Form.Item>
    </Card>
  );

  return (
    <div style={{ padding: 24 }}>
      {contextHolder}
      <Form
        form={form}
        layout="vertical"
        initialValues={config}
        style={{ maxWidth: 800 }}
      >
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

        <Space style={{ marginTop: 16 }}>
          <Button type="primary" onClick={handleSave} loading={loading}>
            保存配置
          </Button>
          <Button icon={<ImportOutlined />} onClick={handleImportIni}>
            导入 INI 配置
          </Button>
        </Space>

        <div style={{ marginTop: 8 }}>
          <Text type="secondary">
            配置文件保存在应用数据目录下的 config.json
          </Text>
        </div>
      </Form>
    </div>
  );
}
