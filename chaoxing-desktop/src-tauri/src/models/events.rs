use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "type")]
pub enum TaskEvent {
    CourseStarted {
        course_id: String,
        course_title: String,
        total_chapters: u32,
    },
    CourseCompleted {
        course_id: String,
        course_title: String,
    },
    CourseError {
        course_id: String,
        course_title: String,
        error: String,
    },
    ChapterStarted {
        course_id: String,
        chapter_id: String,
        chapter_title: String,
        job_count: u32,
    },
    ChapterCompleted {
        course_id: String,
        chapter_id: String,
        chapter_title: String,
    },
    ChapterSkipped {
        course_id: String,
        chapter_id: String,
        chapter_title: String,
        reason: String,
    },
    ChapterRetrying {
        course_id: String,
        chapter_id: String,
        chapter_title: String,
        attempt: u32,
        max_attempts: u32,
    },
    JobStarted {
        course_id: String,
        chapter_id: String,
        job_id: String,
        job_name: String,
        job_type: String,
    },
    JobCompleted {
        course_id: String,
        chapter_id: String,
        job_id: String,
        job_name: String,
        job_type: String,
    },
    JobFailed {
        course_id: String,
        chapter_id: String,
        job_id: String,
        job_name: String,
        error: String,
    },
    VideoProgress {
        course_id: String,
        job_id: String,
        job_name: String,
        current_time: u64,
        total_duration: u64,
    },
    LiveProgress {
        course_id: String,
        job_id: String,
        job_name: String,
        current_minute: u32,
        total_minutes: u32,
    },
    WorkQuestionAnswered {
        course_id: String,
        question_title: String,
        answer: String,
        source: String,
    },
    WorkSubmitted {
        course_id: String,
        chapter_title: String,
        cover_rate: f64,
        submitted: bool,
    },
    AllTasksCompleted,
    Log {
        level: String,
        message: String,
        timestamp: String,
    },
}
