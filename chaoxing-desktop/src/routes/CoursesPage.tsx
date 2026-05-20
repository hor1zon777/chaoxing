import { useEffect, useMemo, useState } from "react";
import { Input, Pagination, Spin, Empty, Alert, Progress, message } from "antd";
import { ReloadOutlined } from "@ant-design/icons";
import { useNavigate } from "react-router-dom";
import { useCourseStore } from "../stores/courseStore";
import type { CourseType } from "../types/course";
import { Card, Chip, PillButton, Tag } from "../components/ui/appleUI";

const courseTypeOptions: Array<{ value: CourseType; label: string }> = [
  { value: "all", label: "全部" },
  { value: "course", label: "课程" },
  { value: "archive", label: "归档" },
  { value: "other", label: "其他" },
];

const pageSizeOptions = [12, 24, 36];
const defaultPageSize = 12;

function getCourseTypeTone(courseType: CourseType): "default" | "neutral" | "success" | "warning" {
  switch (courseType) {
    case "course":
      return "default";
    case "archive":
      return "warning";
    case "other":
      return "neutral";
    default:
      return "default";
  }
}

export function CoursesPage() {
  const navigate = useNavigate();
  const {
    courses,
    selectedCourseIds,
    courseTypeFilter,
    courseTrees,
    isLoading,
    error,
    fetchCourses,
    activateCourse,
    setCourseTypeFilter,
  } = useCourseStore();

  useEffect(() => {
    if (courses.length === 0) void fetchCourses();
  }, [courses.length, fetchCourses]);

  const [searchText, setSearchText] = useState("");
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(defaultPageSize);
  const [activatingId, setActivatingId] = useState<string | null>(null);

  const filteredCourses = useMemo(() => {
    return courses.filter((course) => {
      const matchesSearch =
        !searchText ||
        course.title.includes(searchText) ||
        course.teacher.includes(searchText);
      const matchesType = courseTypeFilter === "all" || course.courseType === courseTypeFilter;
      return matchesSearch && matchesType;
    });
  }, [courses, searchText, courseTypeFilter]);

  useEffect(() => {
    setCurrentPage(1);
  }, [searchText, courseTypeFilter]);

  useEffect(() => {
    const totalPages = Math.max(1, Math.ceil(filteredCourses.length / pageSize));
    if (currentPage > totalPages) setCurrentPage(totalPages);
  }, [filteredCourses.length, currentPage, pageSize]);

  const paginatedCourses = useMemo(() => {
    const startIndex = (currentPage - 1) * pageSize;
    return filteredCourses.slice(startIndex, startIndex + pageSize);
  }, [filteredCourses, currentPage, pageSize]);

  const handleOpenTaskSelector = async (courseId: string) => {
    const course = courses.find((item) => item.id === courseId);
    if (!course) return;

    setActivatingId(courseId);
    try {
      const hadTreeBefore = Boolean(useCourseStore.getState().courseTrees[course.id]);
      await activateCourse(course);
      const hasTreeAfter = Boolean(useCourseStore.getState().courseTrees[course.id]);

      if (!hasTreeAfter && !hadTreeBefore) {
        message.error("课程任务加载失败，请稍后重试");
        return;
      }
      navigate(`/courses/${courseId}/tasks`);
    } finally {
      setActivatingId(null);
    }
  };

  const handleRefresh = async () => {
    setCurrentPage(1);
    await fetchCourses();
  };

  return (
    <div style={{ background: "var(--apple-color-canvas)", flex: 1, display: "flex", flexDirection: "column" }}>
      {/* 紧凑顶栏：标题行 + 筛选行合为一体 */}
      <div
        style={{
          padding: "16px 22px 12px",
          borderBottom: "1px solid var(--apple-color-divider-soft)",
        }}
      >
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          {/* 第一行：标题 + 操作 */}
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              gap: 16,
              flexWrap: "wrap",
              marginBottom: 12,
            }}
          >
            <div style={{ display: "flex", alignItems: "baseline", gap: 12 }}>
              <h2
                style={{
                  margin: 0,
                  fontFamily: "var(--apple-font-display)",
                  fontSize: 24,
                  fontWeight: 600,
                  letterSpacing: 0,
                  lineHeight: 1.2,
                }}
              >
                选择课程
              </h2>
              <span
                style={{
                  fontFamily: "var(--apple-font-text)",
                  fontSize: 14,
                  color: "var(--apple-color-ink-muted-48)",
                  letterSpacing: "-0.224px",
                }}
              >
                {selectedCourseIds.length > 0
                  ? `已选 ${selectedCourseIds.length} 门`
                  : `${filteredCourses.length} 门课程`}
              </span>
            </div>

            <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
              <button
                type="button"
                onClick={() => void handleRefresh()}
                disabled={isLoading}
                style={{
                  background: "none",
                  border: "none",
                  padding: 6,
                  cursor: "pointer",
                  color: "var(--apple-color-ink-muted-48)",
                  display: "flex",
                  alignItems: "center",
                  borderRadius: 9999,
                  width: 32,
                  height: 32,
                  justifyContent: "center",
                }}
                aria-label="刷新课程"
              >
                <ReloadOutlined />
              </button>
              <PillButton
                onClick={() => navigate("/tasks")}
                disabled={selectedCourseIds.length === 0}
              >
                开始学习
              </PillButton>
            </div>
          </div>

          {/* 第二行：类型筛选 + 搜索 */}
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              gap: 12,
              flexWrap: "wrap",
            }}
          >
            <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
              {courseTypeOptions.map((option) => (
                <Chip
                  key={option.value}
                  selected={courseTypeFilter === option.value}
                  onClick={() => setCourseTypeFilter(option.value)}
                  disabled={isLoading}
                >
                  {option.label}
                </Chip>
              ))}
            </div>

            <Input
              placeholder="搜索课程或教师"
              allowClear
              value={searchText}
              onChange={(event) => setSearchText(event.target.value)}
              style={{
                width: 220,
                borderRadius: 9999,
                height: 36,
                fontSize: 14,
              }}
            />
          </div>
        </div>
      </div>

      {error ? (
        <div style={{ padding: "0 22px" }}>
          <div style={{ maxWidth: 1024, margin: "0 auto" }}>
            <Alert message={error} type="error" closable style={{ borderRadius: 11, marginTop: 12 }} />
          </div>
        </div>
      ) : null}

      {/* 课程网格 — 填满剩余空间 */}
      <div style={{ flex: 1, overflow: "auto", padding: "14px 22px 16px" }}>
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          {isLoading ? (
            <div style={{ textAlign: "center", padding: "60px 0" }}>
              <Spin size="default" />
            </div>
          ) : filteredCourses.length === 0 ? (
            <Card>
              <Empty description={searchText || courseTypeFilter !== "all" ? "无匹配课程" : "暂无课程"} />
            </Card>
          ) : (
            <>
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))",
                  gap: 14,
                }}
              >
                {paginatedCourses.map((course) => {
                  const isSelected = selectedCourseIds.includes(course.id);
                  const isActivating = activatingId === course.id;
                  const tree = courseTrees[course.id];
                  const jobs = tree ? tree.points.flatMap((p) => p.jobs) : [];
                  const totalJobs = jobs.length;
                  const completedJobs = jobs.filter((j) => j.isCompleted).length;
                  const completionRate = totalJobs > 0 ? Math.round((completedJobs / totalJobs) * 100) : 0;
                  const hasProgressData = tree !== undefined && totalJobs > 0;
                  return (
                    <Card
                      key={course.id}
                      onClick={() => void handleOpenTaskSelector(course.id)}
                      selected={isSelected}
                      hoverable
                      padding={16}
                      style={{
                        opacity: isActivating ? 0.6 : 1,
                        pointerEvents: isActivating ? "none" : undefined,
                      }}
                    >
                      <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                        <div
                          style={{
                            display: "flex",
                            justifyContent: "space-between",
                            alignItems: "flex-start",
                            gap: 8,
                          }}
                        >
                          <h3
                            style={{
                              margin: 0,
                              fontFamily: "var(--apple-font-display)",
                              fontSize: 17,
                              fontWeight: 600,
                              letterSpacing: "-0.374px",
                              lineHeight: 1.3,
                              overflow: "hidden",
                              textOverflow: "ellipsis",
                              display: "-webkit-box",
                              WebkitBoxOrient: "vertical",
                              WebkitLineClamp: 2,
                            }}
                          >
                            {course.title}
                          </h3>
                          <Tag tone={getCourseTypeTone(course.courseType)}>
                            {course.courseTypeLabel}
                          </Tag>
                        </div>

                        <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
                          {isSelected ? <Tag tone="success">已配置</Tag> : null}
                          <Tag tone="neutral">{course.teacher || "未知教师"}</Tag>
                        </div>

                        {hasProgressData ? (
                          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                            <div
                              style={{
                                display: "flex",
                                justifyContent: "space-between",
                                alignItems: "baseline",
                                fontFamily: "var(--apple-font-text)",
                                fontSize: 12,
                                color: "var(--apple-color-ink-muted-48)",
                                letterSpacing: "-0.12px",
                              }}
                            >
                              <span>完成进度</span>
                              <span style={{ fontWeight: 600, color: "var(--apple-color-ink)" }}>
                                {completedJobs}/{totalJobs} ({completionRate}%)
                              </span>
                            </div>
                            <Progress
                              percent={completionRate}
                              size="small"
                              showInfo={false}
                              strokeColor={
                                completionRate === 100
                                  ? "var(--apple-color-success, #34c759)"
                                  : "var(--apple-color-primary, #007aff)"
                              }
                            />
                          </div>
                        ) : null}

                        <div
                          style={{
                            display: "flex",
                            justifyContent: "space-between",
                            alignItems: "center",
                          }}
                        >
                          <span
                            style={{
                              fontFamily: "var(--apple-font-text)",
                              fontSize: 12,
                              color: "var(--apple-color-ink-muted-48)",
                              letterSpacing: "-0.12px",
                            }}
                          >
                            {course.courseId}
                          </span>
                          <span
                            style={{
                              fontFamily: "var(--apple-font-text)",
                              fontSize: 13,
                              fontWeight: 600,
                              color: "var(--apple-color-primary)",
                              letterSpacing: "-0.224px",
                            }}
                          >
                            {isActivating ? "加载中…" : "进入配置 →"}
                          </span>
                        </div>
                      </div>
                    </Card>
                  );
                })}
              </div>

              {filteredCourses.length > pageSize ? (
                <div
                  style={{
                    marginTop: 14,
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    gap: 12,
                    flexWrap: "wrap",
                  }}
                >
                  <span
                    style={{
                      fontFamily: "var(--apple-font-text)",
                      fontSize: 12,
                      color: "var(--apple-color-ink-muted-48)",
                      letterSpacing: "-0.12px",
                    }}
                  >
                    共 {filteredCourses.length} 门 · 第 {currentPage}/{Math.max(1, Math.ceil(filteredCourses.length / pageSize))} 页
                  </span>
                  <Pagination
                    size="small"
                    current={currentPage}
                    pageSize={pageSize}
                    total={filteredCourses.length}
                    showSizeChanger
                    pageSizeOptions={pageSizeOptions.map(String)}
                    onChange={(page, nextPageSize) => {
                      setCurrentPage(page);
                      if (nextPageSize !== pageSize) setPageSize(nextPageSize);
                    }}
                    showLessItems
                    showQuickJumper={false}
                  />
                </div>
              ) : null}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
