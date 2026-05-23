# 超星学习通助手（桌面 UI 版）

<p align="center">
  <a href="https://github.com/hor1zon777/chaoxing/stargazers">
    <img src="https://img.shields.io/github/stars/hor1zon777/chaoxing" alt="Github Stars" />
  </a>
  <a href="https://github.com/hor1zon777/chaoxing/network/members">
    <img src="https://img.shields.io/github/forks/hor1zon777/chaoxing" alt="Github Forks" />
  </a>
  <a href="https://github.com/hor1zon777/chaoxing/releases">
    <img src="https://img.shields.io/github/v/release/hor1zon777/chaoxing?display_name=tag&sort=semver" alt="version" />
  </a>
  <a href="https://github.com/hor1zon777/chaoxing/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/hor1zon777/chaoxing" alt="License" />
  </a>
</p>

基于 [Samueli924/chaoxing](https://github.com/Samueli924/chaoxing) 命令行版重写的**桌面图形化版本**，使用 **Tauri 2 + Rust + React + TypeScript** 构建。

> ⚠️ 仅供学习交流使用，请遵守原项目的 [GPL-3.0](LICENSE) 协议，禁止用于盈利或违法用途。

---

## 截图预览

> _截图占位：可在发布后补充_
>
> - 登录页 / 课程列表
> - 任务配置 / 进度实时面板
> - 设置（题库、AI、通知）

---

## 功能特性

### 核心能力

- **多账号管理**：账号加密存储、一键切换；支持账号密码登录与 Cookie 登录两种方式
- **课程列表 & 任务树**：自动拉取课程，按章节 / 任务点筛选，支持快捷选择与批量勾选
- **任务点执行**：覆盖视频、音频、文档、阅读、章节检测（作业）、直播全部任务类型
- **403 自动恢复**：视频任务遇到 403 自动刷新 dtoken 并重试，最多 2 次
- **字体反混淆**：内置字体 glyph 二进制解析 + 哈希索引，解决作业题目里字体加密导致的乱码

### 并发与控制

- **章节内任务并发**（`tasksPerChapter`，1–8）：同一章节内多个任务点并行执行
- **跨章节并发**（`chaptersPerCourse`，1–8）：同时处理多个章节，注意可能突破平台顺序解锁约束
- **多课程并发**（`jobs`，1–16）：同时学习多门课程
- **暂停 / 恢复 / 取消**：所有任务类型（含 Document/Read/Work）均能即时响应暂停信号
- **协作式取消**：基于 `Arc<AtomicBool>` 全链路传播，可在任意阶段安全终止

### 题库与 AI 答题

- **多题库支持**：言溪、Like、TikuAdapter，可同时配置多个 Token
- **AI 答题**：兼容任意 OpenAI 接口（OpenAI、DeepSeek、Kimi、智谱、Ollama 等），可设代理
- **SiliconFlow 直连**：内置硅基流动适配器
- **本地缓存**：答案缓存到本地 JSON，重复题目无需再查
- **提交策略**：可配置题库覆盖率阈值、是否自动提交
- **启动连接检查**：开始任务前自动校验 LLM 连接可用性

### 通知

- 支持 Server 酱、Qmsg、Bark、Telegram
- 任务全部完成或出错时主动推送

### 持久化与设置

- 配置自动加载 / 保存到系统 AppConfig 目录
- 兼容 Python 版 INI 配置一键导入
- 设置分 **通用 / 题库 / 通知** 三 Tab，dirty 状态实时提示未保存修改

---

## 快速开始

### 下载预编译版本（推荐）

前往 [Releases](https://github.com/hor1zon777/chaoxing/releases) 下载对应平台安装包：

| 平台 | 文件 |
|------|------|
| Windows | `*.exe` (NSIS 安装包) |
| macOS | `*.dmg` (Intel / Apple Silicon) |
| Linux | `*.AppImage` / `*.deb` |

双击安装即可，无需额外依赖。

### 首次使用

1. 启动后用**手机号 + 密码**或粘贴**浏览器 Cookies** 登录
2. 进入「课程」页面，选择需要学习的课程
3. 进入课程详情，配置要学习的章节与任务点
4. 进入「任务」页面，开始学习
5. （可选）打开「设置」配置题库、AI、通知

---

## 从源码构建

### 环境要求

- **Node.js** ≥ 18，[pnpm](https://pnpm.io/) ≥ 8
- **Rust** ≥ 1.75（含 cargo）
- 平台依赖：[Tauri 系统要求](https://v2.tauri.app/start/prerequisites/)

### 步骤

```bash
git clone https://github.com/hor1zon777/chaoxing.git
cd chaoxing/chaoxing-desktop

pnpm install

# 开发模式（热重载）
pnpm tauri dev

# 生产打包
pnpm tauri build
```

打包产物位于 `chaoxing-desktop/src-tauri/target/release/bundle/`。

---

## 项目结构

```
chaoxing/
├── chaoxing-desktop/           # 桌面应用（Tauri）
│   ├── src/                    # 前端 React + TypeScript
│   │   ├── routes/             # 页面（Login / Courses / Task / Settings）
│   │   ├── stores/             # Zustand 状态切片
│   │   ├── types/              # 类型定义
│   │   └── components/         # 通用组件
│   └── src-tauri/              # 后端 Rust
│       ├── src/
│       │   ├── api/            # 超星 API（client / video / work / live / ...）
│       │   ├── commands/       # Tauri IPC 命令
│       │   ├── task/           # 调度器与章节/任务执行
│       │   ├── tiku/           # 题库适配（言溪 / Like / AI / SiliconFlow）
│       │   ├── parser/         # HTML / JSON 解析
│       │   ├── font/           # 字体反混淆
│       │   ├── crypto/         # AES / 视频签名
│       │   └── notification/   # 通知（Server酱 / Qmsg / Bark / Telegram）
│       └── tauri.conf.json
├── api/, main.py, ...          # 原 Python CLI 版本（保留）
└── .github/workflows/          # CI / Release 工作流
```

---

## 配置说明

应用配置保存在系统 AppConfig 目录：

| 平台 | 路径 |
|------|------|
| Windows | `%APPDATA%\com.chaoxing.desktop\config.json` |
| macOS | `~/Library/Application Support/com.chaoxing.desktop/config.json` |
| Linux | `~/.config/com.chaoxing.desktop/config.json` |

### 主要字段

| 字段 | 说明 | 范围 |
|------|------|------|
| `speed` | 视频倍速 | 1.0 – 2.0 |
| `jobs` | 多课程并发 | 1 – 8 |
| `tasksPerChapter` | 章节内任务并发 | 1 – 8 |
| `chaptersPerCourse` | 跨章节并发 | 1 – 8 |
| `notopenAction` | 未开放章节处理 | `retry` / `continue` |
| `tikuProvider` | 题库类型 | `yanxi` / `like` / `ai` / `siliconflow` / `tikuadapter` |
| `tikuSubmit` | 是否自动提交 | bool |
| `tikuCoverRate` | 题库覆盖率阈值 | 0 – 1 |
| `tikuDelay` | 查询间隔（秒） | ≥ 0 |
| `aiMinInterval` | AI 请求最小间隔（秒） | ≥ 0 |
| `notificationProvider` | 通知类型 | `serverchan` / `qmsg` / `bark` / `telegram` |

> 完整字段见 [`src/types/config.ts`](chaoxing-desktop/src/types/config.ts)。

### 从 Python 版迁移

在「设置」页面点击「导入 INI」选择原 `config.ini`，会自动转换字段并保存。

---

## 与 Python CLI 版的差异

| 维度 | Python CLI | 桌面 UI |
|------|-----------|--------|
| 交互 | 终端 | 图形界面 + 实时进度面板 |
| 并发 | 多课程 + 章节内 | 多课程 + 章节内 + 跨章节，三级并发 |
| 状态管理 | 进程级 | Zustand 切片 + AppState |
| 暂停/取消 | Ctrl+C | UI 按钮，毫秒级响应所有任务类型 |
| 配置 | INI | JSON（兼容 INI 导入） |
| 答题缓存 | 进程内 | 本地 JSON 持久化 |
| 字体反混淆 | 远程 hash 表 | 本地 glyph 二进制解析（更快、可离线） |

---

## 测试与质量

- **133 个 Rust 单元测试**全部通过（`cargo test --lib`）
- **TypeScript strict**：`pnpm tsc --noEmit` 零错误
- **零警告编译**：`cargo check`、`cargo clippy`

---

## 致谢

- 原项目 [Samueli924/chaoxing](https://github.com/Samueli924/chaoxing) 及所有贡献者
- 题库适配参考 [sz134055](https://github.com/sz134055) 的工作
- UI 设计参考 Apple Design Resources

---

## 免责声明

- 本项目遵循 [GPL-3.0 License](LICENSE)，允许开源/免费使用、引用、修改、衍生代码的开源/免费使用；**不允许**将修改或衍生代码作为闭源商业软件发布或销售，禁止以本代码为基础盈利
- 本代码**仅供学习讨论**
- 使用者使用本代码进行的任何违法行为与作者无关
- 使用本工具可能违反学习通的服务条款，由此产生的账号风险由使用者自行承担

---

## License

[GPL-3.0](LICENSE) © 原作者 [Samueli924](https://github.com/Samueli924) 及本项目贡献者
