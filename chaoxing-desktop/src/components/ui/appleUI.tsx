import type { CSSProperties, ReactNode, ButtonHTMLAttributes } from "react";

/**
 * Apple 风格的 UI 原语
 * 严格遵循 apple/DESIGN.md
 *
 * - PillButton: 主CTA 蓝色胶囊 / ghost / danger
 * - UtilityButton: 暗色 8px 圆角紧凑按钮
 * - Tile: 全宽色块容器 (light / parchment / dark)
 * - Card: 18px 圆角 + 1px hairline 的实用卡片
 * - Chip: pill 形态的可选项
 * - Tag: 小型信息徽章
 * - Section / Eyebrow / Headline / Subtitle 排版组件
 */

type PillVariant = "primary" | "ghost" | "danger";

interface PillButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "type"> {
  variant?: PillVariant;
  large?: boolean;
  fullWidth?: boolean;
  htmlType?: "button" | "submit" | "reset";
  icon?: ReactNode;
}

/** 主蓝色 pill CTA */
export function PillButton({
  children,
  variant = "primary",
  large,
  fullWidth,
  htmlType = "button",
  icon,
  style,
  className,
  ...rest
}: PillButtonProps) {
  const classes = ["apple-button-pill"];
  if (variant === "ghost") classes.push("apple-button-pill--ghost");
  if (variant === "danger") classes.push("apple-button-pill--danger");
  if (large) classes.push("apple-button-pill--lg");
  if (className) classes.push(className);

  return (
    <button
      type={htmlType}
      className={classes.join(" ")}
      style={{ width: fullWidth ? "100%" : undefined, ...style }}
      {...rest}
    >
      {icon ? <span aria-hidden style={{ display: "inline-flex" }}>{icon}</span> : null}
      {children}
    </button>
  );
}

interface UtilityButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "type"> {
  light?: boolean;
  htmlType?: "button" | "submit" | "reset";
  icon?: ReactNode;
}

/** 暗色 8px 圆角紧凑按钮，或浅色变体 */
export function UtilityButton({
  children,
  light,
  htmlType = "button",
  icon,
  className,
  ...rest
}: UtilityButtonProps) {
  const classes = ["apple-button-utility"];
  if (light) classes.push("apple-button-utility--light");
  if (className) classes.push(className);

  return (
    <button type={htmlType} className={classes.join(" ")} {...rest}>
      {icon ? <span aria-hidden style={{ display: "inline-flex" }}>{icon}</span> : null}
      {children}
    </button>
  );
}

type TileSurface = "light" | "parchment" | "dark" | "dark-2" | "dark-3";

interface TileProps {
  surface?: TileSurface;
  eyebrow?: ReactNode;
  title?: ReactNode;
  subtitle?: ReactNode;
  actions?: ReactNode;
  align?: "center" | "left";
  compact?: boolean;
  children?: ReactNode;
  style?: CSSProperties;
  innerStyle?: CSSProperties;
}

/** edge-to-edge 全宽色块 */
export function Tile({
  surface = "light",
  eyebrow,
  title,
  subtitle,
  actions,
  align = "center",
  compact,
  children,
  style,
  innerStyle,
}: TileProps) {
  const surfaceClass = `apple-tile--${surface}`;
  const isDark = surface.startsWith("dark");
  return (
    <section
      className={`apple-tile ${surfaceClass}`}
      style={{
        padding: compact ? "48px 22px" : undefined,
        textAlign: align,
        alignItems: align === "center" ? "center" : "flex-start",
        ...style,
      }}
    >
      <div className="apple-tile__inner" style={innerStyle}>
        {eyebrow ? <div className="apple-tile__eyebrow">{eyebrow}</div> : null}
        {title ? (
          <h2
            className="apple-tile__title"
            style={isDark ? { color: "var(--apple-color-body-on-dark)" } : undefined}
          >
            {title}
          </h2>
        ) : null}
        {subtitle ? <p className="apple-tile__subtitle">{subtitle}</p> : null}
        {actions ? <div className="apple-tile__actions">{actions}</div> : null}
        {children}
      </div>
    </section>
  );
}

