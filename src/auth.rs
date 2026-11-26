//! Shared Supabase auth wire types for the Leptos frontend.

use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "ssr", allow(dead_code))]
pub const AUTH_STORAGE_KEY: &str = "vionsky.auth.session";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AuthSession {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: i64,
    pub user: AuthUser,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AuthConfig {
    pub url: String,
    pub anon_key: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct AuthResult {
    pub session: Option<AuthSession>,
    pub error: Option<String>,
    #[serde(default)]
    pub signed_out: bool,
}

/// Parse a Supabase `/auth/v1/token` success payload. Kept separate from JSON
/// deserialization so malformed/non-email rows cannot become a session.
pub fn parse_sign_in_payload(value: &serde_json::Value) -> AuthResult {
    let access_token = value
        .get("access_token")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    if access_token.is_empty() {
        return AuthResult {
            error: Some("Máy chủ xác thực không trả về phiên đăng nhập".to_string()),
            ..AuthResult::default()
        };
    }
    let user_value = value.get("user").unwrap_or(&serde_json::Value::Null);
    let email = user_value
        .get("email")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    let id = user_value
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    if id.is_empty() || email.is_empty() {
        return AuthResult {
            error: Some("Thông tin người dùng không hợp lệ".to_string()),
            ..AuthResult::default()
        };
    }
    let display_name = user_value
        .get("user_metadata")
        .and_then(|meta| meta.get("display_name"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);

    AuthResult {
        session: Some(AuthSession {
            access_token,
            refresh_token: value
                .get("refresh_token")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            token_type: value
                .get("token_type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("bearer")
                .to_owned(),
            expires_in: value
                .get("expires_in")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(3600),
            user: AuthUser {
                id,
                email,
                display_name,
            },
        }),
        error: None,
        signed_out: false,
    }
}

pub fn auth_error_message(value: &serde_json::Value) -> Option<String> {
    value
        .get("msg")
        .or_else(|| value.get("message"))
        .or_else(|| value.get("error_description"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_supabase_token_payload() {
        let payload = json!({
            "access_token": "token-123",
            "refresh_token": "refresh-123",
            "token_type": "bearer",
            "expires_in": 3600,
            "user": {
                "id": "user-1",
                "email": "creator@example.com",
                "user_metadata": { "display_name": "Creator" }
            }
        });
        let result = parse_sign_in_payload(&payload);
        let session = result.session.expect("valid session");
        assert_eq!(session.user.display_name.as_deref(), Some("Creator"));
        assert_eq!(session.access_token, "token-123");
        assert!(result.error.is_none());
    }

    #[test]
    fn rejects_payloads_without_tokens_or_user() {
        assert!(parse_sign_in_payload(&json!({})).error.is_some());
        assert!(parse_sign_in_payload(&json!({ "access_token": "x" }))
            .error
            .is_some());
    }

    #[test]
    fn reads_supabase_error_messages() {
        let payload = json!({ "msg": "Invalid login credentials" });
        assert_eq!(
            auth_error_message(&payload).as_deref(),
            Some("Invalid login credentials")
        );
    }
}
