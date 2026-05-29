import { useCallback } from "react";
import { open as openExternal } from "@tauri-apps/plugin-shell";
import { message } from "antd";
import { Card, Eyebrow, Headline, PillButton, Subtitle, Tag } from "../components/ui/appleUI";

const REPO_URL = "https://github.com/hor1zon777/chaoxing";
const UPSTREAM_REPO_URL = "https://github.com/Samueli924/chaoxing";
const LICENSE_URL = "https://www.gnu.org/licenses/gpl-3.0.html";
const APP_VERSION = "v0.2.0";

/** 关于页：作者、仓库、协议、免责声明 */
export function AboutPage() {
  const [msgApi, contextHolder] = message.useMessage();

  const handleOpen = useCallback(
    async (url: string) => {
      try {
        await openExternal(url);
      } catch {
        try {
          await navigator.clipboard.writeText(url);
          msgApi.info("无法打开浏览器，链接已复制到剪贴板");
        } catch {
          msgApi.error("无法打开链接");
        }
      }
    },
    [msgApi],
  );

  return (
    <div style={{ background: "var(--apple-color-canvas)" }}>
      {contextHolder}

      {/* Hero */}
      <section style={{ padding: "48px 22px 24px" }}>
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          <Eyebrow>关于</Eyebrow>
          <Headline level="md" style={{ marginTop: 8 }}>
            超星学习通助手
          </Headline>
          <Subtitle style={{ marginTop: 10 }}>
            基于 Tauri + Rust + React 构建的桌面端学习辅助工具。
          </Subtitle>
          <div style={{ display: "flex", gap: 8, marginTop: 16, flexWrap: "wrap" }}>
            <Tag tone="neutral">版本 {APP_VERSION}</Tag>
            <Tag tone="neutral">GPL-3.0</Tag>
          </div>
        </div>
      </section>

      {/* 信息卡片 */}
      <section style={{ padding: "0 22px 24px" }}>
        <div
          style={{
            maxWidth: 1024,
            margin: "0 auto",
            display: "grid",
            gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))",
            gap: 16,
          }}
        >
          <Card title="作者" subtitle="hor1zon777（本仓库） · Samueli924（原作者）">
            <p
              style={{
                margin: 0,
                fontFamily: "var(--apple-font-text)",
                fontSize: 14,
                lineHeight: 1.5,
                color: "var(--apple-color-ink-muted-80)",
              }}
            >
              本项目基于 Samueli924/chaoxing 的 Python CLI 实现，重写为 Tauri 桌面端。
              欢迎在 GitHub 提交 Issue 或 Pull Request 参与改进。
            </p>
          </Card>

          <Card title="本仓库" subtitle="桌面端 (Tauri + Rust + React)">
            <p
              style={{
                margin: "0 0 12px",
                fontFamily: "var(--apple-font-text)",
                fontSize: 13,
                lineHeight: 1.5,
                color: "var(--apple-color-ink-muted-80)",
                wordBreak: "break-all",
              }}
            >
              {REPO_URL}
            </p>
            <PillButton variant="ghost" onClick={() => void handleOpen(REPO_URL)}>
              在浏览器中打开
            </PillButton>
          </Card>

          <Card title="原作者仓库" subtitle="Samueli924/chaoxing">
            <p
              style={{
                margin: "0 0 12px",
                fontFamily: "var(--apple-font-text)",
                fontSize: 13,
                lineHeight: 1.5,
                color: "var(--apple-color-ink-muted-80)",
                wordBreak: "break-all",
              }}
            >
              {UPSTREAM_REPO_URL}
            </p>
            <PillButton variant="ghost" onClick={() => void handleOpen(UPSTREAM_REPO_URL)}>
              在浏览器中打开
            </PillButton>
          </Card>

          <Card title="开源协议" subtitle="GNU GPL v3.0">
            <p
              style={{
                margin: "0 0 12px",
                fontFamily: "var(--apple-font-text)",
                fontSize: 14,
                lineHeight: 1.5,
                color: "var(--apple-color-ink-muted-80)",
              }}
            >
              本软件以 GPL-3.0 协议开源。任何分发、修改或衍生作品均须遵守该协议条款。
            </p>
            <PillButton variant="ghost" onClick={() => void handleOpen(LICENSE_URL)}>
              查看协议全文
            </PillButton>
          </Card>
        </div>
      </section>

      {/* 免责声明 */}
      <section style={{ padding: "0 22px 64px" }}>
        <div style={{ maxWidth: 1024, margin: "0 auto" }}>
          <Card title="免责声明">
            <ul
              style={{
                margin: 0,
                paddingLeft: 20,
                fontFamily: "var(--apple-font-text)",
                fontSize: 14,
                lineHeight: 1.7,
                color: "var(--apple-color-ink-muted-80)",
              }}
            >
              <li>
                本项目仅供学习、研究与技术交流使用，严禁用于任何商业用途或违反所在地区法律法规、
                超星平台用户协议的行为。
              </li>
              <li>
                使用本工具产生的一切后果（包括但不限于账号被封禁、学习记录异常、数据丢失等），
                均由使用者自行承担，作者不对此承担任何责任。
              </li>
              <li>
                本工具不收集、不上传任何用户隐私数据；登录凭据仅保存在用户本地。请妥善保管自己的
                账号信息。
              </li>
              <li>
                若本项目存在侵犯他人合法权益的内容，请通过 GitHub Issue 联系作者，我们会在确认后
                及时处理。
              </li>
              <li>
                下载、安装并使用本软件即视为已阅读并同意以上条款；若不接受请立即停止使用并删除本
                软件。
              </li>
            </ul>
          </Card>
        </div>
      </section>
    </div>
  );
}
