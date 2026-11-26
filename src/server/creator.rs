//! Authenticated upstream calls for the Creator Dashboard SSR page.
//!
//! Every call is same-origin (`https://vionsky.com`), carries a fixed path,
//! and forwards only the request's session cookie / bearer token. No user
//! input is ever interpolated into a host, so this cannot become an open proxy.

use crate::creator::{
    CreatorGame, CreatorGamesResult, DigestInfo, FeedbackItem, FeedbackResult, MutationResult,
};
use crate::server::{AppError, AppResult, AppState};
use serde_json::{json, Value};

const API_ORIGIN: &str = "https://vionsky.com";
const MAX_UPSTREAM_BYTES: usize = 262_144;

struct UpstreamResponse {
    status: u16,
    payload: Value,
}

fn error_message(payload: &Value) -> Option<String> {
    payload
        .get("error")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .map(str::to_owned)
}

fn apply_headers(
    init: &mut worker::RequestInit,
    state: &AppState,
    has_body: bool,
) -> AppResult<()> {
    init.headers
        .set("Accept", "application/json")
        .map_err(|_| AppError::Internal)?;
    init.headers
        .set("Cache-Control", "no-store")
        .map_err(|_| AppError::Internal)?;
    if has_body {
        init.headers
            .set("Content-Type", "application/json")
            .map_err(|_| AppError::Internal)?;
    }
    if let Some(cookie) = state.cookie.as_ref() {
        init.headers
            .set("Cookie", cookie)
            .map_err(|_| AppError::Internal)?;
    }
    if let Some(token) = state.auth_header.as_ref() {
        init.headers
            .set("Authorization", token)
            .map_err(|_| AppError::Internal)?;
    }
    Ok(())
}

async fn upstream(
    state: &AppState,
    method: worker::Method,
    path: &str,
    body: Option<Value>,
) -> AppResult<UpstreamResponse> {
    let url = worker::Url::parse(&format!("{API_ORIGIN}{path}")).map_err(|_| AppError::Internal)?;
    let mut init = worker::RequestInit::new();
    init.with_method(method);
    init.with_cache(worker::CacheMode::NoStore);
    if let Some(ref value) = body {
        init.with_body(Some(wasm_bindgen::JsValue::from_str(&value.to_string())));
    }
    apply_headers(&mut init, state, body.is_some())?;

    let request =
        worker::Request::new_with_init(url.as_str(), &init).map_err(|_| AppError::Internal)?;
    let mut response = worker::Fetch::Request(request)
        .send()
        .await
        .map_err(|_| AppError::Internal)?;

    if let Some(length) = response
        .headers()
        .get("content-length")
        .ok()
        .flatten()
        .and_then(|value| value.parse::<usize>().ok())
    {
        if length > MAX_UPSTREAM_BYTES {
            return Err(AppError::Internal);
        }
    }

    let status = response.status_code();
    let text = response.text().await.map_err(|_| AppError::Internal)?;
    if text.len() > MAX_UPSTREAM_BYTES {
        return Err(AppError::Internal);
    }
    let payload = serde_json::from_str::<Value>(&text).unwrap_or(Value::Null);
    Ok(UpstreamResponse { status, payload })
}

fn failure(response: &UpstreamResponse, signed_out_message: &str) -> AppResult<MutationResult> {
    let message = error_message(&response.payload)
        .unwrap_or_else(|| format!("Yêu cầu không thành công ({})", response.status));
    if response.status == 401 {
        return Err(AppError::client(signed_out_message));
    }
    Ok(MutationResult {
        ok: false,
        error: Some(message),
        ..MutationResult::default()
    })
}

