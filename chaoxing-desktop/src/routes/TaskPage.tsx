import { useState, useRef, useEffect, useCallback, useMemo, type ReactNode } from "react";
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
  Grid,
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
import {
  GuideBanner,
  pageTopBannerTexts,
  panelStyle,
  primaryActionButtonStyle,
  SectionTitle,
  SummaryMetric,
  surfaceCardStyle,
} from "../components/ui/pagePrimitives";

const { Text, Title } = Typography;
const { useBreakpoint } = Grid;

function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

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

function getCourseStatusTag(status: string) {
  switch (status) {
    case "running":
      return (
        <Tag
          color="processing"
          style={{ marginInlineEnd: 0, background: "#e0f2fe", color: "#0369a1", borderColor: "#bae6fd" }}
        >
          进行中
        </Tag>
      );
    case "completed":
      return (
        <Tag
          color="success"
          style={{ marginInlineEnd: 0, background: "#dcfce7", color: "#166534", borderColor: "#bbf7d0" }}
        >
          已完成
        </Tag>
      );
    case "error":
      return (
        <Tag
          color="error"
          style={{ marginInlineEnd: 0, background: "#fee2e2", color: "#b91c1c", borderColor: "#fecaca" }}
        >
          出错
        </Tag>
      );
    case "pending":
      return (
        <Tag
          style={{ marginInlineEnd: 0, background: "#fef3c7", color: "#92400e", borderColor: "#fde68a" }}
        >
          等待中
        </Tag>
      );
    default:
      return <Tag style={{ marginInlineEnd: 0 }}>{status}</Tag>;
  }
}

const speedOptions = [
  { value: 1.0, label: "1.0x" },
  { value: 1.25, label: "1.25x" },
  { value: 1.5, label: "1.5x" },
  { value: 1.75, label: "1.75x" },
  { value: 2.0, label: "2.0x" },
];

const jobCountOptions = [
  { value: 1, label: "1 线程" },
  { value: 2, label: "2 线程" },
  { value: 4, label: "4 线程" },
  { value: 6, label: "6 线程" },
  { value: 8, label: "8 线程" },
];

const notopenOptions = [
  { value: "retry", label: "重试" },
  { value: "continue", label: "跳过继续" },
];

