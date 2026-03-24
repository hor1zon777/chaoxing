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
}

/** 章节知识点 */
export interface ChapterPoint {
  id: string;
  title: string;
  jobCount: number;
  hasFinished: boolean;
  needUnlock: boolean;
}

/** 章节树结构 */
export interface ChapterTree {
  hasLocked: boolean;
  points: ChapterPoint[];
}
