import { useEffect, useMemo, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { Alert, Empty, message, Progress } from "antd";
import { ReloadOutlined } from "@ant-design/icons";
import { open as openExternal } from "@tauri-apps/plugin-shell";
import { useCourseStore } from "../stores/courseStore";
import { useTopicStore } from "../stores/topicStore";
import { useAuthStore } from "../stores/authStore";
import type { AddTopicResult, TopicListItem } from "../types/topic";
import { Card, Chip, Eyebrow, Headline, Metric, PillButton, Tag, UtilityButton } from "../components/ui/appleUI";
import { ComposeTopicModal } from "../components/topic/ComposeTopicModal";

/** 话题范围：全部 / 仅我发布的 */
type TopicScope = "all" | "mine";

/** 毫秒时间戳格式化（容错非法 / 非正值） */
function formatTopicTime(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) return "—";
  try {
    return new Date(ms).toLocaleString("zh-CN", { hour12: false });
  } catch {
    return "—";
  }
}

export function CourseTopicsPage() {
  const navigate = useNavigate();
  const { courseId } = useParams<{ courseId: string }>();
  const [msgApi, contextHolder] = message.useMessage();
  const [composeOpen, setComposeOpen] = useState(false);
  const [scope, setScope] = useState<TopicScope>("all");

  // 细粒度 selector，避免任一字段变化导致整页 rerender（与 CourseTaskSelectPage 一致）
  const courses = useCourseStore((s) => s.courses);
  const coursesLoading = useCourseStore((s) => s.isLoading);
  const fetchCourses = useCourseStore((s) => s.fetchCourses);

  const topics = useTopicStore((s) => s.topics);
  const topicsLoading = useTopicStore((s) => s.isLoading);
  const topicSubmitting = useTopicStore((s) => s.submitting);
  const topicError = useTopicStore((s) => s.error);
  const fetchTopics = useTopicStore((s) => s.fetchTopics);
  const prependTopics = useTopicStore((s) => s.prependTopics);
  const resetTopics = useTopicStore((s) => s.reset);

  const username = useAuthStore((s) => s.username);

  const course = useMemo(
    () => courses.find((item) => item.id === courseId),
    [courses, courseId],
  );

  // deep-link 兜底：直达 /courses/:id/topics 时课程列表为空则补拉
  useEffect(() => {
    if (!courseId || courses.length > 0) return;
    void fetchCourses();
  }, [courseId, courses.length, fetchCourses]);

  // course 的稳定原始字段。course 由 undefined→有值（deep-link 兜底拉到课程后）时，
  // 这两个值从 undefined 变为字符串，恰好触发下方 effect 重跑完成首拉；
  // 而 courses 数组因无关原因刷新身份时，同一课程的这两个字符串不变，不会重复拉取。
  const activeCourseId = course?.courseId;
  const activeClazzId = course?.clazzId;

  // 进入页面 / 课程就绪 / 切换范围后拉取话题
  useEffect(() => {
    if (activeCourseId) {
      void fetchTopics(activeCourseId, activeClazzId ?? "", scope === "mine");
    }
  }, [activeCourseId, activeClazzId, scope, fetchTopics]);

  // 离开页面清空，避免切换课程时串显上一门课的话题
  useEffect(() => {
    return () => resetTopics();
  }, [resetTopics]);

  const totalReplies = useMemo(
    () => topics.reduce((sum, item) => sum + (item.replyCount || 0), 0),
    [topics],
  );
  const totalReads = useMemo(
    () => topics.reduce((sum, item) => sum + (item.readCount || 0), 0),
    [topics],
  );

  const handleRefresh = () => {
    if (course) void fetchTopics(course.courseId, course.clazzId, scope === "mine");
  };

  const handleViewOriginal = async (shareUrl: string) => {
    if (!shareUrl) {
      msgApi.info("该话题暂无分享链接");
      return;
    }
    try {
      await openExternal(shareUrl);
    } catch {
      try {
        await navigator.clipboard.writeText(shareUrl);
        msgApi.info("无法打开浏览器，链接已复制到剪贴板");
      } catch {
        msgApi.error("无法打开链接");
      }
    }
  };

  const handleComposeSuccess = (
    results: AddTopicResult[],
    draft: { title: string; content: string },
    error: string | null,
  ) => {
    setComposeOpen(false);
    if (!course || results.length === 0) return;

    // results 按发布顺序（先发的在前）；列表最新在前 → 逆序使最后发布的置顶。
    // 缺失的计数字段容错为 0，发布者用当前账号占位；id 用真实 topicId 保证 key 稳定。
    const baseTime = Date.now();
    const optimistic: TopicListItem[] = results
      .map((result, idx) => ({
        id: result.topicId,
        uuid: result.uuid,
        title: draft.title || "(无标题)",
        content: draft.content,
        creatorName: (scope === "mine" ? topics[0]?.creatorName : "") || username || "我",
        createTime: baseTime + idx,
        replyCount: 0,
        readCount: 0,
        praiseCount: 0,
        shareUrl: result.shareUrl,
      }))
      .reverse();
    prependTopics(optimistic);

    const circleName = results[0]?.circleName || course.title;
    if (error) {
      msgApi.warning(`成功发布 ${results.length} 条，其余失败：${error}`);
    } else if (results.length > 1) {
      msgApi.success(`成功发布 ${results.length} 条到《${circleName}》讨论区`);
    } else {
      msgApi.success(`已发布到《${circleName}》讨论区`);
    }
  };

  // 课程不存在兜底（与 CourseTaskSelectPage 一致）
  if (!courseId || (!course && !coursesLoading)) {
    return (
      <div style={{ padding: "60px 22px", background: "var(--apple-color-canvas)" }}>
        {contextHolder}
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          <Card>
            <Empty description="课程不存在或已失效" />
            <div style={{ display: "flex", justifyContent: "center", marginTop: 24 }}>
              <PillButton onClick={() => navigate("/courses")}>返回课程列表</PillButton>
            </div>
          </Card>
        </div>
      </div>
    );
  }

  // 课程对象尚未加载完成的占位
  if (!course) {
    return (
      <div style={{ background: "var(--apple-color-canvas)", minHeight: "100%" }}>
        {contextHolder}
        <section style={{ padding: "48px 22px 24px" }}>
          <div style={{ maxWidth: 1024, margin: "0 auto" }}>
            <div style={{ marginBottom: 20 }}>
              <UtilityButton light onClick={() => navigate("/courses")}>
                ← 返回课程列表
              </UtilityButton>
            </div>
            <Eyebrow>课程讨论</Eyebrow>
            <Headline level="lg" style={{ marginTop: 8 }}>
              正在加载课程
            </Headline>
            <div style={{ marginTop: 24 }}>
              <Progress percent={100} status="active" showInfo={false} />
            </div>
          </div>
        </section>
      </div>
    );
  }

  const emptyDescription =
    scope === "mine"
      ? "你在该课程还没有发布过话题"
      : "该课程暂无讨论话题，点上方「发布话题」发布第一条";

  return (
    <div style={{ background: "var(--apple-color-canvas)", minHeight: "100%" }}>
      {contextHolder}
      {/* Hero */}
      <section style={{ padding: "48px 22px 24px" }}>
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          <div style={{ marginBottom: 20, display: "flex", flexWrap: "wrap", gap: 12 }}>
            <UtilityButton light onClick={() => navigate(`/courses/${courseId}/tasks`)}>
              ← 返回任务配置
            </UtilityButton>
            <UtilityButton light onClick={() => navigate("/courses")}>
              课程列表
            </UtilityButton>
          </div>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 8, marginBottom: 14 }}>
            <Tag>{course.courseTypeLabel}</Tag>
            <Tag tone="neutral">教师 · {course.teacher || "未知"}</Tag>
          </div>
          <Eyebrow>课程讨论</Eyebrow>
          <Headline level="lg" style={{ marginTop: 8 }}>
            {course.title}
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
            在课程讨论区发布话题、查看已有讨论。发布的内容会真实出现在讨论区，所有同学与教师可见。
          </p>

          {/* 主操作 */}
          <div style={{ marginTop: 28, display: "flex", gap: 12, flexWrap: "wrap", alignItems: "center" }}>
            <PillButton onClick={() => setComposeOpen(true)}>发布话题</PillButton>
            <UtilityButton
              light
              disabled={topicsLoading || topicSubmitting}
              icon={<ReloadOutlined />}
              onClick={handleRefresh}
            >
              刷新
            </UtilityButton>
          </div>

          {/* Metrics */}
          <div
            style={{
              marginTop: 32,
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))",
              gap: 16,
            }}
          >
            <Metric label="话题数" value={topics.length} hint={scope === "mine" ? "我发布的话题" : "当前列表话题"} />
            <Metric label="总回复" value={totalReplies} hint="列表内回复合计" />
            <Metric label="总阅读" value={totalReads} hint="列表内阅读合计" />
          </div>
        </div>
      </section>

      {topicError ? (
        <section style={{ padding: "0 22px 24px" }}>
          <div style={{ maxWidth: 1024, margin: "0 auto" }}>
            <Alert type="error" message={topicError} style={{ borderRadius: 11 }} />
          </div>
        </section>
      ) : null}

      {/* 话题列表 */}
      <section
        style={{
          padding: "16px 22px 96px",
          borderTop: "1px solid var(--apple-color-divider-soft)",
          background: "var(--apple-color-canvas)",
        }}
      >
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          {/* 范围切换：全部 / 仅我发布的 */}
          <div style={{ display: "flex", gap: 8, marginBottom: 16, flexWrap: "wrap", alignItems: "center" }}>
            <Chip selected={scope === "all"} disabled={topicsLoading || topicSubmitting} onClick={() => setScope("all")}>
              全部
            </Chip>
            <Chip selected={scope === "mine"} disabled={topicsLoading || topicSubmitting} onClick={() => setScope("mine")}>
              仅我发布的
            </Chip>
          </div>

          {topicsLoading && topics.length === 0 ? (
            <Card>
              <div style={{ padding: "20px 4px" }}>
                <div
                  style={{
                    fontFamily: "var(--apple-font-text)",
                    fontSize: 14,
                    fontWeight: 600,
                    letterSpacing: "-0.224px",
                    color: "var(--apple-color-ink)",
                    marginBottom: 10,
                  }}
                >
                  正在加载话题列表…
                </div>
                <Progress percent={100} status="active" showInfo={false} />
              </div>
            </Card>
          ) : topics.length === 0 ? (
            <Card>
              <Empty description={emptyDescription} />
            </Card>
          ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
              {topics.map((topic) => (
                <div
                  key={topic.uuid || topic.id}
                  style={{
                    border: "1px solid var(--apple-color-hairline)",
                    borderRadius: 18,
                    padding: "16px 20px",
                    background: "var(--apple-color-canvas)",
                    display: "flex",
                    alignItems: "flex-start",
                    justifyContent: "space-between",
                    gap: 16,
                    flexWrap: "wrap",
                  }}
                >
                  <div style={{ flex: 1, minWidth: 240 }}>
                    <div
                      style={{
                        fontFamily: "var(--apple-font-text)",
                        fontSize: 17,
                        fontWeight: 600,
                        letterSpacing: "-0.374px",
                        color: "var(--apple-color-ink)",
                        lineHeight: 1.4,
                        wordBreak: "break-word",
                      }}
                    >
                      {topic.title || "(无标题)"}
                    </div>
                    {topic.content ? (
                      <div
                        style={{
                          fontFamily: "var(--apple-font-text)",
                          fontSize: 14,
                          color: "var(--apple-color-ink-muted-80)",
                          lineHeight: 1.5,
                          marginTop: 6,
                          display: "-webkit-box",
                          WebkitBoxOrient: "vertical",
                          WebkitLineClamp: 2,
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          wordBreak: "break-word",
                        }}
                      >
                        {topic.content}
                      </div>
                    ) : null}
                    <div style={{ display: "flex", flexWrap: "wrap", gap: 8, marginTop: 10 }}>
                      <Tag tone="neutral">{topic.creatorName || "匿名"}</Tag>
                      <Tag tone="neutral">{formatTopicTime(topic.createTime)}</Tag>
                      <Tag tone="neutral">回复 {topic.replyCount}</Tag>
                      <Tag tone="neutral">阅读 {topic.readCount}</Tag>
                      <Tag tone="neutral">赞 {topic.praiseCount}</Tag>
                    </div>
                  </div>
                  <UtilityButton light onClick={() => void handleViewOriginal(topic.shareUrl)}>
                    查看原帖 →
                  </UtilityButton>
                </div>
              ))}
            </div>
          )}
        </div>
      </section>

      <ComposeTopicModal
        open={composeOpen}
        course={course}
        onClose={() => setComposeOpen(false)}
        onSuccess={handleComposeSuccess}
      />
    </div>
  );
}
