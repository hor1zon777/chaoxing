export type CourseType = "all" | "course" | "archive" | "other";
export type JobType = "video" | "document" | "read" | "live" | "workid";

/** 课程信息 */
export interface Course {
  id: string;
  courseId: string;
  clazzId: string;
  cpi: string;
  title: string;
  teacher: string;
  desc: string;
  info: string;
  roleid: string;
  courseType: CourseType;
  courseTypeLabel: string;
}

/** 任务点 */
export interface CourseJob {
  id: string;
  name: string;
  jobType: JobType;
  typeLabel: string;
  isCompleted: boolean;
}

/** 章节知识点 */
export interface ChapterPoint {
  id: string;
  title: string;
  jobCount: number;
  hasFinished: boolean;
  needUnlock: boolean;
}

/** 带任务的章节节点 */
export interface ChapterSelectionPoint extends ChapterPoint {
  jobs: CourseJob[];
}

/** 章节树结构 */
export interface ChapterTree {
  hasLocked: boolean;
  points: ChapterPoint[];
}

/** 课程内容选择树 */
export interface CourseSelectionTree {
  hasLocked: boolean;
  points: ChapterSelectionPoint[];
}

/** 扁平任务项 */
export interface FlatCourseJob extends CourseJob {
  pointId: string;
  pointTitle: string;
  hasFinished: boolean;
  needUnlock: boolean;
}

/** 学习范围中的章节选择 */
export interface CoursePointSelection {
  pointId: string;
  selectedJobIds: string[];
}

/** 单门课程的学习范围 */
export interface CourseLearningSelection {
  courseId: string;
  clazzId: string;
  cpi: string;
  title: string;
  selectedPoints: CoursePointSelection[];
}
