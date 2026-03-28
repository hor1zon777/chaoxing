import { useEffect, useMemo, useState } from "react";
import {
  Input,
  Button,
  Row,
  Col,
  Card,
  Empty,
  Space,
  Spin,
  Alert,
  Typography,
  Tag,
  Segmented,
  Pagination,
  message,
} from "antd";
import { ReloadOutlined, RightOutlined } from "@ant-design/icons";
import { useNavigate } from "react-router-dom";
import { useCourseStore } from "../stores/courseStore";
import type { CourseType } from "../types/course";
import {
  GuideBanner,
  pageTopBannerTexts,
  panelStyle,
  primaryActionButtonStyle,
  surfaceCardStyle,
} from "../components/ui/pagePrimitives";

const { Search } = Input;
const { Text, Title, Paragraph } = Typography;

const courseTypeOptions: Array<{ value: CourseType; label: string }> = [
  { value: "all", label: "全部类型" },
  { value: "course", label: "课程" },
  { value: "archive", label: "归档" },
  { value: "other", label: "其他" },
];

const pageSizeOptions = [9, 18, 27, 36];
const defaultPageSize = pageSizeOptions[0];

function getCourseTypeTagColor(courseType: CourseType): string {
  switch (courseType) {
    case "course":
      return "blue";
    case "archive":
      return "purple";
    case "other":
      return "default";
    default:
      return "blue";
  }
}

