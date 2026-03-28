use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Client;
use std::sync::Arc;

use url::Url;

use cookie_store::CookieStore;
use reqwest_cookie_store::CookieStoreMutex;

use crate::models::account::StoredCookie;
use crate::utils::rate_limiter::RateLimiter;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Safari/537.36";

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
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("创建 HTTP 客户端失败");

        Self {
            client,
            cookie_store,
            rate_limiter: RateLimiter::new(0.5),
            video_rate_limiter: RateLimiter::new(2.0),
        }
    }

    /// 从 cookie store 获取指定 cookie 的值
    pub fn get_cookie(&self, name: &str, domain: &str) -> Option<String> {
        let store = self.cookie_store.lock().unwrap();
        for cookie in store.iter_unexpired() {
            if cookie.name() == name && cookie.domain().map_or(false, |d| d.contains(domain)) {
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
        let store = self.cookie_store.lock().unwrap();
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
        let mut store = self.cookie_store.lock().unwrap();
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
