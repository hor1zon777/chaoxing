import { useState, useRef, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Channel } from "@tauri-apps/api/core";
import {
  Card,
  Button,
  Progress,
  List,
  Tag,
  Space,
  Select,
  Typography,
  Empty,
  Divider,
  Segmented,
  message,
  Tooltip,
} from "antd";
import {
  PlayCircleOutlined,
  PauseCircleOutlined,
  StopOutlined,
  ClearOutlined,
  SoundOutlined,
  ThunderboltOutlined,
} from "@ant-design/icons";
import { useTaskStore } from "../stores/taskStore";
import { useCourseStore } from "../stores/courseStore";
import type { TaskEvent, LogEntry } from "../types/task";

const { Text } = Typography;

/** 格式化秒数为 mm:ss */
function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

/** 日志级别对应的 Tag 颜色 */
function getLogTagColor(level: string): string {
  switch (level) {
    case "error":
      return "red";
    case "warn":
      return "orange";
    case "info":
      return "blue";
    case "debug":
      return "default";
    default:
      return "default";
  }
}

/** 课程进度状态对应的 Tag */
function getCourseStatusTag(status: string) {
  switch (status) {
    case "running":
      return <Tag color="processing">进行中</Tag>;
    case "completed":
      return <Tag color="success">已完成</Tag>;
    case "error":
      return <Tag color="error">出错</Tag>;
    case "pending":
      return <Tag>等待中</Tag>;
    default:
      return <Tag>{status}</Tag>;
  }
}

/** 速度选项 */
const speedOptions = [
  { value: 1.0, label: "1.0x" },
  { value: 1.25, label: "1.25x" },
  { value: 1.5, label: "1.5x" },
  { value: 1.75, label: "1.75x" },
  { value: 2.0, label: "2.0x" },
];

/** 并发数选项 */
const jobCountOptions = [
  { value: 1, label: "1 线程" },
  { value: 2, label: "2 线程" },
  { value: 4, label: "4 线程" },
  { value: 6, label: "6 线程" },
  { value: 8, label: "8 线程" },
];

/** 未开放课程操作选项 */
const notopenOptions = [
  { value: "retry", label: "重试" },
  { value: "continue", label: "跳过继续" },
];