export function CoursesPage() {
  const navigate = useNavigate();
  const {
    courses,
    selectedCourseIds,
    courseTypeFilter,
    isLoading,
    error,
    fetchCourses,
    activateCourse,
    setCourseTypeFilter,
  } = useCourseStore();

  useEffect(() => {
    if (courses.length === 0) {
      void fetchCourses();
    }
  }, [courses.length, fetchCourses]);

  const [searchText, setSearchText] = useState("");
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(defaultPageSize);

  const filteredCourses = useMemo(() => {
    return courses.filter((course) => {
      const matchesSearch =
        !searchText ||
        course.title.includes(searchText) ||
        course.teacher.includes(searchText);
      const matchesType =
        courseTypeFilter === "all" || course.courseType === courseTypeFilter;
      return matchesSearch && matchesType;
    });
  }, [courses, searchText, courseTypeFilter]);

  useEffect(() => {
    setCurrentPage(1);
  }, [searchText, courseTypeFilter]);

  useEffect(() => {
    const totalPages = Math.max(1, Math.ceil(filteredCourses.length / pageSize));
    if (currentPage > totalPages) {
      setCurrentPage(totalPages);
    }
  }, [filteredCourses.length, currentPage, pageSize]);

  const paginatedCourses = useMemo(() => {
    const startIndex = (currentPage - 1) * pageSize;
    return filteredCourses.slice(startIndex, startIndex + pageSize);
  }, [filteredCourses, currentPage, pageSize]);

  const handleOpenTaskSelector = async (courseId: string) => {
    const course = courses.find((item) => item.id === courseId);
    if (!course) {
      return;
    }

    const hadTreeBefore = Boolean(useCourseStore.getState().courseTrees[course.id]);
    await activateCourse(course);
    const hasTreeAfter = Boolean(useCourseStore.getState().courseTrees[course.id]);

    if (!hasTreeAfter && !hadTreeBefore) {
      message.error("课程任务加载失败，请稍后重试");
      return;
    }

    navigate(`/courses/${courseId}/tasks`);
  };

  const handleRefresh = async () => {
    setCurrentPage(1);
    await fetchCourses();
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 16, minHeight: "100%" }}>
      <GuideBanner text={pageTopBannerTexts.courses} />

      <Card
        style={surfaceCardStyle}
        styles={{ body: { padding: 24 } }}
      >
        <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
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
              <Title level={4} style={{ margin: 0 }}>
                选择课程并进入任务配置
              </Title>
              <Paragraph type="secondary" style={{ margin: "8px 0 0", lineHeight: 1.7 }}>
                当前已配置 {selectedCourseIds.length} 门课程。可直接进入任务配置页继续细化学习范围。
              </Paragraph>
            </div>
            <div
              style={{
                ...panelStyle,
                padding: "10px 14px",
                minWidth: 200,
              }}
            >
              <Text strong style={{ display: "block" }}>
                当前筛选
              </Text>
              <Text type="secondary" style={{ fontSize: 12 }}>
                {courseTypeOptions.find((item) => item.value === courseTypeFilter)?.label ?? "全部类型"}
                {searchText ? ` · 关键词“${searchText}”` : " · 未输入关键词"}
              </Text>
            </div>
          </div>

          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "flex-end",
              gap: 16,
              flexWrap: "wrap",
            }}
          >
            <Space wrap size={12} style={{ alignItems: "flex-end" }}>
              <div>
                <Text type="secondary" style={{ display: "block", marginBottom: 8, fontSize: 12 }}>
                  搜索课程
                </Text>
                <Search
                  placeholder="搜索课程名称或教师"
                  allowClear
                  value={searchText}
                  onSearch={(value) => {
                    setSearchText(value);
                    setCurrentPage(1);
                  }}
                  onChange={(e) => {
                    setSearchText(e.target.value);
                  }}
                  style={{ width: 280 }}
                />
              </div>

              <div>
                <Text type="secondary" style={{ display: "block", marginBottom: 8, fontSize: 12 }}>
                  类型筛选
                </Text>
                <Segmented
                  value={courseTypeFilter}
                  onChange={(value) => {
                    setCourseTypeFilter(value as CourseType);
                    setCurrentPage(1);
                  }}
                  options={courseTypeOptions}
                />
              </div>

              <Button icon={<ReloadOutlined />} onClick={() => void handleRefresh()}>
                刷新课程
              </Button>
            </Space>

            <Text type="secondary" style={{ fontSize: 13 }}>
              点击课程卡片可直接进入任务配置
            </Text>
          </div>
        </div>
      </Card>

      {error && (
        <Alert
          message={error}
          type="error"
          closable
          style={{ borderRadius: 12 }}
        />
      )}

      <div style={{ flex: 1, overflow: "auto", display: "flex", flexDirection: "column", gap: 16 }}>
        {isLoading ? (
          <div style={{ textAlign: "center", padding: 48 }}>
            <Spin size="large" tip="加载课程中..." />
          </div>
        ) : filteredCourses.length === 0 ? (
          <Card style={{ borderRadius: 18 }}>
            <Empty description={searchText || courseTypeFilter !== "all" ? "没有符合当前筛选条件的课程" : "暂无课程"} />
          </Card>
        ) : (
          <>
            <Row gutter={[18, 18]}>
              {paginatedCourses.map((course) => {
                const isSelected = selectedCourseIds.includes(course.id);

                return (
                  <Col key={course.id} xs={24} md={12} xl={8}>
                    <Card
                      size="small"
                      hoverable
                      role="button"
                      tabIndex={0}
                      onClick={() => void handleOpenTaskSelector(course.id)}
                      onKeyDown={(event) => {
                        if (event.key === "Enter" || event.key === " ") {
                          event.preventDefault();
                          void handleOpenTaskSelector(course.id);
                        }
                      }}
                      style={{
                        borderRadius: 18,
                        borderColor: isSelected ? "#b7d3ff" : "#edf1f6",
                        background: isSelected ? "linear-gradient(180deg, #f8fbff 0%, #ffffff 100%)" : "#ffffff",
                        boxShadow: isSelected
                          ? "0 14px 28px rgba(22,119,255,0.10)"
                          : "0 10px 24px rgba(15,23,42,0.05)",
                        transition: "all 0.2s ease",
                        cursor: "pointer",
                      }}
                      styles={{ body: { padding: 20 } }}
                    >
                      <div style={{ display: "flex", flexDirection: "column", gap: 14, minHeight: 182 }}>
                        <div style={{ display: "flex", justifyContent: "space-between", gap: 12, alignItems: "flex-start" }}>
                          <div style={{ flex: 1, minWidth: 0 }}>
                            <Text strong style={{ display: "block", fontSize: 16, lineHeight: 1.6 }}>
                              {course.title}
                            </Text>
                          </div>
                          <Tag color={getCourseTypeTagColor(course.courseType)} style={{ marginInlineEnd: 0 }}>
                            {course.courseTypeLabel}
                          </Tag>
                        </div>

                        <Space wrap size={[8, 8]}>
                          {isSelected && <Tag color="success">已配置</Tag>}
                          <Tag bordered={false} color="default">
                            教师：{course.teacher || "未知教师"}
                          </Tag>
                        </Space>

                        <div style={{ minHeight: 48 }}>
                          {course.desc ? (
                            <Paragraph
                              type="secondary"
                              ellipsis={{ rows: 2, expandable: false }}
                              style={{ margin: 0, lineHeight: 1.75 }}
                            >
                              {course.desc}
                            </Paragraph>
                          ) : (
                            <Text type="secondary" style={{ lineHeight: 1.75 }}>
                              暂无课程简介，可直接进入任务配置查看章节与任务。
                            </Text>
                          )}
                        </div>

                        <div
                          style={{
                            marginTop: "auto",
                            paddingTop: 12,
                            borderTop: "1px solid #f0f2f5",
                            display: "flex",
                            justifyContent: "space-between",
                            alignItems: "center",
                            gap: 12,
                          }}
                        >
                          <Text type="secondary" style={{ fontSize: 12 }}>
                            课程 ID：{course.courseId}
                          </Text>
                          <Text style={{ color: "#1677ff", fontSize: 13 }}>
                            进入配置 <RightOutlined style={{ fontSize: 12 }} />
                          </Text>
                        </div>
                      </div>
                    </Card>
                  </Col>
                );
              })}
            </Row>

            {filteredCourses.length > pageSize && (
              <Card
                style={{ borderRadius: 16 }}
                styles={{ body: { padding: "14px 18px" } }}
              >
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    gap: 16,
                    flexWrap: "wrap",
                  }}
                >
                  <Text type="secondary">
                    共 {filteredCourses.length} 门课程，当前第 {currentPage} / {Math.max(1, Math.ceil(filteredCourses.length / pageSize))} 页
                  </Text>
                  <Pagination
                    current={currentPage}
                    pageSize={pageSize}
                    total={filteredCourses.length}
                    showSizeChanger
                    pageSizeOptions={pageSizeOptions.map(String)}
                    onChange={(page, nextPageSize) => {
                      setCurrentPage(page);
                      if (nextPageSize !== pageSize) {
                        setPageSize(nextPageSize);
                      }
                    }}
                    showLessItems
                    showQuickJumper={false}
                  />
                </div>
              </Card>
            )}
          </>
        )}
      </div>

      <Card
        style={{
          ...surfaceCardStyle,
          position: "sticky",
          bottom: 0,
          boxShadow: "0 -4px 18px rgba(15,23,42,0.04)",
        }}
        styles={{ body: { padding: "16px 20px" } }}
      >
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
              已配置 {selectedCourseIds.length} 门课程
            </Text>
            <Text type="secondary" style={{ fontSize: 12 }}>
              完成任务配置后即可前往任务执行页开始学习
            </Text>
          </div>
          <Button
            type="primary"
            disabled={selectedCourseIds.length === 0}
            onClick={() => navigate("/tasks")}
            style={{ ...primaryActionButtonStyle, minWidth: 168 }}
          >
            开始学习
          </Button>
        </div>
      </Card>
    </div>
  );
}
