import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Course } from "../types/course";

/** 课程状态 */
interface CourseState {
  /** 课程列表 */
  courses: Course[];
  /** 已选中的课程 ID 集合 */
  selectedCourseIds: string[];
  /** 是否加载中 */
  isLoading: boolean;
  /** 错误信息 */
  error: string | null;
  /** 获取课程列表 */
  fetchCourses: () => Promise<void>;
  /** 切换单个课程选中状态 */
  toggleCourseSelection: (courseId: string) => void;
  /** 全选 */
  selectAll: () => void;
  /** 取消全选 */
  deselectAll: () => void;
}

export const useCourseStore = create<CourseState>((set, get) => ({
  courses: [],
  selectedCourseIds: [],
  isLoading: false,
  error: null,

  fetchCourses: async () => {
    set({ isLoading: true, error: null });
    try {
      const courses = await invoke<Course[]>("get_courses");
      set({ courses, isLoading: false });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      set({ isLoading: false, error: message });
    }
  },

  toggleCourseSelection: (courseId: string) => {
    const { selectedCourseIds } = get();
    const isSelected = selectedCourseIds.includes(courseId);
    set({
      selectedCourseIds: isSelected
        ? selectedCourseIds.filter((id) => id !== courseId)
        : [...selectedCourseIds, courseId],
    });
  },

  selectAll: () => {
    const { courses } = get();
    set({ selectedCourseIds: courses.map((c) => c.id) });
  },

  deselectAll: () => {
    set({ selectedCourseIds: [] });
  },
}));
