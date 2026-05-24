import { useEffect, useMemo, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { Alert, Checkbox, Empty, Progress } from "antd";
import { useCourseStore } from "../stores/courseStore";
import type { FlatCourseJob, JobType } from "../types/course";
import { Card, Chip, Eyebrow, Headline, Metric, PillButton, Tag, UtilityButton } from "../components/ui/appleUI";

const quickSelectOptions: Array<{ label: string; types: JobType[] }> = [
  { label: "仅视频", types: ["video"] },
  { label: "视频 + 阅读", types: ["video", "read"] },
  { label: "全部任务", types: [] },
];

const jobTypeOptions: Array<{ label: string; value: JobType }> = [
  { label: "视频", value: "video" },
  { label: "文档", value: "document" },
  { label: "阅读", value: "read" },
  { label: "直播", value: "live" },
  { label: "作业", value: "workid" },
];

function getJobTypeTone(jobType: JobType) {
  switch (jobType) {
    case "video":
      return "default" as const;
    case "document":
      return "warning" as const;
    case "read":
      return "neutral" as const;
    case "live":
      return "danger" as const;
    case "workid":
      return "success" as const;
    default:
      return "default" as const;
  }
}

export function CourseTaskSelectPage() {
  const navigate = useNavigate();
  const { courseId } = useParams<{ courseId: string }>();
  const [selectedJobTypes, setSelectedJobTypes] = useState<JobType[]>([]);
  // 使用细粒度 selector，避免对全 store 解构导致任一字段变化都 rerender
  const courses = useCourseStore((s) => s.courses);
  const courseTrees = useCourseStore((s) => s.courseTrees);
  const treeLoadingIds = useCourseStore((s) => s.treeLoadingIds);
  const learningSelections = useCourseStore((s) => s.learningSelections);
  const coursesLoading = useCourseStore((s) => s.isLoading);
  const error = useCourseStore((s) => s.error);
  const activateCourse = useCourseStore((s) => s.activateCourse);
  const fetchCourses = useCourseStore((s) => s.fetchCourses);
  const selectJobsByType = useCourseStore((s) => s.selectJobsByType);
  const selectAllJobsForCourse = useCourseStore((s) => s.selectAllJobsForCourse);
  const batchUpdateJobsForCourse = useCourseStore((s) => s.batchUpdateJobsForCourse);
  const toggleJobSelection = useCourseStore((s) => s.toggleJobSelection);
  const getFlatJobsForCourse = useCourseStore((s) => s.getFlatJobsForCourse);

  const course = useMemo(
    () => courses.find((item) => item.id === courseId),
    [courses, courseId],
  );

  useEffect(() => {
    if (!courseId || courses.length > 0) return;
    void fetchCourses();
  }, [courseId, courses.length, fetchCourses]);

  // 依赖 courseId（稳定字符串）而非 course 派生对象，
  // 否则 courses 数组刷新时 course 引用变化会触发重复 activateCourse 抓树
  useEffect(() => {
    if (course) void activateCourse(course);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [courseId, activateCourse]);

  const flatJobs = useMemo(
    () => (courseId ? getFlatJobsForCourse(courseId) : []),
    [courseId, getFlatJobsForCourse, courseTrees],
  );

  const filteredJobs = useMemo(() => {
    if (selectedJobTypes.length === 0) return flatJobs;
    return flatJobs.filter((job) => selectedJobTypes.includes(job.jobType));
  }, [flatJobs, selectedJobTypes]);

  const selectedJobKeys = useMemo(() => {
    if (!course) return [] as string[];
    return (
      learningSelections[course.id]?.selectedPoints.flatMap((point) =>
        point.selectedJobIds.map((jobId) => `${point.pointId}:${jobId}`),
      ) ?? []
    );
  }, [course, learningSelections]);

  const selectedJobKeySet = useMemo(() => new Set(selectedJobKeys), [selectedJobKeys]);

  const filteredSelectedCount = useMemo(
    () => filteredJobs.filter((job) => selectedJobKeySet.has(`${job.pointId}:${job.id}`)).length,
    [filteredJobs, selectedJobKeySet],
  );

  const selectedJobCount = selectedJobKeys.length;
  const completedJobCount = useMemo(() => flatJobs.filter((job) => job.isCompleted).length, [flatJobs]);
  const pendingJobCount = flatJobs.length - completedJobCount;
  const chapterCount = useMemo(() => new Set(flatJobs.map((job) => job.pointId)).size, [flatJobs]);
  const completionRate = flatJobs.length > 0 ? Math.round((completedJobCount / flatJobs.length) * 100) : 0;

  const isLoading = coursesLoading || (course ? treeLoadingIds.includes(course.id) : false);
  const batchActionDisabled = isLoading || filteredJobs.length === 0;

  const toggleType = (type: JobType) => {
    setSelectedJobTypes((prev) =>
      prev.includes(type) ? prev.filter((value) => value !== type) : [...prev, type],
    );
  };

  if (!courseId || (!course && !isLoading)) {
    return (
      <div style={{ padding: "60px 22px", background: "var(--apple-color-canvas)" }}>
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

  // 课程对象尚未加载完成：先用占位渲染 Hero + 进度条，避免整页只看到 Spin
  if (!course) {
    return (
      <div style={{ background: "var(--apple-color-canvas)", minHeight: "100%" }}>
        <section style={{ padding: "48px 22px 24px" }}>
          <div style={{ maxWidth: 1024, margin: "0 auto" }}>
            <div style={{ marginBottom: 20 }}>
              <UtilityButton light onClick={() => navigate("/courses")}>
                ← 返回课程列表
              </UtilityButton>
            </div>
            <Eyebrow>任务配置</Eyebrow>
            <Headline level="lg" style={{ marginTop: 8 }}>
              正在加载课程
            </Headline>
            <div style={{ marginTop: 24 }}>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "baseline",
                  marginBottom: 8,
                }}
              >
                <span
                  style={{
                    fontFamily: "var(--apple-font-text)",
                    fontSize: 14,
                    fontWeight: 600,
                    letterSpacing: "-0.224px",
                    color: "var(--apple-color-ink)",
                  }}
                >
                  正在拉取课程信息
                </span>
                <span
                  style={{
                    fontFamily: "var(--apple-font-text)",
                    fontSize: 13,
                    color: "var(--apple-color-ink-muted-48)",
                    letterSpacing: "-0.12px",
                  }}
                >
                  请稍候…
                </span>
              </div>
              <Progress percent={100} status="active" showInfo={false} />
            </div>
          </div>
        </section>
      </div>
    );
  }

  return (
    <div style={{ background: "var(--apple-color-canvas)", minHeight: "100%" }}>
      {/* Hero */}
      <section style={{ padding: "48px 22px 24px" }}>
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          <div style={{ marginBottom: 20 }}>
            <UtilityButton light onClick={() => navigate("/courses")}>
              ← 返回课程列表
            </UtilityButton>
          </div>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 8, marginBottom: 14 }}>
            <Tag>{course.courseTypeLabel}</Tag>
            <Tag tone="neutral">教师 · {course.teacher || "未知"}</Tag>
            {selectedJobCount > 0 ? <Tag tone="success">已选 {selectedJobCount} 个任务</Tag> : null}
          </div>
          <Eyebrow>任务配置</Eyebrow>
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
            按类型筛选任务，勾选需要学习的章节与任务点。选择会自动保存。
          </p>

          {/* Metrics */}
          <div
            style={{
              marginTop: 32,
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))",
              gap: 16,
            }}
          >
            <Metric label="章节数" value={chapterCount} hint="可配置章节" />
            <Metric label="任务总数" value={flatJobs.length} hint="当前课程全部任务点" />
            <Metric label="未完成" value={pendingJobCount} hint="可继续学习" />
            <Metric label="已选任务" value={selectedJobCount} hint="将带入任务执行页" />
          </div>

          {/* 课程完成进度（加载完成且有任务时显示） */}
          {!isLoading && flatJobs.length > 0 ? (
            <div style={{ marginTop: 24 }}>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "baseline",
                  marginBottom: 8,
                }}
              >
                <span
                  style={{
                    fontFamily: "var(--apple-font-text)",
                    fontSize: 14,
                    fontWeight: 600,
                    letterSpacing: "-0.224px",
                    color: "var(--apple-color-ink)",
                  }}
                >
                  完成进度
                </span>
                <span
                  style={{
                    fontFamily: "var(--apple-font-text)",
                    fontSize: 13,
                    color: "var(--apple-color-ink-muted-48)",
                    letterSpacing: "-0.12px",
                  }}
                >
                  {completedJobCount}/{flatJobs.length} 任务 ({completionRate}%)
                </span>
              </div>
              <Progress
                percent={completionRate}
                showInfo={false}
                strokeColor={
                  completionRate === 100
                    ? "var(--apple-color-success, #34c759)"
                    : "var(--apple-color-primary, #007aff)"
                }
              />
            </div>
          ) : null}

          {/* 章节加载进度（仅在加载中显示） */}
          {isLoading ? (
            <div style={{ marginTop: 24 }}>
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "baseline",
                  marginBottom: 8,
                }}
              >
                <span
                  style={{
                    fontFamily: "var(--apple-font-text)",
                    fontSize: 14,
                    fontWeight: 600,
                    letterSpacing: "-0.224px",
                    color: "var(--apple-color-ink)",
                  }}
                >
                  正在加载章节
                </span>
                <span
                  style={{
                    fontFamily: "var(--apple-font-text)",
                    fontSize: 13,
                    color: "var(--apple-color-ink-muted-48)",
                    letterSpacing: "-0.12px",
                  }}
                >
                  {flatJobs.length > 0 ? `已加载 ${flatJobs.length} 个任务…` : "请稍候…"}
                </span>
              </div>
              <Progress percent={100} status="active" showInfo={false} />
            </div>
          ) : null}
        </div>
      </section>

      {error ? (
        <section style={{ padding: "0 22px 24px" }}>
          <div style={{ maxWidth: 1024, margin: "0 auto" }}>
            <Alert type="error" message={error} style={{ borderRadius: 11 }} />
          </div>
        </section>
      ) : null}

      {/* Filter chips */}
      <section
        style={{
          padding: "16px 22px 24px",
          borderTop: "1px solid var(--apple-color-divider-soft)",
          background: "var(--apple-color-canvas)",
        }}
      >
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          <div style={{ marginBottom: 18 }}>
            <span
              style={{
                fontFamily: "var(--apple-font-text)",
                fontSize: 14,
                fontWeight: 600,
                letterSpacing: "-0.224px",
                color: "var(--apple-color-ink-muted-48)",
                marginRight: 12,
              }}
            >
              筛选类型
            </span>
            <span
              style={{
                fontFamily: "var(--apple-font-text)",
                fontSize: 12,
                color: "var(--apple-color-ink-muted-48)",
                letterSpacing: "-0.12px",
              }}
            >
              当前筛选 {filteredJobs.length} 个任务，已选 {filteredSelectedCount} 个
            </span>
          </div>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 10 }}>
            {jobTypeOptions.map((option) => (
              <Chip
                key={option.value}
                selected={selectedJobTypes.includes(option.value)}
                disabled={isLoading}
                onClick={() => toggleType(option.value)}
              >
                {option.label}
              </Chip>
            ))}
          </div>

          <div
            style={{
              marginTop: 28,
              display: "flex",
              flexWrap: "wrap",
              gap: 12,
              alignItems: "center",
            }}
          >
            <span
              style={{
                fontFamily: "var(--apple-font-text)",
                fontSize: 14,
                fontWeight: 600,
                letterSpacing: "-0.224px",
                color: "var(--apple-color-ink-muted-48)",
                marginRight: 4,
              }}
            >
              快捷选择
            </span>
            {quickSelectOptions.map((option) => (
              <UtilityButton
                key={option.label}
                light
                disabled={isLoading}
                onClick={() => selectJobsByType(course, option.types)}
              >
                {option.label}
              </UtilityButton>
            ))}
            <UtilityButton light disabled={isLoading} onClick={() => selectAllJobsForCourse(course)}>
              选择全部任务
            </UtilityButton>
          </div>

          <div
            style={{
              marginTop: 18,
              display: "flex",
              flexWrap: "wrap",
              gap: 12,
              alignItems: "center",
            }}
          >
            <span
              style={{
                fontFamily: "var(--apple-font-text)",
                fontSize: 14,
                fontWeight: 600,
                letterSpacing: "-0.224px",
                color: "var(--apple-color-ink-muted-48)",
                marginRight: 4,
              }}
            >
              批量操作
            </span>
            <UtilityButton
              light
              disabled={batchActionDisabled}
              onClick={() =>
                batchUpdateJobsForCourse(
                  course,
                  filteredJobs.filter((job) => !job.isCompleted),
                  "select",
                )
              }
            >
              全选当前
            </UtilityButton>
            <UtilityButton
              light
              disabled={batchActionDisabled}
              onClick={() =>
                batchUpdateJobsForCourse(
                  course,
                  filteredJobs.filter((job) => !job.isCompleted),
                  "invert",
                )
              }
            >
              反选当前
            </UtilityButton>
            <UtilityButton
              light
              disabled={batchActionDisabled}
              onClick={() =>
                batchUpdateJobsForCourse(
                  course,
                  filteredJobs.filter((job) => !job.isCompleted),
                  "clear",
                )
              }
            >
              清空当前
            </UtilityButton>
          </div>
        </div>
      </section>

      {/* Job list */}
      <section style={{ padding: "8px 22px 96px", background: "var(--apple-color-canvas)" }}>
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          {isLoading ? (
            <Card>
              <div style={{ padding: "20px 4px" }}>
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "baseline",
                    marginBottom: 10,
                  }}
                >
                  <span
                    style={{
                      fontFamily: "var(--apple-font-text)",
                      fontSize: 14,
                      fontWeight: 600,
                      letterSpacing: "-0.224px",
                      color: "var(--apple-color-ink)",
                    }}
                  >
                    正在加载任务列表
                  </span>
                  <span
                    style={{
                      fontFamily: "var(--apple-font-text)",
                      fontSize: 13,
                      color: "var(--apple-color-ink-muted-48)",
                      letterSpacing: "-0.12px",
                    }}
                  >
                    {flatJobs.length > 0
                      ? `已加载 ${flatJobs.length} 个任务，${chapterCount} 个章节…`
                      : "正在解析课程章节…"}
                  </span>
                </div>
                <Progress percent={100} status="active" showInfo={false} />
              </div>
            </Card>
          ) : filteredJobs.length === 0 ? (
            <Card>
              <Empty
                description={selectedJobTypes.length === 0 ? "暂无可选任务" : "当前筛选下暂无任务"}
              />
            </Card>
          ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
              {filteredJobs.map((job: FlatCourseJob) => {
                const checked = selectedJobKeySet.has(`${job.pointId}:${job.id}`);
                return (
                  <div
                    key={`${job.pointId}-${job.id}`}
                    style={{
                      border: "1px solid var(--apple-color-hairline)",
                      borderRadius: 18,
                      padding: "16px 20px",
                      background: "var(--apple-color-canvas)",
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                      gap: 16,
                      flexWrap: "wrap",
                      transition: "border-color 160ms ease",
                      borderColor: checked
                        ? "var(--apple-color-primary-focus)"
                        : "var(--apple-color-hairline)",
                    }}
                  >
                    <div style={{ display: "flex", alignItems: "flex-start", gap: 14, flex: 1, minWidth: 240 }}>
                      <Checkbox
                        checked={checked}
                        disabled={job.isCompleted}
                        onChange={(event) =>
                          toggleJobSelection(course, job.pointId, job.id, event.target.checked)
                        }
                        style={{ marginTop: 4 }}
                      />
                      <div style={{ flex: 1, minWidth: 0 }}>
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
                          {job.name}
                        </div>
                        <div style={{ display: "flex", flexWrap: "wrap", gap: 8, marginTop: 8 }}>
                          {job.isCompleted ? (
                            <Tag tone="success">已完成</Tag>
                          ) : (
                            <Tag>未完成</Tag>
                          )}
                          {job.needUnlock ? <Tag tone="warning">待开放</Tag> : null}
                          {job.hasFinished ? <Tag tone="neutral">章节已完成</Tag> : null}
                          <Tag tone="neutral">{job.pointTitle}</Tag>
                        </div>
                      </div>
                    </div>
                    <Tag tone={getJobTypeTone(job.jobType)}>{job.typeLabel}</Tag>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </section>

      {/* Sticky bottom action */}
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
                color: "var(--apple-color-ink)",
              }}
            >
              已选择 {selectedJobCount} 个任务点
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
              选择已自动保存
            </div>
          </div>
          <PillButton onClick={() => navigate("/courses")}>返回课程列表</PillButton>
        </div>
      </div>
    </div>
  );
}
