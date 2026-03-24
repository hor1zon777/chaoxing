import { useEffect, useState } from "react";
import {
  Input,
  Button,
  Row,
  Col,
  Card,
  Checkbox,
  Empty,
  Space,
  Spin,
  Alert,
  Typography,
} from "antd";
import { ReloadOutlined } from "@ant-design/icons";
import { useNavigate } from "react-router-dom";
import { useCourseStore } from "../stores/courseStore";

const { Search } = Input;
const { Text } = Typography;

/** 课程列表页 */
export function CoursesPage() {
  const navigate = useNavigate();
  const {
    courses,
    selectedCourseIds,
    isLoading,
    error,
    fetchCourses,
    toggleCourseSelection,
    selectAll,
    deselectAll,
  } = useCourseStore();

  useEffect(() => {
    // 首次加载时获取课程列表
    if (courses.length === 0) {
      fetchCourses();
    }
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  /** 搜索过滤（本地过滤） */
  const [searchText, setSearchText] = useState("");

  const filteredCourses = searchText
    ? courses.filter(
        (c) =>
          c.title.includes(searchText) || c.teacher.includes(searchText),
      )
    : courses;

  /** 全选/反选切换 */
  const handleToggleAll = () => {
    if (selectedCourseIds.length === courses.length) {
      deselectAll();
    } else {
      selectAll();
    }
  };

  /** 开始学习 */
  const handleStartLearning = () => {
    if (selectedCourseIds.length > 0) {
      navigate("/tasks");
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      {/* 顶部工具栏 */}
      <Space style={{ marginBottom: 16 }} wrap>
        <Search
          placeholder="搜索课程名称或教师"
          allowClear
          onSearch={setSearchText}
          onChange={(e) => {
            if (!e.target.value) setSearchText("");
          }}
          style={{ width: 240 }}
        />
        <Button icon={<ReloadOutlined />} onClick={fetchCourses}>
          刷新
        </Button>
        <Button onClick={selectAll}>全选</Button>
        <Button onClick={handleToggleAll}>反选</Button>
      </Space>

      {/* 错误提示 */}
      {error && (
        <Alert
          message={error}
          type="error"
          closable
          style={{ marginBottom: 16 }}
        />
      )}

      {/* 课程列表 */}
      <div style={{ flex: 1, overflow: "auto" }}>
        {isLoading ? (
          <div style={{ textAlign: "center", padding: 48 }}>
            <Spin size="large" tip="加载课程中..." />
          </div>
        ) : filteredCourses.length === 0 ? (
          <Empty description="暂无课程" />
        ) : (
          <Row gutter={[16, 16]}>
            {filteredCourses.map((course) => (
              <Col key={course.id} xs={24} sm={12} lg={8}>
                <Card
                  hoverable
                  size="small"
                  onClick={() => toggleCourseSelection(course.id)}
                  style={{
                    borderColor: selectedCourseIds.includes(course.id)
                      ? "#1677ff"
                      : undefined,
                  }}
                >
                  <div
                    style={{
                      display: "flex",
                      alignItems: "flex-start",
                      gap: 8,
                    }}
                  >
                    <Checkbox
                      checked={selectedCourseIds.includes(course.id)}
                      onClick={(e) => e.stopPropagation()}
                      onChange={() => toggleCourseSelection(course.id)}
                    />
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <div
                        style={{
                          fontWeight: 600,
                          marginBottom: 4,
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          whiteSpace: "nowrap",
                        }}
                      >
                        {course.title}
                      </div>
                      <Text type="secondary" style={{ fontSize: 12 }}>
                        教师: {course.teacher}
                      </Text>
                      {course.desc && (
                        <div
                          style={{
                            fontSize: 12,
                            color: "#999",
                            marginTop: 4,
                            overflow: "hidden",
                            textOverflow: "ellipsis",
                            whiteSpace: "nowrap",
                          }}
                        >
                          {course.desc}
                        </div>
                      )}
                      <Text
                        type="secondary"
                        style={{ fontSize: 11, display: "block", marginTop: 4 }}
                      >
                        课程ID: {course.courseId}
                      </Text>
                    </div>
                  </div>
                </Card>
              </Col>
            ))}
          </Row>
        )}
      </div>

      {/* 底部操作栏 */}
      <div
        style={{
          marginTop: 16,
          paddingTop: 16,
          borderTop: "1px solid #f0f0f0",
          textAlign: "right",
        }}
      >
        <Button
          type="primary"
          size="large"
          disabled={selectedCourseIds.length === 0}
          onClick={handleStartLearning}
        >
          开始学习 ({selectedCourseIds.length} 门课程)
        </Button>
      </div>
    </div>
  );
}