export function TaskPage() {
  const screens = useBreakpoint();
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

  const { courses, selectedCourseIds, learningSelections, getSelectedLearningSelections } =
    useCourseStore();
  const logContainerRef = useRef<HTMLDivElement>(null);
  const channelRef = useRef<Channel<TaskEvent> | null>(null);
  const hasInitializedLogScrollRef = useRef(false);
  const previousLastLogRef = useRef<string | null>(null);

  useEffect(() => {
    const container = logContainerRef.current;
    const lastLog = logs.length > 0 ? logs[logs.length - 1] : null;
    const lastLogKey = lastLog
      ? `${lastLog.timestamp}-${lastLog.level}-${lastLog.message}`
      : null;

    if (!container) {
      previousLastLogRef.current = lastLogKey;
      return;
    }

    if (!hasInitializedLogScrollRef.current) {
      hasInitializedLogScrollRef.current = true;
      previousLastLogRef.current = lastLogKey;
      if (lastLogKey) {
        container.scrollTop = container.scrollHeight;
      }
      return;
    }

    if (lastLogKey && lastLogKey !== previousLastLogRef.current) {
      container.scrollTo({
        top: container.scrollHeight,
        behavior: logs.length > 20 ? "auto" : "smooth",
      });
    }

    previousLastLogRef.current = lastLogKey;
  }, [logs]);

  const selectedCourses = useMemo(
    () => courses.filter((c) => selectedCourseIds.includes(c.id)),
    [courses, selectedCourseIds],
  );

  const selectionSummary = useMemo(() => {
    return selectedCourses.map((course) => {
      const selection = learningSelections[course.id];
      const pointCount = selection?.selectedPoints.length ?? 0;
      const selectedJobCount =
        selection?.selectedPoints.reduce(
          (sum, point) => sum + point.selectedJobIds.length,
          0,
        ) ?? 0;
      return {
        courseId: course.id,
        title: course.title,
        pointCount,
        selectedJobCount,
      };
    });
  }, [selectedCourses, learningSelections]);

  const handleStart = useCallback(async () => {
    if (selectedCourses.length === 0) {
      message.warning("请先进入课程配置页选择任务后再开始学习");
      return;
    }

    const courseSelections = getSelectedLearningSelections();
    const invalidSelection = courseSelections.find(
      (course) => course.selectedPoints.length === 0,
    );
    if (invalidSelection) {
      message.warning(`请先为课程“${invalidSelection.title}”选择章节或任务`);
      return;
    }

    reset();
    setRunning(true);

    const channel = new Channel<TaskEvent>();
    channel.onmessage = (event: TaskEvent) => {
      handleTaskEvent(event);
    };
    channelRef.current = channel;

    try {
      await invoke("start_course_tasks", {
        channel,
        courses: courseSelections,
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
    getSelectedLearningSelections,
    speed,
    jobCount,
    notopenAction,
    reset,
    setRunning,
    handleTaskEvent,
    addLog,
  ]);

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

  const filteredLogs = useMemo(() => {
    if (logFilter === "all") return logs;
    if (logFilter === "error") return logs.filter((l) => l.level === "error");
    if (logFilter === "warn") {
      return logs.filter((l) => l.level === "warn" || l.level === "error");
    }
    if (logFilter === "info") {
      return logs.filter((l) => l.level === "info");
    }
    return logs;
  }, [logs, logFilter]);

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

  const videoEntries = useMemo(
    () => Object.values(videoProgress),
    [videoProgress],
  );

  const totalSelectedPoints = useMemo(
    () => selectionSummary.reduce((sum, item) => sum + item.pointCount, 0),
    [selectionSummary],
  );
  const totalSelectedJobs = useMemo(
    () => selectionSummary.reduce((sum, item) => sum + item.selectedJobCount, 0),
    [selectionSummary],
  );
  const logStats = useMemo(() => {
    return logs.reduce(
      (summary, log) => ({
        info: summary.info + (log.level === "info" ? 1 : 0),
        warn: summary.warn + (log.level === "warn" ? 1 : 0),
        error: summary.error + (log.level === "error" ? 1 : 0),
      }),
      { info: 0, warn: 0, error: 0 },
    );
  }, [logs]);

  const statusMeta = useMemo(() => {
    if (isRunning && isPaused) {
      return {
        label: "已暂停",
        color: "warning" as const,
        description: "任务已暂停，可以继续执行或直接停止当前学习流程。",
      };
    }
    if (isRunning) {
      return {
        label: "进行中",
        color: "processing" as const,
        description: "正在执行已配置课程任务，进度和运行日志会实时更新。",
      };
    }
    return {
      label: "待开始",
      color: "default" as const,
      description:
        selectedCourses.length > 0
          ? "已完成学习范围配置，可直接开始学习。"
          : "请先前往课程页配置课程任务，再回到此处执行。",
    };
  }, [isRunning, isPaused, selectedCourses.length]);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 16, height: "100%", paddingBottom: 4 }}>
      <GuideBanner text={pageTopBannerTexts.tasks} />

      <Card
        style={{
          ...surfaceCardStyle,
          background: "linear-gradient(180deg, #ffffff 0%, #fbfdff 100%)",
          boxShadow: "0 16px 32px rgba(15,23,42,0.10)",
          border: "1px solid #dbe5f0",
        }}
        styles={{ body: { padding: 24 } }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "flex-start",
            gap: 16,
            flexWrap: "wrap",
          }}
        >
          <div style={{ maxWidth: 760 }}>
            <Space wrap size={[8, 8]}>
              <Tag
                color={statusMeta.color}
                style={{
                  marginInlineEnd: 0,
                  background: statusMeta.color === "processing" ? "#dbeafe" : statusMeta.color === "warning" ? "#fef3c7" : "#e5e7eb",
                  color: statusMeta.color === "processing" ? "#1d4ed8" : statusMeta.color === "warning" ? "#92400e" : "#374151",
                  borderColor: statusMeta.color === "processing" ? "#bfdbfe" : statusMeta.color === "warning" ? "#fde68a" : "#d1d5db",
                }}
              >
                {statusMeta.label}
              </Tag>
              <Tag
                color="blue"
                style={{ marginInlineEnd: 0, background: "#e0f2fe", color: "#0369a1", borderColor: "#bae6fd" }}
              >
                已选 {selectedCourses.length} 门课程
              </Tag>
              <Tag
                color="purple"
                style={{ marginInlineEnd: 0, background: "#ede9fe", color: "#6d28d9", borderColor: "#ddd6fe" }}
              >
                并发 {jobCount}
              </Tag>
            </Space>
            <Title level={4} style={{ margin: "12px 0 4px" }}>
              学习任务控制台
            </Title>
            <Text type="secondary" style={{ lineHeight: 1.7 }}>
              {isRunning || isPaused ? statusMeta.description : selectedCourses.length > 0 ? "已完成范围配置，可直接开始任务。" : statusMeta.description}
            </Text>
          </div>

          <Space wrap size="small">
            {!isRunning ? (
              <Button
                type="primary"
                icon={<PlayCircleOutlined />}
                onClick={handleStart}
                disabled={selectedCourses.length === 0}
                style={{ ...primaryActionButtonStyle, background: "#2563eb", borderColor: "#2563eb", color: "#ffffff", boxShadow: "0 12px 24px rgba(37,99,235,0.20)" }}
              >
                开始学习
              </Button>
            ) : (
              <>
                {isPaused ? (
                  <Button
                    icon={<PlayCircleOutlined />}
                    onClick={handleResume}
                    style={{ ...primaryActionButtonStyle, minWidth: 110 }}
                  >
                    继续
                  </Button>
                ) : (
                  <Button
                    icon={<PauseCircleOutlined />}
                    onClick={handlePause}
                    style={{ ...primaryActionButtonStyle, minWidth: 110 }}
                  >
                    暂停
                  </Button>
                )}
                <Button
                  danger
                  icon={<StopOutlined />}
                  onClick={handleStop}
                  style={{ ...primaryActionButtonStyle, minWidth: 110 }}
                >
                  停止
                </Button>
              </>
            )}
          </Space>
        </div>

        <div
          style={{
            display: "grid",
            gridTemplateColumns: screens.md ? "repeat(4, minmax(0, 1fr))" : "repeat(2, minmax(0, 1fr))",
            gap: 12,
            marginTop: 20,
          }}
        >
          <SummaryMetric
            label="学习课程"
            value={`${selectedCourses.length} 门`}
            hint="已完成任务配置"
          />
          <SummaryMetric
            label="学习章节"
            value={`${totalSelectedPoints} 个`}
            hint="本次计划覆盖"
          />
          <SummaryMetric
            label="任务点"
            value={`${totalSelectedJobs} 个`}
            hint="等待执行"
          />
          <SummaryMetric
            label={isRunning ? "总体进度" : "当前状态"}
            value={isRunning && totalChapters > 0 ? `${overallPercent}%` : selectedCourses.length > 0 ? "就绪" : "待配置"}
            hint={isRunning && totalChapters > 0 ? `${completedChapters}/${totalChapters} 章节` : "可随时开始"}
          />
        </div>

        {(isRunning || totalSelectedJobs > 0) && (
          <div
            style={{
              ...panelStyle,
              marginTop: 16,
              background: "linear-gradient(180deg, #eff6ff 0%, #ffffff 100%)",
              borderColor: "#bfdbfe",
              boxShadow: "inset 0 1px 0 rgba(255,255,255,0.8)",
            }}
          >
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                gap: 12,
                marginBottom: 10,
                flexWrap: "wrap",
              }}
            >
              <Text strong>{isRunning ? "任务总体进度" : "已配置学习范围"}</Text>
              <Text type="secondary">
                {isRunning
                  ? `${completedChapters} / ${totalChapters || totalSelectedPoints || 0} 章节`
                  : `共 ${totalSelectedJobs} 个任务点待执行`}
              </Text>
            </div>
            <Progress
              percent={isRunning && totalChapters > 0 ? overallPercent : totalSelectedJobs > 0 ? 100 : 0}
              status={isRunning ? (overallPercent === 100 ? "success" : "active") : "normal"}
              strokeColor={isRunning ? undefined : "#91caff"}
              format={() =>
                isRunning && totalChapters > 0
                  ? `${overallPercent}%`
                  : totalSelectedJobs > 0
                    ? "已完成配置"
                    : "待配置"
              }
            />
          </div>
        )}
      </Card>

      <div
        style={{
          display: "flex",
          flex: 1,
          gap: 16,
          minHeight: 0,
          flexDirection: screens.lg ? "row" : "column",
        }}
      >
        <div
          style={{
            width: screens.lg ? (screens.xl ? 420 : 360) : "100%",
            display: "flex",
            flexDirection: "column",
            gap: 16,
            minHeight: 0,
          }}
        >
          <Card
            title={<SectionTitle title="执行参数" subtitle="开始前调整播放速度、并发数和未开放课程策略" />}
            size="small"
            style={surfaceCardStyle}
            styles={{ body: { padding: 16 } }}
          >
            <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
              <ControlField
                icon={<SoundOutlined />}
                label="播放速度"
                description="控制视频任务的模拟播放速度"
              >
                <Select
                  value={speed}
                  onChange={setSpeed}
                  options={speedOptions}
                  disabled={isRunning}
                  style={{ width: 120 }}
                />
              </ControlField>
              <ControlField
                icon={<ThunderboltOutlined />}
                label="并发数"
                description="同时执行的任务线程数"
              >
                <Select
                  value={jobCount}
                  onChange={setJobCount}
                  options={jobCountOptions}
                  disabled={isRunning}
                  style={{ width: 120 }}
                />
              </ControlField>
              <ControlField
                label="未开放课程"
                description="遇到暂未开放内容时的默认处理方式"
              >
                <Select
                  value={notopenAction}
                  onChange={setNotopenAction}
                  options={notopenOptions}
                  disabled={isRunning}
                  style={{ width: 140 }}
                />
              </ControlField>
            </div>
          </Card>

          <Card
            title={<SectionTitle title="本次学习范围" subtitle="确认每门课程的章节和任务选择情况" />}
            size="small"
            style={surfaceCardStyle}
            styles={{ body: { padding: 16 } }}
          >
            {selectionSummary.length === 0 ? (
              <Empty
                description="尚未配置课程任务"
                image={Empty.PRESENTED_IMAGE_SIMPLE}
                style={{ marginBlock: 12 }}
              />
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                {selectionSummary.map((item) => (
                  <div key={item.courseId} style={panelStyle}>
                    <div
                      style={{
                        display: "flex",
                        justifyContent: "space-between",
                        alignItems: "flex-start",
                        gap: 12,
                      }}
                    >
                      <Text strong style={{ flex: 1, lineHeight: 1.6 }}>
                        {item.title}
                      </Text>
                      <Tag color="blue" style={{ marginInlineEnd: 0 }}>
                        {item.pointCount} 章节
                      </Tag>
                    </div>
                    <Text type="secondary" style={{ display: "block", marginTop: 8, fontSize: 12, lineHeight: 1.7 }}>
                      已选 {item.pointCount} 个章节 / {item.selectedJobCount} 个任务点
                    </Text>
                  </div>
                ))}
              </div>
            )}

            {selectedCourses.length === 0 && !isRunning && (
              <div
                style={{
                  ...panelStyle,
                  marginTop: 12,
                  background: "#fffbe6",
                  borderColor: "#ffe58f",
                }}
              >
                <Text type="warning">尚未配置课程任务，请先前往课程页进入任务配置</Text>
              </div>
            )}
          </Card>
        </div>

        <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: 16, minHeight: 0 }}>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: screens.xl ? "minmax(0, 1.2fr) minmax(0, 0.8fr)" : "1fr",
              gap: 16,
            }}
          >
            <Card
              title={<SectionTitle title="课程进度" subtitle="查看每门课程的执行状态与章节完成情况" />}
              size="small"
              style={{ ...surfaceCardStyle, minHeight: 280 }}
              styles={{ body: { padding: "8px 16px 12px", minHeight: 214 } }}
            >
              {progressEntries.length === 0 ? (
                <Empty
                  description={isRunning ? "等待课程开始..." : "暂无进度数据"}
                  image={Empty.PRESENTED_IMAGE_SIMPLE}
                  style={{ marginTop: 32 }}
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
                      <List.Item style={{ padding: "10px 0" }}>
                        <div style={{ width: "100%", ...panelStyle, padding: "12px 14px" }}>
                          <div
                            style={{
                              display: "flex",
                              justifyContent: "space-between",
                              alignItems: "center",
                              gap: 12,
                              marginBottom: 10,
                            }}
                          >
                            <Tooltip title={item.courseTitle}>
                              <Text strong ellipsis style={{ maxWidth: "72%" }}>
                                {item.courseTitle}
                              </Text>
                            </Tooltip>
                            {getCourseStatusTag(item.status)}
                          </div>
                          <Progress
                            percent={percent}
                            size="small"
                            format={() => `${item.completedChapters}/${item.totalChapters}`}
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

            <Card
              title={<SectionTitle title="播放进度" subtitle="跟踪视频任务的播放位置与完成比例" />}
              size="small"
              style={{ ...surfaceCardStyle, minHeight: 280 }}
              styles={{ body: { padding: "8px 16px 12px", minHeight: 214 } }}
            >
              {videoEntries.length === 0 ? (
                <Empty
                  description={isRunning ? "当前暂无视频播放任务" : "开始任务后将显示视频播放进度"}
                  image={Empty.PRESENTED_IMAGE_SIMPLE}
                  style={{ marginTop: 32 }}
                />
              ) : (
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
                      <List.Item style={{ padding: "10px 0" }}>
                        <div style={{ width: "100%", ...panelStyle, padding: "12px 14px" }}>
                          <Text ellipsis strong style={{ display: "block", marginBottom: 10 }}>
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
              )}
            </Card>
          </div>

          <Card
            title={<SectionTitle title="运行日志" subtitle="按级别筛选日志，快速定位执行状态与错误信息" />}
            size="small"
            style={{
              ...surfaceCardStyle,
              flex: 1,
              display: "flex",
              flexDirection: "column",
              overflow: "hidden",
            }}
            styles={{
              body: {
                flex: 1,
                overflow: "hidden",
                display: "flex",
                flexDirection: "column",
                padding: 0,
              },
            }}
            extra={
              <Space size="small" wrap>
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
                <Button size="small" icon={<ClearOutlined />} onClick={clearLogs}>
                  清空
                </Button>
              </Space>
            }
          >
            <div
              style={{
                padding: "12px 16px",
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                gap: 12,
                flexWrap: "wrap",
                borderBottom: "1px solid #eef2f6",
                background: "#fcfcfd",
              }}
            >
              <Space wrap size={[8, 8]}>
                <Tag bordered={false} style={{ background: "#e5e7eb", color: "#374151" }}>总计 {logs.length}</Tag>
                <Tag color="blue" bordered={false} style={{ background: "#dbeafe", color: "#1d4ed8" }}>信息 {logStats.info}</Tag>
                <Tag color="orange" bordered={false} style={{ background: "#fef3c7", color: "#92400e" }}>警告 {logStats.warn}</Tag>
                <Tag color="red" bordered={false} style={{ background: "#fee2e2", color: "#b91c1c" }}>错误 {logStats.error}</Tag>
              </Space>
              <Text type="secondary" style={{ fontSize: 12 }}>
                当前显示 {filteredLogs.length} 条{logFilter !== "all" ? `（${logFilter}）` : ""}
              </Text>
            </div>

            <div
              ref={logContainerRef}
              style={{
                flex: 1,
                overflow: "auto",
                padding: "10px 16px 12px",
                fontFamily: "Consolas, Monaco, 'Courier New', monospace",
                fontSize: 12,
                lineHeight: 1.7,
                background: "#f8fafc",
              }}
            >
              {filteredLogs.length === 0 ? (
                <Empty
                  description="暂无日志"
                  image={Empty.PRESENTED_IMAGE_SIMPLE}
                  style={{ marginTop: 56 }}
                />
              ) : (
                filteredLogs.map((log, index) => <LogLine key={index} log={log} />)
              )}
            </div>

            <Divider style={{ margin: 0 }} />
            <div
              style={{
                padding: "8px 16px",
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                gap: 12,
                fontSize: 12,
                color: "#8c8c8c",
                flexWrap: "wrap",
              }}
            >
              <span>最近日志会自动滚动到最新位置</span>
              <span>共 {logs.length} 条日志记录</span>
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}

function ControlField({
  icon,
  label,
  description,
  children,
}: {
  icon?: ReactNode;
  label: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <div style={panelStyle}>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          gap: 12,
          flexWrap: "wrap",
        }}
      >
        <div style={{ minWidth: 180, flex: 1 }}>
          <Text strong>
            {icon ? (
              <Space size={6}>
                <span>{icon}</span>
                <span>{label}</span>
              </Space>
            ) : (
              label
            )}
          </Text>
          <Text type="secondary" style={{ display: "block", marginTop: 4, fontSize: 12, lineHeight: 1.6 }}>
            {description}
          </Text>
        </div>
        {children}
      </div>
    </div>
  );
}

function LogLine({ log }: { log: LogEntry }) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "flex-start",
        gap: 10,
        padding: "8px 10px",
        marginBottom: 8,
        borderRadius: 12,
        background:
          log.level === "error"
            ? "#fff2f0"
            : log.level === "warn"
              ? "#fffbe6"
              : "#ffffff",
        border:
          log.level === "error"
            ? "1px solid #ffccc7"
            : log.level === "warn"
              ? "1px solid #ffe58f"
              : "1px solid #f0f0f0",
        color:
          log.level === "error"
            ? "#cf1322"
            : log.level === "warn"
              ? "#ad6800"
              : "#262626",
      }}
    >
      <Text type="secondary" style={{ fontSize: 12, flexShrink: 0, minWidth: 64 }}>
        {log.timestamp}
      </Text>
      <Tag
        color={getLogTagColor(log.level)}
        style={{ fontSize: 10, lineHeight: "16px", padding: "0 4px", margin: 0, flexShrink: 0 }}
      >
        {log.level.toUpperCase()}
      </Tag>
      <span style={{ wordBreak: "break-all", flex: 1 }}>{log.message}</span>
    </div>
  );
}
