import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type {
  Course,
  CourseLearningSelection,
  CourseSelectionTree,
  CourseType,
  FlatCourseJob,
  JobType,
} from "../types/course";

/** 课程状态 */
interface CourseState {
  /** 课程列表 */
  courses: Course[];
  /** 已配置任务的课程 ID 集合 */
  selectedCourseIds: string[];
  /** 当前任务选择页课程 */
  activeCourseId: string | null;
  /** 课程类型筛选 */
  courseTypeFilter: CourseType;
  /** 课程内容缓存 */
  courseTrees: Record<string, CourseSelectionTree>;
  /** 课程内容加载状态 */
  treeLoadingIds: string[];
  /** 已选中的课程章节/任务 */
  learningSelections: Record<string, CourseLearningSelection>;
  /** 是否加载中 */
  isLoading: boolean;
  /** 错误信息 */
  error: string | null;
  /** 获取课程列表 */
  fetchCourses: () => Promise<void>;
  /** 获取课程内容树 */
  fetchCourseTree: (course: Course) => Promise<CourseSelectionTree | null>;
  /** 激活当前课程并确保任务树已加载 */
  activateCourse: (course: Course) => Promise<void>;
  /** 设置当前任务选择页课程 */
  setActiveCourseId: (courseId: string | null) => void;
  /** 设置课程类型筛选 */
  setCourseTypeFilter: (courseType: CourseType) => void;
  /** 切换章节整体选择 */
  togglePointSelection: (course: Course, pointId: string, checked: boolean) => void;
  /** 切换任务点选择 */
  toggleJobSelection: (
    course: Course,
    pointId: string,
    jobId: string,
    checked: boolean,
  ) => void;
  /** 按任务类型快捷选择 */
  selectJobsByType: (course: Course, types: JobType[]) => void;
  /** 默认选中课程全部任务 */
  selectAllJobsForCourse: (course: Course) => void;
  /** 按范围批量更新任务点选择 */
  batchUpdateJobsForCourse: (
    course: Course,
    jobs: FlatCourseJob[],
    mode: "select" | "invert" | "clear",
  ) => void;
  /** 获取课程扁平任务列表 */
  getFlatJobsForCourse: (courseId: string) => FlatCourseJob[];
  /** 获取已选课程的学习范围 */
  getSelectedLearningSelections: () => CourseLearningSelection[];
  /** 重置课程相关状态 */
  reset: () => void;
}

function ensureSelection(
  state: CourseState,
  course: Course,
): CourseLearningSelection {
  return (
    state.learningSelections[course.id] ?? {
      courseId: course.courseId,
      clazzId: course.clazzId,
      cpi: course.cpi,
      title: course.title,
      selectedPoints: [],
    }
  );
}

function buildSelectionFromTree(
  course: Course,
  tree: CourseSelectionTree,
  types: JobType[] = [],
): CourseLearningSelection {
  return {
    courseId: course.courseId,
    clazzId: course.clazzId,
    cpi: course.cpi,
    title: course.title,
    selectedPoints: tree.points
      .map((point) => ({
        pointId: point.id,
        selectedJobIds: point.jobs
          .filter(
            (job) =>
              !job.isCompleted &&
              (types.length === 0 || types.includes(job.jobType)),
          )
          .map((job) => job.id),
      }))
      .filter((point) => point.selectedJobIds.length > 0),
  };
}

function syncSelectedCourseIds(
  selectedCourseIds: string[],
  courseId: string,
  selectedPointsLength: number,
): string[] {
  if (selectedPointsLength > 0) {
    return selectedCourseIds.includes(courseId)
      ? selectedCourseIds
      : [...selectedCourseIds, courseId];
  }

  return selectedCourseIds.filter((id) => id !== courseId);
}

function buildSelectionStateUpdate(
  state: CourseState,
  course: Course,
  selection: CourseLearningSelection,
) {
  return {
    learningSelections: {
      ...state.learningSelections,
      [course.id]: selection,
    },
    selectedCourseIds: syncSelectedCourseIds(
      state.selectedCourseIds,
      course.id,
      selection.selectedPoints.length,
    ),
  };
}

function buildSelectedPointsFromMap(
  tree: CourseSelectionTree | undefined,
  pointJobMap: Map<string, Set<string>>,
): CourseLearningSelection["selectedPoints"] {
  if (!tree) {
    return Array.from(pointJobMap.entries()).map(([pointId, selectedJobIds]) => ({
      pointId,
      selectedJobIds: Array.from(selectedJobIds),
    }));
  }

  return tree.points
    .map((point) => {
      const selectedJobIds = pointJobMap.get(point.id);
      if (!selectedJobIds || selectedJobIds.size === 0) {
        return null;
      }
      return {
        pointId: point.id,
        selectedJobIds: point.jobs
          .map((job) => job.id)
          .filter((jobId) => selectedJobIds.has(jobId)),
      };
    })
    .filter((point): point is NonNullable<typeof point> => point !== null);
}

function buildFlatJobs(tree: CourseSelectionTree | undefined): FlatCourseJob[] {
  if (!tree) {
    return [];
  }

  return tree.points.flatMap((point) =>
    point.jobs.map((job) => ({
      ...job,
      pointId: point.id,
      pointTitle: point.title,
      hasFinished: point.hasFinished,
      needUnlock: point.needUnlock,
    })),
  );
}

