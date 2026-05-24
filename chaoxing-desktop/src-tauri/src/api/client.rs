use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::redirect::Policy;
use reqwest::Client;
use std::sync::Arc;

use url::Url;

use cookie_store::CookieStore;
use reqwest_cookie_store::CookieStoreMutex;

use crate::models::account::StoredCookie;
use crate::utils::rate_limiter::RateLimiter;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36";

#[derive(Clone)]
pub struct HttpClient {
    pub client: Client,
    pub cookie_store: Arc<CookieStoreMutex>,
    pub rate_limiter: RateLimiter,
    pub video_rate_limiter: RateLimiter,
}

impl HttpClient {
    pub fn new() -> Self {
        let cookie_store = CookieStore::default();
        let cookie_store = Arc::new(CookieStoreMutex::new(cookie_store));

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(UA));
        headers.insert(
            "sec-ch-ua",
            HeaderValue::from_static(
                "\"Chromium\";v=\"118\", \"Google Chrome\";v=\"118\", \"Not=A?Brand\";v=\"99\"",
            ),
        );
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert(
            "sec-ch-ua-platform",
            HeaderValue::from_static("\"Windows\""),
        );

        let client = Client::builder()
            .cookie_provider(cookie_store.clone())
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|e| {
                // 命令路径上若 panic 会让整个 Tauri runtime 崩溃；
                // 回退到无定制 headers 的最小客户端，保证应用不死
                tracing::error!("创建 HTTP 客户端失败: {}，回退为默认客户端", e);
                Client::builder()
                    .cookie_provider(cookie_store.clone())
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .unwrap_or_default()
            });

        Self {
            client,
            cookie_store,
            rate_limiter: RateLimiter::new(0.5),
            video_rate_limiter: RateLimiter::new(2.0),
        }
    }

    /// 构建不自动跟随重定向的客户端（用于 work API 等需要手动处理重定向的场景）
    pub fn client_builder_no_redirect(&self) -> Client {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(UA));
        headers.insert(
            "sec-ch-ua",
            HeaderValue::from_static(
                "\"Chromium\";v=\"118\", \"Google Chrome\";v=\"118\", \"Not=A?Brand\";v=\"99\"",
            ),
        );
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert(
            "sec-ch-ua-platform",
            HeaderValue::from_static("\"Windows\""),
        );

        Client::builder()
            .cookie_provider(self.cookie_store.clone())
            .default_headers(headers)
            .redirect(Policy::none())
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|e| {
                tracing::error!("创建无重定向 HTTP 客户端失败: {}，回退为默认客户端", e);
                Client::builder()
                    .cookie_provider(self.cookie_store.clone())
                    .redirect(Policy::none())
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .unwrap_or_default()
            })
    }

    /// 从 cookie store 获取指定 cookie 的值
    ///
    /// domain 匹配规则：cookie 的 domain 必须等于 `domain`、
    /// 等于 `.domain`，或以 `.domain` 结尾（subdomain match）。
    /// 用 `contains` 会把 "evil.chaoxing.com.attacker.io" 这种子串误判，故收紧。
    pub fn get_cookie(&self, name: &str, domain: &str) -> Option<String> {
        let store = match self.cookie_store.lock() {
            Ok(s) => s,
            Err(poisoned) => {
                tracing::warn!("CookieStore mutex 已中毒，尝试恢复");
                poisoned.into_inner()
            }
        };
        let target = domain.trim_start_matches('.');
        for cookie in store.iter_unexpired() {
            if cookie.name() != name {
                continue;
            }
            let Some(d) = cookie.domain() else {
                continue;
            };
            let d = d.trim_start_matches('.');
            if d == target || d.ends_with(&format!(".{}", target)) {
                return Some(cookie.value().to_string());
            }
        }
        None
    }

    /// 获取 uid
    pub fn get_uid(&self) -> Option<String> {
        self.get_cookie("_uid", "chaoxing.com")
            .or_else(|| self.get_cookie("UID", "chaoxing.com"))
    }

    /// 获取 fid
    pub fn get_fid(&self) -> Option<String> {
        self.get_cookie("fid", "chaoxing.com")
    }

    pub fn export_cookies(&self) -> Vec<StoredCookie> {
        let store = match self.cookie_store.lock() {
            Ok(s) => s,
            Err(poisoned) => {
                tracing::warn!("CookieStore mutex 已中毒，尝试恢复");
                poisoned.into_inner()
            }
        };
        store
            .iter_unexpired()
            .map(|cookie| StoredCookie {
                name: cookie.name().to_string(),
                value: cookie.value().to_string(),
                domain: cookie.domain().unwrap_or(".chaoxing.com").to_string(),
                path: cookie.path().unwrap_or("/").to_string(),
            })
            .collect()
    }

    pub fn import_cookies(&self, cookies: &[StoredCookie]) -> Result<(), String> {
        let mut store = match self.cookie_store.lock() {
            Ok(s) => s,
            Err(poisoned) => {
                tracing::warn!("CookieStore mutex 已中毒，尝试恢复");
                poisoned.into_inner()
            }
        };
        for cookie in cookies {
            let normalized_domain = if cookie.domain.is_empty() {
                ".chaoxing.com".to_string()
            } else {
                cookie.domain.clone()
            };
            let normalized_path = if cookie.path.is_empty() {
                "/".to_string()
            } else {
                cookie.path.clone()
            };
            let host = normalized_domain.trim_start_matches('.');
            let url: Url = format!("https://{}{}", host, normalized_path)
                .parse()
                .map_err(|e| format!("URL 解析失败: {}", e))?;
            let set_cookie = format!(
                "{}={}; Domain={}; Path={}",
                cookie.name, cookie.value, normalized_domain, normalized_path
            );
            if let Ok(parsed) = cookie_store::RawCookie::parse(set_cookie) {
                let _ = store.insert_raw(&parsed, &url);
            }
        }
        Ok(())
    }
}
