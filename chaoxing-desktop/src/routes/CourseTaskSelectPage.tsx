import { useEffect, useMemo, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import {
  Alert,
  Button,
  Card,
  Checkbox,
  Empty,
  List,
  Select,
  Space,
  Spin,
  Tag,
  Typography,
  Grid,
} from "antd";
import { ArrowLeftOutlined } from "@ant-design/icons";
import { useCourseStore } from "../stores/courseStore";
import type { FlatCourseJob, JobType } from "../types/course";
import {
  GuideBanner,
  pageTopBannerTexts,
  panelStyle,
  primaryActionButtonStyle,
  SectionTitle,
  SummaryMetric,
  surfaceCardStyle,
} from "../components/ui/pagePrimitives";

const { Text, Title, Paragraph } = Typography;
const { useBreakpoint } = Grid;

const quickSelectOptions: Array<{ label: string; types: JobType[] }> = [
  { label: "仅视频", types: ["video"] },
  { label: "视频+阅读", types: ["video", "read"] },
  { label: "全部任务", types: [] },
];

const jobTypeOptions: Array<{ label: string; value: JobType }> = [
  { label: "视频", value: "video" },
  { label: "文档", value: "document" },
  { label: "阅读", value: "read" },
  { label: "直播", value: "live" },
  { label: "作业", value: "workid" },
];

function getJobTypeColor(jobType: JobType): string {
  switch (jobType) {
    case "video":
      return "blue";
    case "document":
      return "gold";
    case "read":
      return "geekblue";
    case "live":
      return "purple";
    case "workid":
      return "cyan";
    default:
      return "default";
  }
}

export function CourseTaskSelectPage() {
  const screens = useBreakpoint();
  const navigate = useNavigate();
  const { courseId } = useParams<{ courseId: string }>();
  const [selectedJobTypes, setSelectedJobTypes] = useState<JobType[]>([]);
  const {
    courses,
    courseTrees,
    treeLoadingIds,
    learningSelections,
    isLoading: coursesLoading,
    error,
    activateCourse,
    fetchCourses,
    selectJobsByType,
    selectAllJobsForCourse,
    batchUpdateJobsForCourse,
    toggleJobSelection,
    getFlatJobsForCourse,
  } = useCourseStore();

  const course = useMemo(
    () => courses.find((item) => item.id === courseId),
    [courses, courseId],
  );

  useEffect(() => {
    if (!courseId || courses.length > 0) {
      return;
    }
    void fetchCourses();
  }, [courseId, courses.length, fetchCourses]);

  useEffect(() => {
    if (course) {
      void activateCourse(course);
    }
  }, [course, activateCourse]);

  const flatJobs = useMemo(
    () => (courseId ? getFlatJobsForCourse(courseId) : []),
    [courseId, getFlatJobsForCourse, courseTrees],
  );

  const filteredJobs = useMemo(() => {
    if (selectedJobTypes.length === 0) {
      return flatJobs;
    }
    return flatJobs.filter((job) => selectedJobTypes.includes(job.jobType));
  }, [flatJobs, selectedJobTypes]);

  const selectedJobKeys = useMemo(() => {
    if (!course) {
      return [] as string[];
    }

    return (
      learningSelections[course.id]?.selectedPoints.flatMap((point) =>
        point.selectedJobIds.map((jobId) => `${point.pointId}:${jobId}`),
      ) ?? []
    );
  }, [course, learningSelections]);

  const selectedJobKeySet = useMemo(
    () => new Set(selectedJobKeys),
    [selectedJobKeys],
  );

  const filteredSelectedCount = useMemo(
    () =>
      filteredJobs.filter((job) => selectedJobKeySet.has(`${job.pointId}:${job.id}`)).length,
    [filteredJobs, selectedJobKeySet],
  );

  const selectedJobCount = selectedJobKeys.length;
  const completedJobCount = useMemo(
    () => flatJobs.filter((job) => job.isCompleted).length,
    [flatJobs],
  );
  const pendingJobCount = flatJobs.length - completedJobCount;
  const chapterCount = useMemo(
    () => new Set(flatJobs.map((job) => job.pointId)).size,
    [flatJobs],
  );

  const isLoading = coursesLoading || (course ? treeLoadingIds.includes(course.id) : false);
  const batchActionDisabled = isLoading || filteredJobs.length === 0;

  if (!courseId || (!course && !isLoading)) {
    return (
      <Card style={surfaceCardStyle}>
        <Empty description="课程不存在或已失效" />
      </Card>
    );
  }

  if (!course) {
    return (
      <Card style={surfaceCardStyle}>
        <div style={{ textAlign: "center", padding: 40 }}>
          <Spin size="large" tip="加载课程中..." />
        </div>
      </Card>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
      <GuideBanner text={pageTopBannerTexts.courseTasks} />

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
          <div style={{ maxWidth: 840 }}>
            <Space wrap size={[8, 8]}>
              <Button
                icon={<ArrowLeftOutlined />}
                onClick={() => navigate("/courses")}
                style={{ borderRadius: 12 }}
              >
                返回课程列表
              </Button>
              <Tag color="blue" style={{ marginInlineEnd: 0 }}>
                {course.courseTypeLabel}
              </Tag>
              <Tag color="cyan" style={{ marginInlineEnd: 0 }}>
                已选 {selectedJobCount} 个任务
              </Tag>
            </Space>

            <Title level={4} style={{ margin: "14px 0 6px" }}>
              {course.title}
            </Title>
            <Paragraph type="secondary" style={{ margin: 0, lineHeight: 1.7 }}>
              教师：{course.teacher || "未知教师"}。可按类型筛选任务，并勾选需要学习的章节与任务点。
            </Paragraph>
          </div>

          <div
            style={{
              ...panelStyle,
              minWidth: screens.md ? 240 : "100%",
              background: "linear-gradient(180deg, #f8fbff 0%, #ffffff 100%)",
            }}
          >
            <Text strong style={{ display: "block", marginBottom: 8 }}>
              当前筛选摘要
            </Text>
            <Text type="secondary" style={{ display: "block", fontSize: 12, lineHeight: 1.7 }}>
              当前筛选 {filteredJobs.length} 个任务，已选 {filteredSelectedCount} 个。
            </Text>
            <Text type="secondary" style={{ display: "block", fontSize: 12, lineHeight: 1.7 }}>
              总共 {chapterCount} 个章节 / {flatJobs.length} 个任务点。
            </Text>
            <Text type="secondary" style={{ display: "block", fontSize: 12, lineHeight: 1.7 }}>
              快捷选择会直接按类型重置当前课程的任务勾选结果。
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
          <SummaryMetric label="章节数" value={`${chapterCount}`} hint="可配置章节" />
          <SummaryMetric label="任务总数" value={`${flatJobs.length}`} hint="当前课程全部任务点" />
          <SummaryMetric label="未完成" value={`${pendingJobCount}`} hint="可继续学习" />
          <SummaryMetric label="已选任务" value={`${selectedJobCount}`} hint="将带入任务执行页" />
        </div>
      </Card>

      {error && <Alert type="error" message={error} style={{ borderRadius: 12 }} />}

      <Card
        title={<SectionTitle title="筛选与批量操作" subtitle="先筛选，再对当前结果执行全选、反选或清空" />}
        style={surfaceCardStyle}
        styles={{ body: { padding: 18 } }}
      >
        <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "flex-end",
              gap: 16,
              flexWrap: "wrap",
            }}
          >
            <div style={{ minWidth: 260, flex: 1 }}>
              <Text type="secondary" style={{ display: "block", marginBottom: 8, fontSize: 12 }}>
                任务类型筛选
              </Text>
              <Select
                mode="multiple"
                allowClear
                placeholder="筛选任务类型"
                value={selectedJobTypes}
                options={jobTypeOptions}
                onChange={(value) => setSelectedJobTypes(value)}
                disabled={isLoading}
                style={{ width: "100%" }}
                maxTagCount="responsive"
              />
            </div>

            <div style={{ minWidth: 260, flex: 1 }}>
              <Text type="secondary" style={{ display: "block", marginBottom: 8, fontSize: 12 }}>
                快捷选择
              </Text>
              <div style={{ display: "flex", flexWrap: "wrap", gap: 10 }}>
                {quickSelectOptions.map((option) => (
                  <Button
                    key={option.label}
                    disabled={isLoading}
                    onClick={() => selectJobsByType(course, option.types)}
                    style={{ borderRadius: 12 }}
                  >
                    {option.label}
                  </Button>
                ))}
              </div>
            </div>
          </div>

          <div style={{ display: "flex", flexWrap: "wrap", gap: 10 }}>
            <Button
              disabled={batchActionDisabled}
              onClick={() =>
                batchUpdateJobsForCourse(
                  course,
                  filteredJobs.filter((job) => !job.isCompleted),
                  "select",
                )
              }
              style={{ borderRadius: 12 }}
            >
              全选当前筛选
            </Button>
            <Button
              disabled={batchActionDisabled}
              onClick={() =>
                batchUpdateJobsForCourse(
                  course,
                  filteredJobs.filter((job) => !job.isCompleted),
                  "invert",
                )
              }
              style={{ borderRadius: 12 }}
            >
              反选当前筛选
            </Button>
            <Button
              disabled={batchActionDisabled}
              onClick={() =>
                batchUpdateJobsForCourse(
                  course,
                  filteredJobs.filter((job) => !job.isCompleted),
                  "clear",
                )
              }
              style={{ borderRadius: 12 }}
            >
              清空当前筛选
            </Button>
            <Button
              disabled={isLoading}
              onClick={() => selectAllJobsForCourse(course)}
              style={{ borderRadius: 12 }}
            >
              选择全部任务
            </Button>
          </div>
        </div>
      </Card>

      <Card
        title={<SectionTitle title="章节任务列表" subtitle="逐项勾选任务点，已完成任务会自动禁用" />}
        style={surfaceCardStyle}
        styles={{ body: { padding: 18 } }}
      >
        {isLoading ? (
          <div style={{ textAlign: "center", padding: 40 }}>
            <Spin size="large" tip="加载任务中..." />
          </div>
        ) : filteredJobs.length === 0 ? (
          <Empty
            description={
              selectedJobTypes.length === 0 ? "暂无可选任务" : "当前筛选下暂无任务"
            }
          />
        ) : (
          <List
            itemLayout="horizontal"
            dataSource={filteredJobs}
            renderItem={(job: FlatCourseJob) => (
              <List.Item key={`${job.pointId}-${job.id}`} style={{ padding: "10px 0" }}>
                <div
                  style={{
                    ...panelStyle,
                    width: "100%",
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    gap: 16,
                    flexWrap: "wrap",
                  }}
                >
                  <Checkbox
                    checked={selectedJobKeySet.has(`${job.pointId}:${job.id}`)}
                    disabled={job.isCompleted}
                    onChange={(e) =>
                      toggleJobSelection(
                        course,
                        job.pointId,
                        job.id,
                        e.target.checked,
                      )
                    }
                    style={{ flex: 1, minWidth: 240 }}
                  >
                    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                      <Text strong style={{ lineHeight: 1.6 }}>
                        {job.name}
                      </Text>
                      <Space size={[8, 8]} wrap>
                        {job.isCompleted ? (
                          <Tag color="success" style={{ marginInlineEnd: 0 }}>
                            已完成
                          </Tag>
                        ) : (
                          <Tag style={{ marginInlineEnd: 0 }}>未完成</Tag>
                        )}
                        {job.needUnlock && (
                          <Tag color="orange" style={{ marginInlineEnd: 0 }}>
                            待开放
                          </Tag>
                        )}
                        {job.hasFinished && (
                          <Tag color="blue" style={{ marginInlineEnd: 0 }}>
                            章节已完成
                          </Tag>
                        )}
                        <Tag bordered={false} style={{ marginInlineEnd: 0 }}>
                          {job.pointTitle}
                        </Tag>
                      </Space>
                    </div>
                  </Checkbox>
                  <Tag color={getJobTypeColor(job.jobType)} style={{ marginInlineEnd: 0 }}>
                    {job.typeLabel}
                  </Tag>
                </div>
              </List.Item>
            )}
          />
        )}
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
              已选择 {selectedJobCount} 个任务点
            </Text>
            <Text type="secondary" style={{ fontSize: 12 }}>
              选择会自动保存，可直接返回课程列表，后续再前往任务执行页开始学习
            </Text>
          </div>
          <Button type="primary" onClick={() => navigate("/courses")} style={{ ...primaryActionButtonStyle }}>
            返回课程列表
          </Button>
        </div>
      </Card>
    </div>
  );
}

