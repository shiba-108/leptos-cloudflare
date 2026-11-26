// ============================================================================
//! Server-side bridge to the deployed Next.js `/api/visky/chat/` route.
//! Mirrors the host-allowlisted fetch in `src/server/search.rs`; upstream
//! ZeroClaw WebSocket, fallback models and quotas stay authoritative there.

use axum::http::Uri;
use wasm_bindgen::JsValue;

use crate::visky::{
    build_history, collect_stream, should_reuse_session, validate_chat_input, ChatTurn,
    ViskyChatReply, MAX_CHARS_PER_TURN, MAX_HISTORY_ITEMS, MAX_HISTORY_ROUNDS,
    MAX_SESSION_ID_CHARS,
};

use super::{AppError, AppResult};

const MAX_SSE_BYTES: usize = 2_000_000;

#[derive(serde::Serialize)]
struct OutboundBody {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    history: Vec<ChatTurn>,
}

pub async fn visky_chat(
    message: String,
    session_id: Option<String>,
    history_json: String,
) -> AppResult<ViskyChatReply> {
    validate_chat_input(&message).map_err(AppError::client)?;
    // Encoded as a JSON string so the server-fn PostUrl codec never has to
    // serialize a sequence of structs (which it cannot represent).
    let history: Vec<ChatTurn> = serde_json::from_str(&history_json).unwrap_or_default();

    // Client history is untrusted (`chat/route.ts:62-87`): allowlist roles,
    // cap item count and truncate each turn without breaking emoji graphemes.
    let sanitized = sanitize_history(&history);
    let kept = prune_rounds(&sanitized);
    let sid = session_id
        .map(|id| id.trim().to_owned())
        .filter(|id| !id.is_empty() && id.chars().count() <= MAX_SESSION_ID_CHARS);
    let reused = should_reuse_session(sid.as_deref(), sanitized.len(), kept);

    let body = OutboundBody {
        message: message.trim().to_owned(),
        session_id: sid,
        history: sanitized,
    };
    let json = serde_json::to_string(&body).map_err(|_| AppError::Internal)?;

    let uri = leptos_axum::extract::<Uri>()
        .await
        .map_err(|_| AppError::Internal)?;
    let Some(url) = chat_api_url(&uri) else {
        return Err(AppError::client("Tính năng chat chưa sẵn sàng."));
    };

    let mut init = worker::RequestInit::new();
    init.with_method(worker::Method::Post);
    init.with_body(Some(JsValue::from_str(&json)));
    init.with_cache(worker::CacheMode::NoStore);
    init.headers
        .set("Content-Type", "application/json")
        .map_err(|_| AppError::Internal)?;
    init.headers
        .set("Accept", "text/event-stream")
        .map_err(|_| AppError::Internal)?;
    let request =
        worker::Request::new_with_init(url.as_str(), &init).map_err(|_| AppError::Internal)?;
    let mut response = worker::Fetch::Request(request)
        .send()
        .await
        .map_err(|_| AppError::Internal)?;
    let status = response.status_code();

    if status == 429 {
        return Ok(ViskyChatReply {
            text: String::new(),
            session_id: None,
            error: Some("rate_limited".to_owned()),
        });
    }
    if !(200..300).contains(&status) {
        return Err(AppError::client("Hiện chưa trò chuyện được. Thử lại sau."));
    }
    if response
        .headers()
        .get("content-length")
        .map_err(|_| AppError::Internal)?
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|length| length > MAX_SSE_BYTES)
    {
        return Err(AppError::client("Phản hồi quá lớn."));
    }
    let body = response.text().await.map_err(|_| AppError::Internal)?;
    if body.len() > MAX_SSE_BYTES {
        return Err(AppError::client("Phản hồi quá lớn."));
    }
    Ok(collect_stream(&body))
}

fn sanitize_history(history: &[ChatTurn]) -> Vec<ChatTurn> {
    history
        .iter()
        .take(MAX_HISTORY_ITEMS)
        .filter(|turn| turn.role == "user" || turn.role == "assistant")
        .map(|turn| {
            let content: String = turn.content.chars().take(MAX_CHARS_PER_TURN).collect();
            ChatTurn {
                role: turn.role.clone(),
                content: content.trim().to_owned(),
            }
        })
        .collect()
}

fn prune_rounds(history: &[ChatTurn]) -> usize {
    crate::visky::prune_context(history, MAX_HISTORY_ROUNDS).len()
}

/// Same-origin allowlist: only the public VionSky zone may be proxied.
fn chat_api_url(uri: &Uri) -> Option<worker::Url> {
    if uri.scheme_str() != Some("https") {
        return None;
    }
    let authority = uri.authority()?;
    let host = authority.host().trim_end_matches('.');
    let normalized = host.to_ascii_lowercase();
    if normalized != "vionsky.com" && !normalized.ends_with(".vionsky.com") {
        return None;
    }
    if authority.port_u16().is_some_and(|port| port != 443) {
        return None;
    }
    worker::Url::parse(&format!("https://{host}/api/visky/chat/")).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_only_vionsky_https_origin() {
        let ok: Uri = "https://vionsky.com/vi/".parse().unwrap();
        assert_eq!(chat_api_url(&ok).unwrap().path(), "/api/visky/chat/");
        assert!(chat_api_url(&"https://example.test/".parse().unwrap()).is_none());
        assert!(chat_api_url(&"http://vionsky.com/".parse().unwrap()).is_none());
    }

    #[test]
    fn sanitizes_untrusted_history_shape() {
        let history = vec![
            ChatTurn {
                role: "user".into(),
                content: "hi".into(),
            },
            ChatTurn {
                role: "system".into(),
                content: "drop".into(),
            },
        ];
        assert_eq!(sanitize_history(&history).len(), 1);
    }
}
