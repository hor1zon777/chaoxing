import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { Empty, Progress, Select, Tooltip, message } from "antd";
import {
  PlayCircleOutlined,
  PauseCircleOutlined,
  StopOutlined,
  ClearOutlined,
} from "@ant-design/icons";
import { useTaskStore } from "../stores/taskStore";
import { useCourseStore } from "../stores/courseStore";
import type { TaskEvent, LogEntry } from "../types/task";
import { Card, Chip, Eyebrow, Headline, Metric, PillButton, Tag, UtilityButton } from "../components/ui/appleUI";

function formatTime(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

function getLogTone(level: string): "default" | "warning" | "danger" | "neutral" {
  if (level === "error") return "danger";
  if (level === "warn") return "warning";
  if (level === "info") return "default";
  return "neutral";
}

function getCourseStatusTag(status: string) {
  switch (status) {
    case "running":
      return <Tag>进行中</Tag>;
    case "completed":
      return <Tag tone="success">已完成</Tag>;
    case "error":
      return <Tag tone="danger">出错</Tag>;
    case "pending":
      return <Tag tone="warning">等待中</Tag>;
    default:
      return <Tag tone="neutral">{status}</Tag>;
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

type LogFilterValue = "all" | "info" | "warn" | "error";

export function TaskPage() {
  const [speed, setSpeed] = useState(1.0);
  const [jobCount, setJobCount] = useState(4);
  const [notopenAction, setNotopenAction] = useState<"retry" | "continue">("retry");
  const [isActionPending, setIsActionPending] = useState(false);

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
    const lastLogKey = lastLog ? `${lastLog.timestamp}-${lastLog.level}-${lastLog.message}` : null;
    if (!container) {
      previousLastLogRef.current = lastLogKey;
      return;
    }
    if (!hasInitializedLogScrollRef.current) {
      hasInitializedLogScrollRef.current = true;
      previousLastLogRef.current = lastLogKey;
      if (lastLogKey) container.scrollTop = container.scrollHeight;
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
    if (isActionPending) return;
    if (selectedCourses.length === 0) {
      message.warning("请先进入课程配置页选择任务后再开始学习");
      return;
    }
    const courseSelections = getSelectedLearningSelections();
    const invalidSelection = courseSelections.find(
      (course) => course.selectedPoints.length === 0,
    );
    if (invalidSelection) {
      message.warning(`请先为课程"${invalidSelection.title}"选择章节或任务`);
      return;
    }

    setIsActionPending(true);
    reset();
    setRunning(true);

    const channel = new Channel<TaskEvent>();
    channel.onmessage = (event: TaskEvent) => handleTaskEvent(event);
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
    } finally {
      setIsActionPending(false);
    }
  }, [
    isActionPending,
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
    if (isActionPending) return;
    setIsActionPending(true);
    try {
      await invoke("pause_tasks");
      setPaused(true);
      addLog("info", "任务已暂停");
    } catch (e: unknown) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      addLog("error", `暂停失败: ${errorMsg}`);
    } finally {
      setIsActionPending(false);
    }
  }, [isActionPending, setPaused, addLog]);

  const handleResume = useCallback(async () => {
    if (isActionPending) return;
    setIsActionPending(true);
    try {
      await invoke("resume_tasks");
      setPaused(false);
      addLog("info", "任务已恢复");
    } catch (e: unknown) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      addLog("error", `恢复失败: ${errorMsg}`);
    } finally {
      setIsActionPending(false);
    }
  }, [isActionPending, setPaused, addLog]);

  const handleStop = useCallback(async () => {
    if (isActionPending) return;
    setIsActionPending(true);
    try {
      await invoke("cancel_tasks");
      setRunning(false);
      setPaused(false);
      channelRef.current = null;
      addLog("warn", "任务已手动停止");
    } catch (e: unknown) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      addLog("error", `停止失败: ${errorMsg}`);
    } finally {
      setIsActionPending(false);
    }
  }, [isActionPending, setRunning, setPaused, addLog]);

  const filteredLogs = useMemo(() => {
    if (logFilter === "all") return logs;
    if (logFilter === "error") return logs.filter((l) => l.level === "error");
    if (logFilter === "warn") return logs.filter((l) => l.level === "warn" || l.level === "error");
    if (logFilter === "info") return logs.filter((l) => l.level === "info");
    return logs;
  }, [logs, logFilter]);

  const progressEntries = useMemo(() => Object.values(courseProgress), [courseProgress]);
  const totalChapters = progressEntries.reduce((sum, p) => sum + p.totalChapters, 0);
  const completedChapters = progressEntries.reduce((sum, p) => sum + p.completedChapters, 0);
  const overallPercent =
    totalChapters > 0 ? Math.round((completedChapters / totalChapters) * 100) : 0;

  const videoEntries = useMemo(() => Object.values(videoProgress), [videoProgress]);

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
      return { label: "已暂停", description: "任务已暂停，可继续或停止。" };
    }
    if (isRunning) {
      return { label: "进行中", description: "正在执行已配置课程任务。" };
    }
    return {
      label: "待开始",
      description:
        selectedCourses.length > 0
          ? "已完成学习范围配置，可直接开始。"
          : "请先前往课程页配置任务，再回到此处执行。",
    };
  }, [isRunning, isPaused, selectedCourses.length]);

  return (
    <div style={{ background: "var(--apple-color-canvas)" }}>
      {/* 任务控制台 */}
      <section
        style={{
          padding: "32px 22px",
          background: "var(--apple-color-canvas)",
          borderBottom: "1px solid var(--apple-color-divider-soft)",
        }}
      >
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 8, marginBottom: 12 }}>
            <Tag>{statusMeta.label}</Tag>
            <Tag tone="neutral">已选 {selectedCourses.length} 门课程</Tag>
            <Tag tone="neutral">并发 {jobCount}</Tag>
          </div>

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
              <span className="apple-eyebrow">
                任务控制台
              </span>
              <Headline level="lg" style={{ marginTop: 10 }}>
                {isRunning
                  ? isPaused
                    ? "继续未完成的任务。"
                    : "正在执行任务。"
                  : "开始学习。"}
              </Headline>
              <p
                style={{
                  fontFamily: "var(--apple-font-text)",
                  fontSize: 17,
                  color: "var(--apple-color-ink-muted-48)",
                  lineHeight: 1.47,
                  letterSpacing: "-0.374px",
                  margin: "14px 0 0",
                  maxWidth: 640,
                }}
              >
                {statusMeta.description}
              </p>
            </div>

            <div style={{ display: "flex", gap: 12, flexWrap: "wrap" }}>
              {!isRunning ? (
                <PillButton
                  large
                  onClick={() => void handleStart()}
                  disabled={selectedCourses.length === 0 || isActionPending}
                  icon={<PlayCircleOutlined />}
                >
                  开始学习
                </PillButton>
              ) : (
                <>
                  {isPaused ? (
                    <PillButton
                      onClick={() => void handleResume()}
                      disabled={isActionPending}
                      icon={<PlayCircleOutlined />}
                    >
                      继续
                    </PillButton>
                  ) : (
                    <PillButton
                      variant="ghost"
                      onClick={() => void handlePause()}
                      disabled={isActionPending}
                      icon={<PauseCircleOutlined />}
                    >
                      暂停
                    </PillButton>
                  )}
                  <PillButton
                    variant="danger"
                    onClick={() => void handleStop()}
                    disabled={isActionPending}
                    icon={<StopOutlined />}
                  >
                    停止
                  </PillButton>
                </>
              )}
            </div>
          </div>

          {/* Metrics row */}
          <div
            style={{
              marginTop: 28,
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))",
              gap: 16,
            }}
          >
            <Metric label="学习课程" value={`${selectedCourses.length}`} hint="已完成任务配置" />
            <Metric label="学习章节" value={`${totalSelectedPoints}`} hint="本次计划覆盖" />
            <Metric label="任务点" value={`${totalSelectedJobs}`} hint="等待执行" />
            <Metric
              label={isRunning ? "总体进度" : "当前状态"}
              value={
                isRunning && totalChapters > 0
                  ? `${overallPercent}%`
                  : selectedCourses.length > 0
                    ? "就绪"
                    : "待配置"
              }
              hint={
                isRunning && totalChapters > 0
                  ? `${completedChapters}/${totalChapters} 章节`
                  : "可随时开始"
              }
            />
          </div>

          {(isRunning || totalSelectedJobs > 0) && (
            <div style={{ marginTop: 28 }}>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  marginBottom: 10,
                  gap: 12,
                  flexWrap: "wrap",
                }}
              >
                <span
                  style={{
                    fontFamily: "var(--apple-font-text)",
                    fontSize: 14,
                    fontWeight: 600,
                    letterSpacing: "-0.224px",
                  }}
                >
                  {isRunning ? "任务总体进度" : "已配置学习范围"}
                </span>
                <span
                  style={{
                    fontFamily: "var(--apple-font-text)",
                    fontSize: 14,
                    color: "var(--apple-color-ink-muted-48)",
                    letterSpacing: "-0.224px",
                  }}
                >
                  {isRunning
                    ? `${completedChapters} / ${totalChapters || totalSelectedPoints || 0} 章节`
                    : `共 ${totalSelectedJobs} 个任务点待执行`}
                </span>
              </div>
              <Progress
                percent={isRunning && totalChapters > 0 ? overallPercent : totalSelectedJobs > 0 ? 100 : 0}
                status={isRunning ? (overallPercent === 100 ? "success" : "active") : "normal"}
                showInfo={false}
              />
            </div>
          )}
        </div>
      </section>

      {/* Light section - 参数与范围 */}
      <section style={{ padding: "48px 22px", background: "var(--apple-color-canvas)" }}>
        <div
          style={{
            maxWidth: 1024,
            margin: "0 auto",
            display: "grid",
            gridTemplateColumns: "repeat(auto-fit, minmax(320px, 1fr))",
            gap: 20,
          }}
        >
          <Card title="执行参数" subtitle="开始前调整播放速度、并发与未开放策略">
            <div style={{ display: "flex", flexDirection: "column", gap: 18, marginTop: 12 }}>
              <ControlField label="播放速度" description="模拟视频任务的播放倍速">
                <Select
                  value={speed}
                  onChange={setSpeed}
                  options={speedOptions}
                  disabled={isRunning}
                  style={{ minWidth: 140 }}
                />
              </ControlField>
              <ControlField label="并发数" description="同时执行的任务线程数">
                <Select
                  value={jobCount}
                  onChange={setJobCount}
                  options={jobCountOptions}
                  disabled={isRunning}
                  style={{ minWidth: 140 }}
                />
              </ControlField>
              <ControlField label="未开放课程" description="遇到暂未开放内容时的处理方式">
                <Select
                  value={notopenAction}
                  onChange={setNotopenAction}
                  options={notopenOptions}
                  disabled={isRunning}
                  style={{ minWidth: 160 }}
                />
              </ControlField>
            </div>
          </Card>

          <Card title="本次学习范围" subtitle="确认每门课程的章节和任务">
            <div style={{ marginTop: 12 }}>
              {selectionSummary.length === 0 ? (
                <Empty
                  description="尚未配置课程任务"
                  image={Empty.PRESENTED_IMAGE_SIMPLE}
                  style={{ marginBlock: 12 }}
                />
              ) : (
                <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                  {selectionSummary.map((item) => (
                    <div
                      key={item.courseId}
                      style={{
                        padding: "14px 16px",
                        borderRadius: 11,
                        background: "var(--apple-color-canvas-parchment)",
                        border: "1px solid var(--apple-color-divider-soft)",
                      }}
                    >
                      <div
                        style={{
                          display: "flex",
                          justifyContent: "space-between",
                          gap: 12,
                          alignItems: "flex-start",
                        }}
                      >
                        <span
                          style={{
                            fontFamily: "var(--apple-font-text)",
                            fontSize: 15,
                            fontWeight: 600,
                            letterSpacing: "-0.224px",
                            lineHeight: 1.4,
                            wordBreak: "break-word",
                          }}
                        >
                          {item.title}
                        </span>
                        <Tag>{item.pointCount} 章节</Tag>
                      </div>
                      <div
                        style={{
                          fontFamily: "var(--apple-font-text)",
                          fontSize: 12,
                          letterSpacing: "-0.12px",
                          color: "var(--apple-color-ink-muted-48)",
                          marginTop: 8,
                        }}
                      >
                        已选 {item.pointCount} 个章节 / {item.selectedJobCount} 个任务点
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </Card>
        </div>
      </section>

      {/* Parchment section - 进度 */}
      <section
        style={{
          padding: "48px 22px",
          background: "var(--apple-color-canvas-parchment)",
        }}
      >
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          <div style={{ marginBottom: 24 }}>
            <Eyebrow>实时进度</Eyebrow>
            <Headline level="md" style={{ marginTop: 8 }}>
              课程进度与播放状态
            </Headline>
          </div>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(360px, 1fr))",
              gap: 20,
            }}
          >
            <Card title="课程进度" subtitle="每门课程的执行状态与章节情况">
              <div style={{ marginTop: 12, minHeight: 180 }}>
                {progressEntries.length === 0 ? (
                  <Empty
                    description={isRunning ? "等待课程开始..." : "暂无进度数据"}
                    image={Empty.PRESENTED_IMAGE_SIMPLE}
                  />
                ) : (
                  <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                    {progressEntries.map((item) => {
                      const percent =
                        item.totalChapters > 0
                          ? Math.round((item.completedChapters / item.totalChapters) * 100)
                          : 0;
                      return (
                        <div
                          key={item.courseId}
                          style={{
                            padding: "12px 14px",
                            borderRadius: 11,
                            background: "var(--apple-color-canvas)",
                            border: "1px solid var(--apple-color-divider-soft)",
                          }}
                        >
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
                              <span
                                style={{
                                  fontFamily: "var(--apple-font-text)",
                                  fontSize: 14,
                                  fontWeight: 600,
                                  letterSpacing: "-0.224px",
                                  maxWidth: "72%",
                                  overflow: "hidden",
                                  textOverflow: "ellipsis",
                                  whiteSpace: "nowrap",
                                }}
                              >
                                {item.courseTitle}
                              </span>
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
                      );
                    })}
                  </div>
                )}
              </div>
            </Card>

            <Card title="播放进度" subtitle="视频任务的当前播放位置">
              <div style={{ marginTop: 12, minHeight: 180 }}>
                {videoEntries.length === 0 ? (
                  <Empty
                    description={
                      isRunning ? "当前暂无视频播放任务" : "开始任务后将显示视频播放进度"
                    }
                    image={Empty.PRESENTED_IMAGE_SIMPLE}
                  />
                ) : (
                  <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                    {videoEntries.map((item) => {
                      const percent =
                        item.totalDuration > 0
                          ? Math.round((item.currentTime / item.totalDuration) * 100)
                          : 0;
                      return (
                        <div
                          key={item.jobId}
                          style={{
                            padding: "12px 14px",
                            borderRadius: 11,
                            background: "var(--apple-color-canvas)",
                            border: "1px solid var(--apple-color-divider-soft)",
                          }}
                        >
                          <span
                            style={{
                              fontFamily: "var(--apple-font-text)",
                              fontSize: 14,
                              fontWeight: 600,
                              letterSpacing: "-0.224px",
                              display: "block",
                              marginBottom: 10,
                              overflow: "hidden",
                              textOverflow: "ellipsis",
                              whiteSpace: "nowrap",
                            }}
                          >
                            {item.jobName}
                          </span>
                          <Progress
                            percent={percent}
                            size="small"
                            format={() =>
                              `${formatTime(item.currentTime)} / ${formatTime(item.totalDuration)}`
                            }
                            status="active"
                          />
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            </Card>
          </div>
        </div>
      </section>

      {/* Light section - 日志 */}
      <section style={{ padding: "48px 22px 80px", background: "var(--apple-color-canvas)" }}>
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "flex-end",
              gap: 16,
              flexWrap: "wrap",
              marginBottom: 20,
            }}
          >
            <div>
              <Eyebrow>运行日志</Eyebrow>
              <Headline level="md" style={{ marginTop: 8 }}>
                实时定位执行状态
              </Headline>
            </div>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 10, alignItems: "center" }}>
              {(["all", "info", "warn", "error"] as LogFilterValue[]).map((value) => (
                <Chip
                  key={value}
                  selected={logFilter === value}
                  onClick={() => setLogFilter(value)}
                >
                  {value === "all"
                    ? "全部"
                    : value === "info"
                      ? "信息"
                      : value === "warn"
                        ? "警告"
                        : "错误"}
                </Chip>
              ))}
              <UtilityButton light onClick={clearLogs} icon={<ClearOutlined />}>
                清空
              </UtilityButton>
            </div>
          </div>

          <Card padding={0}>
            <div
              style={{
                padding: "16px 20px",
                borderBottom: "1px solid var(--apple-color-divider-soft)",
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                gap: 12,
                flexWrap: "wrap",
              }}
            >
              <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
                <Tag tone="neutral">总计 {logs.length}</Tag>
                <Tag>信息 {logStats.info}</Tag>
                <Tag tone="warning">警告 {logStats.warn}</Tag>
                <Tag tone="danger">错误 {logStats.error}</Tag>
              </div>
              <span
                style={{
                  fontFamily: "var(--apple-font-text)",
                  fontSize: 12,
                  letterSpacing: "-0.12px",
                  color: "var(--apple-color-ink-muted-48)",
                }}
              >
                当前显示 {filteredLogs.length} 条
              </span>
            </div>

            <div
              ref={logContainerRef}
              style={{
                maxHeight: 480,
                minHeight: 280,
                overflow: "auto",
                padding: "16px 20px",
                fontFamily: '"SF Mono", Consolas, Monaco, monospace',
                fontSize: 13,
                lineHeight: 1.7,
                background: "var(--apple-color-canvas-parchment)",
              }}
            >
              {filteredLogs.length === 0 ? (
                <Empty
                  description="暂无日志"
                  image={Empty.PRESENTED_IMAGE_SIMPLE}
                  style={{ marginTop: 32 }}
                />
              ) : (
                filteredLogs.map((log, index) => <LogLine key={index} log={log} />)
              )}
            </div>
          </Card>
        </div>
      </section>
    </div>
  );
}

interface ControlFieldProps {
  label: string;
  description: string;
  children: React.ReactNode;
}

function ControlField({ label, description, children }: ControlFieldProps) {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        gap: 16,
        flexWrap: "wrap",
        padding: "14px 16px",
        borderRadius: 11,
        background: "var(--apple-color-canvas-parchment)",
        border: "1px solid var(--apple-color-divider-soft)",
      }}
    >
      <div style={{ minWidth: 180, flex: 1 }}>
        <div
          style={{
            fontFamily: "var(--apple-font-text)",
            fontSize: 15,
            fontWeight: 600,
            letterSpacing: "-0.224px",
            color: "var(--apple-color-ink)",
          }}
        >
          {label}
        </div>
        <div
          style={{
            fontFamily: "var(--apple-font-text)",
            fontSize: 12,
            letterSpacing: "-0.12px",
            color: "var(--apple-color-ink-muted-48)",
            marginTop: 4,
            lineHeight: 1.5,
          }}
        >
          {description}
        </div>
      </div>
      {children}
    </div>
  );
}

function LogLine({ log }: { log: LogEntry }) {
  const tone = getLogTone(log.level);
  return (
    <div
      style={{
        display: "flex",
        alignItems: "flex-start",
        gap: 12,
        padding: "8px 10px",
        marginBottom: 6,
        borderRadius: 8,
        background: "var(--apple-color-canvas)",
        border: "1px solid var(--apple-color-divider-soft)",
      }}
    >
      <span
        style={{
          fontFamily: '"SF Mono", monospace',
          fontSize: 11,
          color: "var(--apple-color-ink-muted-48)",
          flexShrink: 0,
          minWidth: 72,
        }}
      >
        {log.timestamp}
      </span>
      <Tag tone={tone} style={{ fontSize: 10, padding: "1px 8px", flexShrink: 0 }}>
        {log.level.toUpperCase()}
      </Tag>
      <span
        style={{
          wordBreak: "break-all",
          flex: 1,
          color: "var(--apple-color-ink)",
          fontFamily: '"SF Mono", monospace',
          fontSize: 12.5,
        }}
      >
        {log.message}
      </span>
    </div>
  );
}
