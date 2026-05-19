/**
 * Apple 设计 token
 * 来源: apple/DESIGN.md
 *
 * 设计原则:
 * - 单一 Action Blue (#0066cc) 为唯一交互色
 * - 无渐变 / 无装饰阴影
 * - 17px 正文 · 600 重量标题 · 负字距
 * - edge-to-edge 色块交替作为分隔
 * - pill CTA + utility card 是两种主要容器语法
 */

export const appleColors = {
  primary: "#0066cc",
  primaryFocus: "#0071e3",
  primaryOnDark: "#2997ff",
  ink: "#1d1d1f",
  body: "#1d1d1f",
  bodyOnDark: "#ffffff",
  bodyMuted: "#cccccc",
  inkMuted80: "#333333",
  inkMuted48: "#7a7a7a",
  dividerSoft: "#f0f0f0",
  hairline: "#e0e0e0",
  canvas: "#ffffff",
  canvasParchment: "#f5f5f7",
  surfacePearl: "#fafafc",
  surfaceTile1: "#272729",
  surfaceTile2: "#2a2a2c",
  surfaceTile3: "#252527",
  surfaceBlack: "#000000",
  surfaceChipTranslucent: "rgba(210, 210, 215, 0.64)",
  onPrimary: "#ffffff",
  onDark: "#ffffff",
  success: "#1e7e34",
  warning: "#a85d00",
  danger: "#c8261d",
} as const;

export const appleRadii = {
  none: 0,
  xs: 5,
  sm: 8,
  md: 11,
  lg: 18,
  pill: 9999,
  full: 9999,
} as const;

export const appleSpacing = {
  xxs: 4,
  xs: 8,
  sm: 12,
  md: 17,
  lg: 24,
  xl: 32,
  xxl: 48,
  section: 80,
} as const;

/** 唯一产品阴影 - 仅用于产品图 */
export const appleProductShadow = "rgba(0, 0, 0, 0.22) 3px 5px 30px 0";

/** 1px hairline 边界 (实际为 0.04 alpha) */
export const appleHairline = "1px solid #e0e0e0";
export const appleHairlineSoft = "1px solid #f0f0f0";

export const appleFontStack = {
  display: '"SF Pro Display", system-ui, -apple-system, "PingFang SC", "Microsoft YaHei", sans-serif',
  text: '"SF Pro Text", system-ui, -apple-system, "PingFang SC", "Microsoft YaHei", sans-serif',
} as const;

export const appleTypography = {
  heroDisplay: {
    fontFamily: appleFontStack.display,
    fontSize: 56,
    fontWeight: 600,
    lineHeight: 1.07,
    letterSpacing: "-0.28px",
  },
  displayLg: {
    fontFamily: appleFontStack.display,
    fontSize: 40,
    fontWeight: 600,
    lineHeight: 1.1,
    letterSpacing: 0,
  },
  displayMd: {
    fontFamily: appleFontStack.text,
    fontSize: 34,
    fontWeight: 600,
    lineHeight: 1.47,
    letterSpacing: "-0.374px",
  },
  lead: {
    fontFamily: appleFontStack.display,
    fontSize: 28,
    fontWeight: 400,
    lineHeight: 1.14,
    letterSpacing: "0.196px",
  },
  leadAiry: {
    fontFamily: appleFontStack.text,
    fontSize: 24,
    fontWeight: 300,
    lineHeight: 1.5,
    letterSpacing: 0,
  },
  tagline: {
    fontFamily: appleFontStack.display,
    fontSize: 21,
    fontWeight: 600,
    lineHeight: 1.19,
    letterSpacing: "0.231px",
  },
  bodyStrong: {
    fontFamily: appleFontStack.text,
    fontSize: 17,
    fontWeight: 600,
    lineHeight: 1.24,
    letterSpacing: "-0.374px",
  },
  body: {
    fontFamily: appleFontStack.text,
    fontSize: 17,
    fontWeight: 400,
    lineHeight: 1.47,
    letterSpacing: "-0.374px",
  },
  caption: {
    fontFamily: appleFontStack.text,
    fontSize: 14,
    fontWeight: 400,
    lineHeight: 1.43,
    letterSpacing: "-0.224px",
  },
  captionStrong: {
    fontFamily: appleFontStack.text,
    fontSize: 14,
    fontWeight: 600,
    lineHeight: 1.29,
    letterSpacing: "-0.224px",
  },
  buttonLarge: {
    fontFamily: appleFontStack.text,
    fontSize: 18,
    fontWeight: 300,
    lineHeight: 1,
    letterSpacing: 0,
  },
  buttonUtility: {
    fontFamily: appleFontStack.text,
    fontSize: 14,
    fontWeight: 400,
    lineHeight: 1.29,
    letterSpacing: "-0.224px",
  },
  finePrint: {
    fontFamily: appleFontStack.text,
    fontSize: 12,
    fontWeight: 400,
    lineHeight: 1,
    letterSpacing: "-0.12px",
  },
  navLink: {
    fontFamily: appleFontStack.text,
    fontSize: 12,
    fontWeight: 400,
    lineHeight: 1,
    letterSpacing: "-0.12px",
  },
} as const;

/** 全局导航高度 */
export const appleGlobalNavHeight = 44;
/** 子导航高度 */
export const appleSubNavHeight = 52;