pub async fn fetch_creator_games(state: &AppState) -> AppResult<CreatorGamesResult> {
    let response = upstream(state, worker::Method::Get, "/api/creator/games/", None).await?;
    if response.status == 401 {
        return Ok(CreatorGamesResult {
            signed_out: true,
            ..CreatorGamesResult::default()
        });
    }
    if !(200..300).contains(&response.status) {
        return Ok(CreatorGamesResult {
            error: Some(
                error_message(&response.payload)
                    .unwrap_or_else(|| "Không tải được sản phẩm của bạn".to_string()),
            ),
            ..CreatorGamesResult::default()
        });
    }

    let games = response
        .payload
        .get("games")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| serde_json::from_value::<CreatorGame>(row.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    Ok(CreatorGamesResult {
        games,
        ..CreatorGamesResult::default()
    })
}

fn valid_http_url(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("http://") || trimmed.starts_with("https://")
}

pub async fn patch_creator_game(
    state: &AppState,
    game_id: i64,
    description: String,
    play_url: String,
) -> AppResult<MutationResult> {
    if game_id <= 0 {
        return Err(AppError::client("game_id không hợp lệ"));
    }
    let description = description.trim().to_owned();
    if description.chars().count() > 5000 {
        return Err(AppError::client("Mô tả không được vượt quá 5000 ký tự"));
    }
    let play_url = play_url.trim().to_owned();
    if !valid_http_url(&play_url) || worker::Url::parse(&play_url).is_err() {
        return Err(AppError::client("Link chơi phải là địa chỉ http(s) hợp lệ"));
    }

    let body = json!({
        "game_id": game_id,
        "description": description,
        "play_url": play_url,
    });
    let response = upstream(
        state,
        worker::Method::Patch,
        "/api/creator/games/",
        Some(body),
    )
    .await?;
    if !(200..300).contains(&response.status) {
        return failure(
            &response,
            "Phiên đăng nhập đã hết hạn. Vui lòng đăng nhập lại.",
        );
    }
    let re_review = response
        .payload
        .get("re_review")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    Ok(MutationResult {
        ok: true,
        re_review,
        error: None,
    })
}

pub async fn post_re_audit(state: &AppState, game_id: i64) -> AppResult<MutationResult> {
    if game_id <= 0 {
        return Err(AppError::client("id không hợp lệ"));
    }
    let path = format!("/api/creator/games/{game_id}/re-audit/");
    let response = upstream(state, worker::Method::Post, &path, None).await?;
    if !(200..300).contains(&response.status) {
        return failure(
            &response,
            "Phiên đăng nhập đã hết hạn. Vui lòng đăng nhập lại.",
        );
    }
    Ok(MutationResult {
        ok: true,
        ..MutationResult::default()
    })
}

pub async fn fetch_creator_feedback(state: &AppState) -> AppResult<FeedbackResult> {
    let response = upstream(state, worker::Method::Get, "/api/creator/feedback/", None).await?;
    if response.status == 401 {
        return Ok(FeedbackResult {
            signed_out: true,
            ..FeedbackResult::default()
        });
    }
    if !(200..300).contains(&response.status) {
        return Ok(FeedbackResult {
            error: Some(error_message(&response.payload).unwrap_or_default()),
            ..FeedbackResult::default()
        });
    }

    let items = response
        .payload
        .get("items")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| serde_json::from_value::<FeedbackItem>(row.clone()).ok())
                .collect()
        })
        .unwrap_or_default();
    let digest = response
        .payload
        .get("digest")
        .filter(|value| value.is_object())
        .and_then(|value| serde_json::from_value::<DigestInfo>(value.clone()).ok());

    Ok(FeedbackResult {
        items,
        digest,
        ..FeedbackResult::default()
    })
}

pub async fn fetch_feedback_settings(state: &AppState) -> AppResult<bool> {
    let response = upstream(
        state,
        worker::Method::Get,
        "/api/creator/feedback/settings/",
        None,
    )
    .await?;
    if response.status == 401 {
        return Ok(true);
    }
    if !(200..300).contains(&response.status) {
        return Err(AppError::client(
            error_message(&response.payload)
                .unwrap_or_else(|| "Không tải được cài đặt".to_string()),
        ));
    }
    Ok(response
        .payload
        .get("notify_feedback")
        .and_then(Value::as_bool)
        .unwrap_or(true))
}

pub async fn patch_feedback_settings(
    state: &AppState,
    notify_feedback: bool,
) -> AppResult<MutationResult> {
    let body = json!({ "notify_feedback": notify_feedback });
    let response = upstream(
        state,
        worker::Method::Patch,
        "/api/creator/feedback/settings/",
        Some(body),
    )
    .await?;
    if !(200..300).contains(&response.status) {
        return failure(&response, "Phiên đăng nhập đã hết hạn.");
    }
    Ok(MutationResult {
        ok: true,
        ..MutationResult::default()
    })
}

pub async fn post_handle_feedback(
    state: &AppState,
    kind: String,
    id: i64,
) -> AppResult<MutationResult> {
    if id <= 0 || !matches!(kind.as_str(), "review" | "comment") {
        return Err(AppError::client("Góp ý không hợp lệ"));
    }
    let body = json!({ "kind": kind, "id": id });
    let response = upstream(
        state,
        worker::Method::Post,
        "/api/creator/feedback/handle/",
        Some(body),
    )
    .await?;
    if !(200..300).contains(&response.status) {
        return failure(&response, "Phiên đăng nhập đã hết hạn.");
    }
    Ok(MutationResult {
        ok: true,
        ..MutationResult::default()
    })
}
