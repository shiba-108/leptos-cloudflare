//! Browser-side session storage and `fetchWithAuth` equivalent.
//!
//! Server rendering never touches localStorage: the SSR stubs keep the same
//! signatures so event-handler code compiles for both targets.

use crate::auth::AuthSession;
#[cfg(not(feature = "ssr"))]
use crate::auth::AUTH_STORAGE_KEY;
use leptos::prelude::*;

#[cfg(not(feature = "ssr"))]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

#[cfg(not(feature = "ssr"))]
pub fn stored_session() -> Option<AuthSession> {
    let raw = storage()?.get_item(AUTH_STORAGE_KEY).ok().flatten()?;
    serde_json::from_str(&raw).ok()
}

#[cfg(feature = "ssr")]
pub fn stored_session() -> Option<AuthSession> {
    None
}

#[cfg(not(feature = "ssr"))]
pub fn store_session(session: &AuthSession) {
    if let Some(storage) = storage() {
        let _ = storage.set_item(
            AUTH_STORAGE_KEY,
            &serde_json::to_string(session).unwrap_or_default(),
        );
    }
}

#[cfg(feature = "ssr")]
pub fn store_session(_: &AuthSession) {}

#[cfg(not(feature = "ssr"))]
pub fn clear_session() {
    if let Some(storage) = storage() {
        let _ = storage.remove_item(AUTH_STORAGE_KEY);
    }
}

#[cfg(feature = "ssr")]
pub fn clear_session() {}

pub fn access_token() -> String {
    stored_session()
        .map(|session| session.access_token)
        .unwrap_or_default()
}

/// Port of the Next.js `fetchWithAuth`: same-origin path + Bearer header from
/// localStorage. API routes remain the authority; this only attaches identity.
#[cfg(not(feature = "ssr"))]
pub async fn fetch_with_auth(path: &str) -> Result<serde_json::Value, String> {
    let token = access_token();
    let mut request = gloo_net::http::Request::get(path);
    if !token.is_empty() {
        request = request.header("Authorization", &format!("Bearer {token}"));
    }
    request = request.header("Accept", "application/json");
    let response = request.send().await.map_err(|error| error.to_string())?;
    let body = response.text().await.map_err(|error| error.to_string())?;
    serde_json::from_str(&body).map_err(|error| error.to_string())
}

#[cfg(feature = "ssr")]
pub async fn fetch_with_auth(_: &str) -> Result<serde_json::Value, String> {
    Err("fetchWithAuth chỉ chạy trên trình duyệt".to_string())
}

/// Shared session signal provided once by `App`, so the nav and login page
/// stay in sync without a reload.
pub fn auth_signal() -> RwSignal<Option<AuthSession>> {
    use_context::<RwSignal<Option<AuthSession>>>()
        .unwrap_or_else(|| RwSignal::new(stored_session()))
}
