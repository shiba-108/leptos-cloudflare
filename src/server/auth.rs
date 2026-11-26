//! Server-side Supabase Auth calls. The anon key is public by design; the
//! service-role key is never referenced here.

use crate::auth::{auth_error_message, parse_sign_in_payload, AuthConfig, AuthResult};
use crate::server::{AppError, AppResult, AppState};
use serde_json::{json, Value};

const MAX_EMAIL_CHARS: usize = 254;
const MAX_PASSWORD_CHARS: usize = 256;
const MAX_RESPONSE_BYTES: usize = 262_144;

pub fn auth_config(state: &AppState) -> AppResult<AuthConfig> {
    let url = state
        .env
        .var("NEXT_PUBLIC_SUPABASE_URL")
        .map_err(|_| AppError::Internal)?
        .to_string();
    let anon_key = state
        .env
        .var("NEXT_PUBLIC_SUPABASE_ANON_KEY")
        .map_err(|_| AppError::Internal)?
        .to_string();
    if url.is_empty() || anon_key.is_empty() {
        return Err(AppError::Internal);
    }
    Ok(AuthConfig {
        url: url.trim_end_matches('/').to_string(),
        anon_key,
    })
}

async fn supabase_request(
    state: &AppState,
    method: worker::Method,
    path: &str,
    body: Option<Value>,
    bearer: Option<&str>,
) -> AppResult<(u16, Value)> {
    let config = auth_config(state)?;
    let url =
        worker::Url::parse(&format!("{}{path}", config.url)).map_err(|_| AppError::Internal)?;
    let mut init = worker::RequestInit::new();
    init.with_method(method);
    init.with_cache(worker::CacheMode::NoStore);
    init.headers
        .set("apikey", &config.anon_key)
        .map_err(|_| AppError::Internal)?;
    init.headers
        .set("Content-Type", "application/json")
        .map_err(|_| AppError::Internal)?;
    init.headers
        .set("Accept", "application/json")
        .map_err(|_| AppError::Internal)?;
    if let Some(token) = bearer {
        init.headers
            .set("Authorization", &format!("Bearer {token}"))
            .map_err(|_| AppError::Internal)?;
    }
    if let Some(ref value) = body {
        init.with_body(Some(wasm_bindgen::JsValue::from_str(&value.to_string())));
    }

    let request =
        worker::Request::new_with_init(url.as_str(), &init).map_err(|_| AppError::Internal)?;
    let mut response = worker::Fetch::Request(request)
        .send()
        .await
        .map_err(|_| AppError::client("Không kết nối được máy chủ xác thực"))?;
    let status = response.status_code();
    let text = response.text().await.map_err(|_| AppError::Internal)?;
    if text.len() > MAX_RESPONSE_BYTES {
        return Err(AppError::Internal);
    }
    let payload = serde_json::from_str::<Value>(&text).unwrap_or(Value::Null);
    Ok((status, payload))
}

fn failure(status: u16, payload: &Value, fallback: &str) -> AuthResult {
    AuthResult {
        session: None,
        error: Some(auth_error_message(payload).unwrap_or_else(|| {
            if status == 400 || status == 401 || status == 422 {
                fallback.to_string()
            } else {
                "Máy chủ xác thực tạm thời không khả dụng".to_string()
            }
        })),
        signed_out: false,
    }
}

pub async fn sign_in(state: &AppState, email: String, password: String) -> AppResult<AuthResult> {
    let email = email.trim().to_owned();
    if email.is_empty() || email.len() > MAX_EMAIL_CHARS || !email.contains('@') {
        return Err(AppError::client("Email không hợp lệ"));
    }
    if password.is_empty() || password.len() > MAX_PASSWORD_CHARS {
        return Err(AppError::client("Mật khẩu không hợp lệ"));
    }
    let body = json!({ "email": email, "password": password });
    let (status, payload) = supabase_request(
        state,
        worker::Method::Post,
        "/auth/v1/token?grant_type=password",
        Some(body),
        None,
    )
    .await?;
    if !(200..300).contains(&status) {
        return Ok(failure(status, &payload, "Email hoặc mật khẩu không đúng"));
    }
    Ok(parse_sign_in_payload(&payload))
}

pub async fn sign_out(state: &AppState, token: String) -> AppResult<AuthResult> {
    if token.is_empty() {
        return Ok(AuthResult {
            signed_out: true,
            ..AuthResult::default()
        });
    }
    let response = supabase_request(
        state,
        worker::Method::Post,
        "/auth/v1/logout",
        None,
        Some(&token),
    )
    .await;
    match response {
        Ok((code, payload)) if (200..300).contains(&code) => Ok(AuthResult {
            signed_out: true,
            session: None,
            error: auth_error_message(&payload),
        }),
        _ => Ok(AuthResult {
            signed_out: true,
            session: None,
            error: None,
        }),
    }
}

pub async fn refresh_session(state: &AppState, refresh_token: String) -> AppResult<AuthResult> {
    if refresh_token.is_empty() {
        return Err(AppError::client("Không có phiên để làm mới"));
    }
    let body = json!({ "refresh_token": refresh_token });
    let (status, payload) = supabase_request(
        state,
        worker::Method::Post,
        "/auth/v1/token?grant_type=refresh_token",
        Some(body),
        None,
    )
    .await?;
    if !(200..300).contains(&status) {
        return Ok(failure(status, &payload, "Phiên đăng nhập đã hết hạn"));
    }
    Ok(parse_sign_in_payload(&payload))
}
