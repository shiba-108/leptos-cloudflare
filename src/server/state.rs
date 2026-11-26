use std::sync::Arc;

use axum::extract::FromRef;
use leptos::prelude::LeptosOptions;

#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub env: Arc<worker::Env>,
    pub session_id: Arc<str>,
    pub client_ip: Option<Arc<str>>,
    /// Original Supabase session cookie, forwarded only to the authenticated
    /// same-origin upstream API from server functions.
    pub cookie: Option<Arc<str>>,
    /// Bearer token from the original request, forwarded only upstream.
    pub auth_header: Option<Arc<str>>,
}

impl AppState {
    pub fn new(
        leptos_options: LeptosOptions,
        env: worker::Env,
        session_id: String,
        client_ip: Option<String>,
        cookie: Option<String>,
        auth_header: Option<String>,
    ) -> Self {
        Self {
            leptos_options,
            env: Arc::new(env),
            session_id: Arc::<str>::from(session_id),
            client_ip: client_ip.map(Arc::<str>::from),
            cookie: cookie.map(Arc::<str>::from),
            auth_header: auth_header.map(Arc::<str>::from),
        }
    }

    pub fn from_context() -> Result<Self, leptos::prelude::ServerFnError> {
        leptos::prelude::use_context::<Self>().ok_or_else(|| {
            leptos::prelude::ServerFnError::ServerError("Request context unavailable".to_string())
        })
    }

    pub fn db(&self) -> worker::Result<worker::D1Database> {
        self.env.d1("DB")
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(input: &AppState) -> Self {
        input.leptos_options.clone()
    }
}
