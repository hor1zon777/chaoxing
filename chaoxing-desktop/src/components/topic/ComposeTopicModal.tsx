import { useState } from "react";
import { Alert, Input, InputNumber, message, Modal } from "antd";
import type { Course } from "../../types/course";
import type { AddTopicResult } from "../../types/topic";
import { useTopicStore } from "../../stores/topicStore";
import { PillButton, UtilityButton } from "../ui/appleUI";

/** 单次最多发送条数（防止误操作产生海量发帖） */
const MAX_COUNT = 50;

const clampCount = (n: number): number =>
  Math.max(1, Math.min(MAX_COUNT, Math.floor(Number.isFinite(n) ? n : 1)));

interface ComposeTopicModalProps {
  open: boolean;
  /** 目标课程（提供 courseId / clazzId / title） */
  course: Course;
  onClose: () => void;
  /**
   * 发布成功回调（含部分成功）：交由页面展示提示、乐观插入列表。
   * results 为已成功发布的话题（按发布顺序）；error 为遇到的首个错误（停在此处），无错误则为 null。
   */
  onSuccess: (
    results: AddTopicResult[],
    draft: { title: string; content: string },
    error: string | null,
  ) => void;
}

/**
 * 发布课程讨论话题的弹窗。
 *
 * 真实写操作安全：
 * 1) 顶部常驻 warning Alert，点名具体课程、明示“师生可见、无法撤回”；
 * 2) 主按钮文案为“发布话题”（非“保存/提交”），避免误认为草稿；
 * 3) 点击后必经 Modal.confirm 二次确认（红色危险按钮，回显标题+正文预览+数量），仅显式确认才真正发帖；
 * 4) 提交中 submitting 锁按钮/表单防重复，批量时按钮显示进度；失败保留草稿。
 *
 * 校验：标题与正文【其一非空】即可发布（与后端一致）。
 * 数量：可一次连续发布 1–50 条相同内容，遇到第一个错误即停止。
 *
 * 成功提示由页面（onSuccess）展示——本弹窗的 message 实例随弹窗关闭而卸载，
 * 故成功消息放在常驻挂载的页面上，仅错误消息（弹窗保持打开）用本地实例。
 */