interface CardProps {
  parchment?: boolean;
  title?: ReactNode;
  subtitle?: ReactNode;
  action?: ReactNode;
  padding?: number | string;
  style?: CSSProperties;
  className?: string;
  children?: ReactNode;
  onClick?: () => void;
  hoverable?: boolean;
  selected?: boolean;
}

/** 18px 圆角 1px hairline 的实用卡片 */
export function Card({
  parchment,
  title,
  subtitle,
  action,
  padding,
  style,
  className,
  children,
  onClick,
  hoverable,
  selected,
}: CardProps) {
  const classes = ["apple-card"];
  if (parchment) classes.push("apple-card--parchment");
  if (className) classes.push(className);

  const interactiveStyle: CSSProperties = onClick
    ? {
        cursor: "pointer",
        transition: "border-color 160ms ease, transform 100ms ease",
        ...(selected
          ? {
              borderColor: "var(--apple-color-primary-focus)",
              boxShadow: "0 0 0 1px var(--apple-color-primary-focus) inset",
            }
          : null),
      }
    : {};

  const hoverStyle: CSSProperties = hoverable
    ? {
        transition: "border-color 160ms ease, transform 100ms ease",
      }
    : {};

  return (
    <div
      className={classes.join(" ")}
      role={onClick ? "button" : undefined}
      tabIndex={onClick ? 0 : undefined}
      onClick={onClick}
      onKeyDown={(event) => {
        if (!onClick) return;
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onClick();
        }
      }}
      style={{
        padding: padding ?? 24,
        ...interactiveStyle,
        ...hoverStyle,
        ...style,
      }}
    >
      {(title || action) && (
        <div
          style={{
            display: "flex",
            alignItems: "flex-start",
            justifyContent: "space-between",
            gap: 12,
            marginBottom: subtitle ? 4 : children ? 16 : 0,
          }}
        >
          <div style={{ minWidth: 0 }}>
            {title ? <div className="apple-card__title">{title}</div> : null}
            {subtitle ? <div className="apple-card__subtitle">{subtitle}</div> : null}
          </div>
          {action ? <div style={{ flexShrink: 0 }}>{action}</div> : null}
        </div>
      )}
      {!title && subtitle ? <div className="apple-card__subtitle" style={{ marginBottom: 12 }}>{subtitle}</div> : null}
      {children}
    </div>
  );
}

interface ChipProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "type"> {
  selected?: boolean;
  children: ReactNode;
}

/** pill 形态的可选项 chip */
export function Chip({ selected, children, className, ...rest }: ChipProps) {
  const classes = ["apple-chip"];
  if (selected) classes.push("apple-chip--selected");
  if (className) classes.push(className);
  return (
    <button type="button" className={classes.join(" ")} aria-pressed={selected} {...rest}>
      {children}
    </button>
  );
}

interface TagProps {
  tone?: "default" | "neutral" | "success" | "warning" | "danger" | "dark";
  children: ReactNode;
  style?: CSSProperties;
}

/** 小型信息徽章 */
export function Tag({ tone = "default", children, style }: TagProps) {
  const classes = ["apple-tag"];
  if (tone === "neutral") classes.push("apple-tag--neutral");
  if (tone === "success") classes.push("apple-tag--success");
  if (tone === "warning") classes.push("apple-tag--warning");
  if (tone === "danger") classes.push("apple-tag--danger");
  if (tone === "dark") classes.push("apple-tag--dark");
  return (
    <span className={classes.join(" ")} style={style}>
      {children}
    </span>
  );
}

/** 段落 eyebrow 文本（蓝色小标） */
export function Eyebrow({ children, style }: { children: ReactNode; style?: CSSProperties }) {
  return (
    <div className="apple-eyebrow" style={style}>
      {children}
    </div>
  );
}

interface HeadlineProps {
  level?: "hero" | "lg" | "md" | "section";
  children: ReactNode;
  style?: CSSProperties;
  onDark?: boolean;
}

/** SF Pro Display 标题 */
export function Headline({ level = "lg", children, style, onDark }: HeadlineProps) {
  const classes = ["apple-headline"];
  if (level === "hero") classes.push("apple-headline--hero");
  if (level === "md") classes.push("apple-headline--md");
  if (level === "section") classes.push("apple-section-title");
  return (
    <h1
      className={classes.join(" ")}
      style={{ color: onDark ? "var(--apple-color-body-on-dark)" : undefined, ...style }}
    >
      {children}
    </h1>
  );
}

