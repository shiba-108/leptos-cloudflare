//! SSR-side bridge to the canonical same-origin `/api/posts` contract.
//! Read-only list plus the authenticated 'Đăng bài' mutation. No token is
//! ever logged or persisted; the incoming cookie is forwarded only upstream.
use super::{AppError, AppResult};
use crate::community_index::{
    CommunityIndexResult, CommunityIndexStatus, CreatePostInput, CreatePostResult, MAX_PAGE_SIZE,
};
use serde::Deserialize;

const MAX_RESPONSE_BYTES: usize = 262_144;

#[derive(Debug, Deserialize)]
struct Envelope<T> {
    success: bool,
    #[serde(default)]
    data: Option<T>,
    #[serde(default)]
    error: Option<String>,
}

fn upstream(path: &str) -> AppResult<worker::Url> {
    // Same allowlist rationale as `server/search.rs::search_api_url`:
    // restrict the SSR fetch to the VionSky zone so a forged Host cannot turn
    // this into a generic server-side proxy.
    worker::Url::parse(&format!("https://vionsky.com{path}")).map_err(|_| AppError::Internal)
}

fn init() -> AppResult<worker::RequestInit> {
    let mut init = worker::RequestInit::new();
    init.with_cache(worker::CacheMode::NoStore);
    init.headers
        .set("Accept", "application/json")
        .map_err(|_| AppError::Internal)?;
    init.headers
        .set("Cache-Control", "no-store")
        .map_err(|_| AppError::Internal)?;
    // fetchWithAuth equivalent: forward the viewer cookie so `is_upvoted`
    // and `feed=following` are viewer-correct. Absent cookie => public list.
    if let Ok(state) = crate::server::AppState::from_context() {
        if let Some(cookie) = state.cookie.clone() {
            init.headers
                .set("Cookie", cookie.as_ref())
                .map_err(|_| AppError::Internal)?;
        }
    }
    Ok(init)
}

async fn body_of(mut response: worker::Response) -> AppResult<String> {
    if response
        .headers()
        .get("content-length")
        .map_err(|_| AppError::Internal)?
        .and_then(|v| v.parse::<usize>().ok())
        .is_some_and(|n| n > MAX_RESPONSE_BYTES)
    {
        return Err(AppError::Internal);
    }
    let body = response.text().await.map_err(|_| AppError::Internal)?;
    if body.len() > MAX_RESPONSE_BYTES {
        return Err(AppError::Internal);
    }
    Ok(body)
}

pub async fn fetch_community_index(
    page: u32,
    limit: u32,
    category: Option<String>,
    feed: String,
) -> AppResult<CommunityIndexResult> {
    let page = page.max(1);
    let limit = limit.clamp(1, MAX_PAGE_SIZE);
    let mut url = upstream("/api/posts")?;
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("page", &page.to_string());
        q.append_pair("limit", &limit.to_string());
        if let Some(c) = category
            .as_deref()
            .filter(|c| crate::community_index::CATEGORIES.contains(c))
        {
            q.append_pair("category", c);
        }
        if feed == "following" {
            q.append_pair("feed", "following");
        }
    }
    let request =
        worker::Request::new_with_init(url.as_str(), &init()?).map_err(|_| AppError::Internal)?;
    let mut response = worker::Fetch::Request(request)
        .send()
        .await
        .map_err(|_| AppError::Internal)?;
    let status = response.status_code();
    let body = body_of(response).await?;
    if status == 401 {
        return Ok(CommunityIndexResult {
            status: CommunityIndexStatus::AuthRequired,
            ..Default::default()
        });
    }
    if status != 200 {
        return Ok(CommunityIndexResult {
            status: CommunityIndexStatus::Unavailable,
            ..Default::default()
        });
    }
    let envelope: Envelope<CommunityIndexResult> =
        serde_json::from_str(&body).map_err(|_| AppError::Internal)?;
    if !envelope.success {
        return Ok(CommunityIndexResult {
            status: CommunityIndexStatus::Unavailable,
            ..Default::default()
        });
    }
    let mut data = envelope.data.ok_or(AppError::Internal)?;
    data.status = CommunityIndexStatus::Loaded;
    data.page = data.page.max(1);
    data.limit = data.limit.clamp(1, MAX_PAGE_SIZE);
    Ok(data)
}

pub async fn create_community_post(input: CreatePostInput) -> AppResult<CreatePostResult> {
    let url = upstream("/api/posts")?;
    let mut request_init = init()?;
    request_init.with_method(worker::Method::Post);
    request_init
        .headers
        .set("Content-Type", "application/json")
        .map_err(|_| AppError::Internal)?;
    let payload = serde_json::to_string(&input).map_err(|_| AppError::Internal)?;
    request_init.with_body(Some(worker::wasm_bindgen::JsValue::from_str(&payload)));
    let request = worker::Request::new_with_init(url.as_str(), &request_init)
        .map_err(|_| AppError::Internal)?;
    let mut response = worker::Fetch::Request(request)
        .send()
        .await
        .map_err(|_| AppError::Internal)?;
    let status = response.status_code();
    let body = body_of(response).await?;
    let envelope: Envelope<serde_json::Value> =
        serde_json::from_str(&body).map_err(|_| AppError::Internal)?;
    let mut result = CreatePostResult {
        success: envelope.success && (status == 200 || status == 201),
        id: envelope
            .data
            .as_ref()
            .and_then(|d| d.get("id"))
            .and_then(|v| v.as_i64()),
        error: envelope.error,
        status: if status == 401 {
            CommunityIndexStatus::AuthRequired
        } else {
            CommunityIndexStatus::Loaded
        },
    };
    if status == 401 {
        result.error = Some("Vui lòng đăng nhập để đăng bài".to_string());
    }
    Ok(result)
}