export const useCourseStore = create<CourseState>((set, get) => ({
  courses: [],
  selectedCourseIds: [],
  activeCourseId: null,
  courseTypeFilter: "all",
  courseTrees: {},
  treeLoadingIds: [],
  learningSelections: {},
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

  fetchCourseTree: async (course) => {
    const { courseTrees, treeLoadingIds } = get();
    if (courseTrees[course.id]) {
      return courseTrees[course.id];
    }
    if (treeLoadingIds.includes(course.id)) {
      return null;
    }

    set({ treeLoadingIds: [...treeLoadingIds, course.id], error: null });
    try {
      const tree = await invoke<CourseSelectionTree>("get_course_selection_tree", {
        courseId: course.courseId,
        clazzId: course.clazzId,
        cpi: course.cpi,
      });
      set((state) => ({
        courseTrees: {
          ...state.courseTrees,
          [course.id]: tree,
        },
        treeLoadingIds: state.treeLoadingIds.filter((id) => id !== course.id),
      }));
      return tree;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      set((state) => ({
        treeLoadingIds: state.treeLoadingIds.filter((id) => id !== course.id),
        error: message,
      }));
      return null;
    }
  },

  activateCourse: async (course) => {
    const state = get();
    let tree: CourseSelectionTree | null = state.courseTrees[course.id] ?? null;

    if (!tree) {
      tree = await state.fetchCourseTree(course);
    }

    set((currentState) => ({
      activeCourseId: course.id,
      selectedCourseIds: syncSelectedCourseIds(
        currentState.selectedCourseIds,
        course.id,
        currentState.learningSelections[course.id]?.selectedPoints.length ?? 0,
      ),
    }));
  },

  setActiveCourseId: (activeCourseId) => set({ activeCourseId }),

  setCourseTypeFilter: (courseTypeFilter) => set({ courseTypeFilter }),

  togglePointSelection: (course, pointId, checked) => {
    const state = get();
    const selection = ensureSelection(state, course);
    const point = state.courseTrees[course.id]?.points.find((item) => item.id === pointId);
    const selectedPoints = selection.selectedPoints.filter((item) => item.pointId !== pointId);

    if (checked && point) {
      selectedPoints.push({
        pointId,
        selectedJobIds: point.jobs
        .filter((job) => !job.isCompleted)
        .map((job) => job.id),
      });
    }

    set(
      buildSelectionStateUpdate(state, course, {
        ...selection,
        selectedPoints,
      }),
    );
  },

  toggleJobSelection: (course, pointId, jobId, checked) => {
    const state = get();
    const selection = ensureSelection(state, course);
    const selectedPoints = [...selection.selectedPoints];
    const pointIndex = selectedPoints.findIndex((item) => item.pointId === pointId);

    if (pointIndex === -1) {
      if (!checked) {
        return;
      }
      selectedPoints.push({ pointId, selectedJobIds: [jobId] });
    } else {
      const target = selectedPoints[pointIndex];
      const selectedJobIds = checked
        ? Array.from(new Set([...target.selectedJobIds, jobId]))
        : target.selectedJobIds.filter((id) => id !== jobId);

      if (selectedJobIds.length === 0) {
        selectedPoints.splice(pointIndex, 1);
      } else {
        selectedPoints[pointIndex] = {
          ...target,
          selectedJobIds,
        };
      }
    }

    set(
      buildSelectionStateUpdate(state, course, {
        ...selection,
        selectedPoints,
      }),
    );
  },

  selectJobsByType: (course, types) => {
    const state = get();
    const tree = state.courseTrees[course.id];
    if (!tree) {
      return;
    }

    set(buildSelectionStateUpdate(state, course, buildSelectionFromTree(course, tree, types)));
  },

  selectAllJobsForCourse: (course) => {
    const state = get();
    const tree = state.courseTrees[course.id];
    if (!tree) {
      return;
    }

    set(buildSelectionStateUpdate(state, course, buildSelectionFromTree(course, tree)));
  },

  batchUpdateJobsForCourse: (course, jobs, mode) => {
    if (jobs.length === 0) {
      return;
    }

    const state = get();
    const selection = ensureSelection(state, course);
    const pointJobMap = new Map(
      selection.selectedPoints.map((point) => [point.pointId, new Set(point.selectedJobIds)]),
    );

    jobs.forEach((job) => {
      const currentSet = pointJobMap.get(job.pointId) ?? new Set<string>();

      if (mode === "select") {
        currentSet.add(job.id);
      } else if (mode === "clear") {
        currentSet.delete(job.id);
      } else if (currentSet.has(job.id)) {
        currentSet.delete(job.id);
      } else {
        currentSet.add(job.id);
      }

      if (currentSet.size === 0) {
        pointJobMap.delete(job.pointId);
      } else {
        pointJobMap.set(job.pointId, currentSet);
      }
    });

    set(
      buildSelectionStateUpdate(state, course, {
        ...selection,
        selectedPoints: buildSelectedPointsFromMap(state.courseTrees[course.id], pointJobMap),
      }),
    );
  },

  getFlatJobsForCourse: (courseId) => buildFlatJobs(get().courseTrees[courseId]),

  getSelectedLearningSelections: () => {
    const { selectedCourseIds, courses, learningSelections } = get();
    return selectedCourseIds
      .map((courseId) => {
        const course = courses.find((item) => item.id === courseId);
        if (!course) {
          return null;
        }
        const selection = learningSelections[course.id];
        if (!selection || selection.selectedPoints.length === 0) {
          return null;
        }
        return {
          courseId: course.courseId,
          clazzId: course.clazzId,
          cpi: course.cpi,
          title: course.title,
          selectedPoints: selection.selectedPoints,
        };
      })
      .filter((item): item is CourseLearningSelection => item !== null);
  },

  reset: () =>
    set({
      courses: [],
      selectedCourseIds: [],
      activeCourseId: null,
      courseTypeFilter: "all",
      courseTrees: {},
      treeLoadingIds: [],
      learningSelections: {},
      isLoading: false,
      error: null,
    }),
}));