export function ComposeTopicModal({ open, course, onClose, onSuccess }: ComposeTopicModalProps) {
  const [msgApi, contextHolder] = message.useMessage();
  // 用实例方法而非静态 Modal.confirm：静态方法渲染在 ConfigProvider 之外，
  // 无法继承全局主题 token / zhCN locale，且会触发 antd dev 警告。
  const [modal, modalHolder] = Modal.useModal();
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [count, setCount] = useState(1);
  // 批量发布进度（done/total）；null 表示当前不在批量发布中
  const [progress, setProgress] = useState<{ done: number; total: number } | null>(null);
  const sendTopicBatch = useTopicStore((s) => s.sendTopicBatch);
  const submitting = useTopicStore((s) => s.submitting);

  const hasText = title.trim().length > 0 || content.trim().length > 0;
  const canSubmit = hasText && !submitting;
  const effectiveCount = clampCount(count);

  const resetForm = () => {
    setTitle("");
    setContent("");
    setCount(1);
  };

  const handleClose = () => {
    if (submitting) return; // 提交中禁止关闭
    onClose();
  };

  const doSubmit = async () => {
    const trimmedTitle = title.trim();
    const trimmedContent = content.trim();
    if (!trimmedTitle && !trimmedContent) return;
    const total = clampCount(count);
    setProgress({ done: 0, total });
    try {
      const { results, error } = await sendTopicBatch(
        course.courseId,
        course.clazzId,
        trimmedTitle,
        trimmedContent,
        total,
        (done) => setProgress({ done, total }),
      );
      if (results.length > 0) {
        // 至少成功 1 条：交由页面提示 + 乐观插入 + 关闭弹窗（含部分成功时附带 error）
        onSuccess(results, { title: trimmedTitle, content: trimmedContent }, error);
        resetForm();
      } else {
        // 一条都没成功：弹窗保持打开、草稿保留，便于修改后重试
        msgApi.error(error || "发布失败");
      }
    } finally {
      setProgress(null);
    }
  };

  const handlePublishClick = () => {
    if (!canSubmit) return;
    const total = clampCount(count);
    const titleText = title.trim() || "(无标题)";
    const contentText = content.trim() || "(无正文)";
    const contentPreview = contentText.length > 80 ? `${contentText.slice(0, 80)}…` : contentText;
    modal.confirm({
      title:
        total > 1
          ? `确认发布 ${total} 条到《${course.title}》讨论区？`
          : `确认发布到《${course.title}》讨论区？`,
      content: (
        <div>
          <div style={{ marginBottom: 8, wordBreak: "break-word" }}>标题：{titleText}</div>
          <div
            style={{
              marginBottom: 8,
              color: "var(--apple-color-ink-muted-80)",
              fontSize: 13,
              lineHeight: 1.5,
              wordBreak: "break-word",
              whiteSpace: "pre-wrap",
            }}
          >
            正文：{contentPreview}
          </div>
          {total > 1 ? (
            <div style={{ marginBottom: 8, color: "var(--apple-color-ink-muted-80)", fontSize: 13, lineHeight: 1.5 }}>
              将连续发布 <b>{total}</b> 条相同内容（每条间隔约 0.5 秒，发布期间请勿关闭窗口）。
            </div>
          ) : null}
          <div style={{ color: "var(--apple-color-ink-muted-48)", fontSize: 13, lineHeight: 1.5 }}>
            发布后将真实出现在该课程讨论区，所有同班同学与教师可见，本工具无法撤回或删除。
          </div>
        </div>
      ),
      okText: total > 1 ? `确认发布 ${total} 条` : "确认发布",
      okButtonProps: { danger: true },
      cancelText: "再改改",
      onOk: () => doSubmit(),
    });
  };

  const publishLabel = submitting
    ? progress
      ? `发布中 ${progress.done}/${progress.total}`
      : "发布中…"
    : effectiveCount > 1
      ? `发布 ${effectiveCount} 条`
      : "发布话题";

  return (
    <Modal
      open={open}
      onCancel={handleClose}
      title="发布到课程讨论区"
      footer={null}
      maskClosable={!submitting}
      width={560}
    >
      {contextHolder}
      {modalHolder}
      <Alert
        type="warning"
        showIcon
        style={{ marginBottom: 16 }}
        message={`将以你的账号真实发布到《${course.title}》的讨论区，所有同班同学与教师可见，发布后本工具无法撤回 / 删除，请确认内容无误。`}
      />
      <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
        <div className="apple-field">
          <label className="apple-field__label">标题</label>
          <Input
            value={title}
            onChange={(event) => setTitle(event.target.value)}
            placeholder="标题与正文至少填写其一"
            maxLength={100}
            showCount
            disabled={submitting}
          />
        </div>
        <div className="apple-field">
          <label className="apple-field__label">正文</label>
          <Input.TextArea
            value={content}
            onChange={(event) => setContent(event.target.value)}
            placeholder="标题与正文至少填写其一（回车换行，不会直接发布）"
            autoSize={{ minRows: 6, maxRows: 12 }}
            maxLength={2000}
            showCount
            disabled={submitting}
          />
        </div>
        <div className="apple-field">
          <label className="apple-field__label">发送数量</label>
          <div style={{ display: "flex", alignItems: "center", gap: 10, flexWrap: "wrap" }}>
            <InputNumber
              min={1}
              max={MAX_COUNT}
              precision={0}
              value={count}
              onChange={(value) =>
                setCount(typeof value === "number" && Number.isFinite(value) ? value : 1)
              }
              disabled={submitting}
              style={{ width: 120 }}
            />
            <span className="apple-field__help">
              {effectiveCount > 1
                ? `将连续发布 ${effectiveCount} 条相同内容（间隔约 0.5 秒）`
                : `默认发布 1 条，最多 ${MAX_COUNT} 条`}
            </span>
          </div>
        </div>
      </div>
      <div style={{ display: "flex", justifyContent: "flex-end", gap: 12, marginTop: 20 }}>
        <UtilityButton light onClick={handleClose} disabled={submitting}>
          取消
        </UtilityButton>
        <PillButton onClick={handlePublishClick} disabled={!canSubmit}>
          {publishLabel}
        </PillButton>
      </div>
    </Modal>
  );
}