interface SubtitleProps {
  children: ReactNode;
  style?: CSSProperties;
  airy?: boolean;
  onDark?: boolean;
}

/** SF Pro Display 28px 副标 */
export function Subtitle({ children, style, airy, onDark }: SubtitleProps) {
  return (
    <p
      style={{
        fontFamily: airy ? "var(--apple-font-text)" : "var(--apple-font-display)",
        fontSize: airy ? 24 : 21,
        fontWeight: airy ? 300 : 400,
        lineHeight: airy ? 1.5 : 1.5,
        letterSpacing: airy ? 0 : "0.04px",
        color: onDark ? "var(--apple-color-body-muted)" : "var(--apple-color-ink-muted-80)",
        margin: 0,
        ...style,
      }}
    >
      {children}
    </p>
  );
}

interface SectionHeaderProps {
  eyebrow?: ReactNode;
  title: ReactNode;
  description?: ReactNode;
  action?: ReactNode;
  align?: "left" | "center";
  onDark?: boolean;
  style?: CSSProperties;
}

/** 卡片 / section 顶部的可复用组合 */
export function SectionHeader({
  eyebrow,
  title,
  description,
  action,
  align = "left",
  onDark,
  style,
}: SectionHeaderProps) {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "flex-end",
        gap: 16,
        flexWrap: "wrap",
        textAlign: align,
        ...style,
      }}
    >
      <div style={{ minWidth: 0, flex: "1 1 320px" }}>
        {eyebrow ? <Eyebrow>{eyebrow}</Eyebrow> : null}
        <Headline level="md" onDark={onDark} style={{ marginTop: eyebrow ? 8 : 0 }}>
          {title}
        </Headline>
        {description ? (
          <p
            style={{
              fontFamily: "var(--apple-font-text)",
              fontSize: 17,
              lineHeight: 1.47,
              letterSpacing: "-0.374px",
              color: onDark ? "var(--apple-color-body-muted)" : "var(--apple-color-ink-muted-48)",
              margin: "10px 0 0",
              maxWidth: 640,
            }}
          >
            {description}
          </p>
        ) : null}
      </div>
      {action ? <div style={{ flexShrink: 0 }}>{action}</div> : null}
    </div>
  );
}

interface MetricProps {
  label: ReactNode;
  value: ReactNode;
  hint?: ReactNode;
  onDark?: boolean;
}

/** 指标小卡：白底 / 极简 (无渐变) */
export function Metric({ label, value, hint, onDark }: MetricProps) {
  return (
    <div
      style={{
        padding: "16px 18px",
        borderRadius: 18,
        background: onDark ? "rgba(255, 255, 255, 0.06)" : "var(--apple-color-canvas)",
        border: onDark ? "1px solid rgba(255, 255, 255, 0.08)" : "1px solid var(--apple-color-hairline)",
        display: "flex",
        flexDirection: "column",
        gap: 6,
      }}
    >
      <span
        style={{
          fontFamily: "var(--apple-font-text)",
          fontSize: 12,
          fontWeight: 600,
          letterSpacing: "-0.12px",
          color: onDark ? "var(--apple-color-body-muted)" : "var(--apple-color-ink-muted-48)",
          textTransform: "none",
        }}
      >
        {label}
      </span>
      <span
        style={{
          fontFamily: "var(--apple-font-display)",
          fontSize: 28,
          fontWeight: 600,
          letterSpacing: 0,
          lineHeight: 1.14,
          color: onDark ? "var(--apple-color-body-on-dark)" : "var(--apple-color-ink)",
        }}
      >
        {value}
      </span>
      {hint ? (
        <span
          style={{
            fontFamily: "var(--apple-font-text)",
            fontSize: 12,
            letterSpacing: "-0.12px",
            color: onDark ? "var(--apple-color-body-muted)" : "var(--apple-color-ink-muted-48)",
            lineHeight: 1.4,
          }}
        >
          {hint}
        </span>
      ) : null}
    </div>
  );
}