/** 任务执行页面 */
export function TaskPage() {
  const [speed, setSpeed] = useState(1.0);
  const [jobCount, setJobCount] = useState(4);
  const [notopenAction, setNotopenAction] = useState<"retry" | "continue">(
    "retry",
  );

  const {
    isRunning,
    isPaused,
    courseProgress,
    videoProgress,
    logs,
    logFilter,
    handleTaskEvent,
    setLogFilter,
    clearLogs,
    setRunning,
    setPaused,
    reset,
    addLog,
  } = useTaskStore();

  const { courses, selectedCourseIds } = useCourseStore();
  const logEndRef = useRef<HTMLDivElement>(null);
  const channelRef = useRef<Channel<TaskEvent> | null>(null);

  /** 自动滚动到日志底部 */
  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs.length]);

  /** 从课程列表中找出已选中的课程 */
  const selectedCourses = useMemo(
    () => courses.filter((c) => selectedCourseIds.includes(c.id)),
    [courses, selectedCourseIds],
  );

  /** 开始学习 */
  const handleStart = useCallback(async () => {
    if (selectedCourses.length === 0) {
      message.warning("请先在课程列表页选择要学习的课程");
      return;
    }

    reset();
    setRunning(true);

    const channel = new Channel<TaskEvent>();
    channel.onmessage = (event: TaskEvent) => {
      handleTaskEvent(event);
    };
    channelRef.current = channel;

    // 构建课程参数对象
    const courseParams = selectedCourses.map((c) => ({
      courseId: c.courseId,
      clazzId: c.clazzId,
      cpi: c.cpi,
      title: c.title,
    }));

    try {
      await invoke("start_course_tasks", {
        channel,
        courses: courseParams,
        speed,
        jobs: jobCount,
        notopenAction,
      });
    } catch (e: unknown) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      addLog("error", `任务启动失败: ${errorMsg}`);
      setRunning(false);
    }
  }, [
    selectedCourses,
    speed,
    jobCount,
    notopenAction,
    reset,
    setRunning,
    handleTaskEvent,
    addLog,
  ]);

  /** 暂停 */
  const handlePause = useCallback(async () => {
    try {
      await invoke("pause_tasks");
      setPaused(true);
      addLog("info", "任务已暂停");
    } catch (e: unknown) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      addLog("error", `暂停失败: ${errorMsg}`);
    }
  }, [setPaused, addLog]);

  /** 继续 */
  const handleResume = useCallback(async () => {
    try {
      await invoke("resume_tasks");
      setPaused(false);
      addLog("info", "任务已恢复");
    } catch (e: unknown) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      addLog("error", `恢复失败: ${errorMsg}`);
    }
  }, [setPaused, addLog]);

  /** 停止 */
  const handleStop = useCallback(async () => {
    try {
      await invoke("cancel_tasks");
      setRunning(false);
      setPaused(false);
      channelRef.current = null;
      addLog("warn", "任务已手动停止");
    } catch (e: unknown) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      addLog("error", `停止失败: ${errorMsg}`);
    }
  }, [setRunning, setPaused, addLog]);

  /** 过滤日志 */
  const filteredLogs = useMemo(() => {
    if (logFilter === "all") return logs;
    if (logFilter === "error") return logs.filter((l) => l.level === "error");
    if (logFilter === "warn")
      return logs.filter((l) => l.level === "warn" || l.level === "error");
    return logs;
  }, [logs, logFilter]);

  /** 统计进度 */
  const progressEntries = useMemo(
    () => Object.values(courseProgress),
    [courseProgress],
  );
  const totalChapters = progressEntries.reduce(
    (sum, p) => sum + p.totalChapters,
    0,
  );
  const completedChapters = progressEntries.reduce(
    (sum, p) => sum + p.completedChapters,
    0,
  );
  const overallPercent =
    totalChapters > 0 ? Math.round((completedChapters / totalChapters) * 100) : 0;

  /** 视频进度列表 */
  const videoEntries = useMemo(
    () => Object.values(videoProgress),
    [videoProgress],
  );

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      {/* 总体进度条 */}
      {isRunning && totalChapters > 0 && (
        <Card size="small" style={{ marginBottom: 16 }}>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 16,
            }}
          >
            <ThunderboltOutlined style={{ fontSize: 20, color: "#1677ff" }} />
            <div style={{ flex: 1 }}>
              <Progress
                percent={overallPercent}
                status={overallPercent === 100 ? "success" : "active"}
                format={() =>
                  `${completedChapters} / ${totalChapters} 章节`
                }
              />
            </div>
            <Text type="secondary">{progressEntries.length} 门课程</Text>
          </div>
        </Card>
      )}

      {/* 主内容区：左侧控制面板 + 右侧日志 */}
      <div style={{ display: "flex", flex: 1, gap: 16, minHeight: 0 }}>
        {/* 左侧面板 */}
        <div
          style={{
            width: "40%",
            display: "flex",
            flexDirection: "column",
            gap: 16,
            minHeight: 0,
          }}
        >
          {/* 控制面板 */}
          <Card
            title="任务控制"
            size="small"
            extra={
              <Space size="small">
                {!isRunning ? (
                  <Button
                    type="primary"
                    icon={<PlayCircleOutlined />}
                    onClick={handleStart}
                    disabled={selectedCourses.length === 0}
                  >
                    开始学习
                  </Button>
                ) : (
                  <>
                    {isPaused ? (
                      <Button
                        icon={<PlayCircleOutlined />}
                        onClick={handleResume}
                      >
                        继续
                      </Button>
                    ) : (
                      <Button
                        icon={<PauseCircleOutlined />}
                        onClick={handlePause}
                      >
                        暂停
                      </Button>
                    )}
                    <Button
                      danger
                      icon={<StopOutlined />}
                      onClick={handleStop}
                    >
                      停止
                    </Button>
                  </>
                )}
              </Space>
            }
          >
            <Space direction="vertical" style={{ width: "100%" }} size="small">
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                }}
              >
                <Text style={{ width: 80, flexShrink: 0 }}>
                  <SoundOutlined /> 播放速度
                </Text>
                <Select
                  value={speed}
                  onChange={setSpeed}
                  options={speedOptions}
                  disabled={isRunning}
                  size="small"
                  style={{ width: 100 }}
                />
              </div>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                }}
              >
                <Text style={{ width: 80, flexShrink: 0 }}>
                  <ThunderboltOutlined /> 并发数
                </Text>
                <Select
                  value={jobCount}
                  onChange={setJobCount}
                  options={jobCountOptions}
                  disabled={isRunning}
                  size="small"
                  style={{ width: 100 }}
                />
              </div>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                }}
              >
                <Text style={{ width: 80, flexShrink: 0 }}>未开放课程</Text>
                <Select
                  value={notopenAction}
                  onChange={setNotopenAction}
                  options={notopenOptions}
                  disabled={isRunning}
                  size="small"
                  style={{ width: 100 }}
                />
              </div>
            </Space>

            {selectedCourses.length === 0 && !isRunning && (
              <div style={{ marginTop: 12 }}>
                <Text type="warning">
                  尚未选择课程，请先前往课程列表页选择
                </Text>
              </div>
            )}
          </Card>

          {/* 课程进度列表 */}
          <Card
            title="课程进度"
            size="small"
            style={{ flex: 1, overflow: "hidden", display: "flex", flexDirection: "column" }}
            styles={{ body: { flex: 1, overflow: "auto", padding: "8px 16px" } }}
          >
            {progressEntries.length === 0 ? (
              <Empty
                description={
                  isRunning ? "等待课程开始..." : "暂无进度数据"
                }
                image={Empty.PRESENTED_IMAGE_SIMPLE}
              />
            ) : (
              <List
                size="small"
                dataSource={progressEntries}
                renderItem={(item) => {
                  const percent =
                    item.totalChapters > 0
                      ? Math.round(
                          (item.completedChapters / item.totalChapters) * 100,
                        )
                      : 0;
                  return (
                    <List.Item style={{ padding: "8px 0" }}>
                      <div style={{ width: "100%" }}>
                        <div
                          style={{
                            display: "flex",
                            justifyContent: "space-between",
                            alignItems: "center",
                            marginBottom: 4,
                          }}
                        >
                          <Tooltip title={item.courseTitle}>
                            <Text
                              strong
                              ellipsis
                              style={{ maxWidth: "70%" }}
                            >
                              {item.courseTitle}
                            </Text>
                          </Tooltip>
                          {getCourseStatusTag(item.status)}
                        </div>
                        <Progress
                          percent={percent}
                          size="small"
                          format={() =>
                            `${item.completedChapters}/${item.totalChapters}`
                          }
                          status={
                            item.status === "error"
                              ? "exception"
                              : item.status === "completed"
                                ? "success"
                                : "active"
                          }
                        />
                      </div>
                    </List.Item>
                  );
                }}
              />
            )}
          </Card>

          {/* 视频播放进度 */}
          {videoEntries.length > 0 && (
            <Card title="播放进度" size="small">
              <List
                size="small"
                dataSource={videoEntries}
                renderItem={(item) => {
                  const percent =
                    item.totalDuration > 0
                      ? Math.round(
                          (item.currentTime / item.totalDuration) * 100,
                        )
                      : 0;
                  return (
                    <List.Item style={{ padding: "4px 0" }}>
                      <div style={{ width: "100%" }}>
                        <Text
                          ellipsis
                          style={{ fontSize: 12, marginBottom: 2 }}
                        >
                          {item.jobName}
                        </Text>
                        <Progress
                          percent={percent}
                          size="small"
                          format={() =>
                            `${formatTime(item.currentTime)} / ${formatTime(item.totalDuration)}`
                          }
                          status="active"
                          strokeColor="#52c41a"
                        />
                      </div>
                    </List.Item>
                  );
                }}
              />
            </Card>
          )}
        </div>

        {/* 右侧日志面板 */}
        <Card
          title="运行日志"
          size="small"
          style={{
            flex: 1,
            display: "flex",
            flexDirection: "column",
            overflow: "hidden",
          }}
          styles={{ body: { flex: 1, overflow: "hidden", display: "flex", flexDirection: "column", padding: 0 } }}
          extra={
            <Space size="small">
              <Segmented
                size="small"
                value={logFilter}
                onChange={(val) =>
                  setLogFilter(val as "all" | "info" | "warn" | "error")
                }
                options={[
                  { value: "all", label: "全部" },
                  { value: "info", label: "信息" },
                  { value: "warn", label: "警告" },
                  { value: "error", label: "错误" },
                ]}
              />
              <Button
                size="small"
                icon={<ClearOutlined />}
                onClick={clearLogs}
              >
                清空
              </Button>
            </Space>
          }
        >
          <div
            style={{
              flex: 1,
              overflow: "auto",
              padding: "8px 16px",
              fontFamily: "Consolas, Monaco, 'Courier New', monospace",
              fontSize: 12,
              lineHeight: 1.6,
              background: "#fafafa",
            }}
          >
            {filteredLogs.length === 0 ? (
              <Empty
                description="暂无日志"
                image={Empty.PRESENTED_IMAGE_SIMPLE}
                style={{ marginTop: 48 }}
              />
            ) : (
              filteredLogs.map((log, index) => (
                <LogLine key={index} log={log} />
              ))
            )}
            <div ref={logEndRef} />
          </div>

          {/* 日志统计栏 */}
          <Divider style={{ margin: 0 }} />
          <div
            style={{
              padding: "6px 16px",
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              fontSize: 12,
              color: "#999",
            }}
          >
            <span>共 {logs.length} 条日志</span>
            <span>
              显示 {filteredLogs.length} 条
              {logFilter !== "all" ? ` (${logFilter})` : ""}
            </span>
          </div>
        </Card>
      </div>
    </div>
  );
}

/** 单条日志行 */
function LogLine({ log }: { log: LogEntry }) {
  return (
    <div
      style={{
        display: "flex",
        gap: 8,
        padding: "1px 0",
        color: log.level === "error" ? "#ff4d4f" : log.level === "warn" ? "#faad14" : "#333",
      }}
    >
      <Text type="secondary" style={{ fontSize: 12, flexShrink: 0 }}>
        {log.timestamp}
      </Text>
      <Tag
        color={getLogTagColor(log.level)}
        style={{ fontSize: 10, lineHeight: "16px", padding: "0 4px", margin: 0 }}
      >
        {log.level.toUpperCase()}
      </Tag>
      <span style={{ wordBreak: "break-all" }}>{log.message}</span>
    </div>
  );
}
