//! Direct Supabase login/logout on Leptos (no hand-off to Next.js).

use crate::api::{leptos_sign_in, leptos_sign_out};
use crate::auth::AuthSession;
use crate::auth_client::{
    access_token, auth_signal, clear_session, fetch_with_auth, store_session,
};
use crate::seo::{seo_view, PageSeo};
use leptos::{ev::SubmitEvent, prelude::*, task::spawn_local};
use leptos_router::{components::A, hooks::use_navigate};

#[component]
pub fn LoginPage() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let pending = RwSignal::new(false);
    let verified = RwSignal::new(false);
    let session = auth_signal();
    let navigate = use_navigate();

    let on_submit = move |event: SubmitEvent| {
        event.prevent_default();
        if pending.get_untracked() {
            return;
        }
        if email.get_untracked().trim().is_empty() || password.get_untracked().is_empty() {
            error.set(Some("Nhập email và mật khẩu".to_string()));
            return;
        }
        error.set(None);
        pending.set(true);
        let submitted_email = email.get_untracked();
        let submitted_password = password.get_untracked();
        let navigate = navigate.clone();
        spawn_local(async move {
            match leptos_sign_in(submitted_email, submitted_password).await {
                Ok(result) if result.session.is_some() => {
                    let auth = result.session.expect("checked above");
                    store_session(&auth);
                    session.set(Some(auth.clone()));
                    verified.set(fetch_with_auth("/api/users/me/").await.is_ok());
                    password.set(String::new());
                    navigate("/vi/", Default::default());
                }
                Ok(result) => error.set(result.error.or(Some("Đăng nhập thất bại".to_string()))),
                Err(problem) => error.set(Some(problem.to_string())),
            }
            pending.set(false);
        });
    };

    view! {
        {seo_view(PageSeo::new("vi", "/login", "Đăng nhập", "Đăng nhập vào Vionsky để quản lý sản phẩm và nhận góp ý.").with_noindex())}
        <div class="page-shell auth-page">
            <h1>"Đăng nhập Vionsky"</h1>
            <Show when=move || error.get().is_some() fallback=|| ()>
                <p class="auth-error" role="alert">{move || error.get().unwrap_or_default()}</p>
            </Show>
            <Show when=move || verified.get() fallback=|| ()>
                <p class="auth-ok" role="status">"Phiên hợp lệ — Bearer token đã được gắn vào API call."</p>
            </Show>
            <form on:submit=on_submit class="auth-form">
                <label for="auth-email">"Email"</label>
                <input
                    id="auth-email"
                    type="email"
                    autocomplete="email"
                    required
                    prop:value=move || email.get()
                    on:input=move |event| email.set(event_target_value(&event))
                />
                <label for="auth-password">"Mật khẩu"</label>
                <input
                    id="auth-password"
                    type="password"
                    autocomplete="current-password"
                    required
                    prop:value=move || password.get()
                    on:input=move |event| password.set(event_target_value(&event))
                />
                <button type="submit" class="control-frame control-frame--primary" disabled=move || pending.get()>
                    {move || if pending.get() { "Đang đăng nhập…" } else { "Đăng nhập" }}
                </button>
            </form>
            <p><A href="/vi/">"Về trang chủ"</A></p>
        </div>
    }
}

#[component]
pub fn AuthControls() -> impl IntoView {
    let session = auth_signal();
    let busy = RwSignal::new(false);

    view! {
        <Show
            when=move || session.get().is_some()
            fallback=move || view! {
                <A href="/vi/login" attr:class="site-nav__link site-nav__link--auth">"Đăng nhập"</A>
            }
        >
            <button
                type="button"
                class="site-nav__link site-nav__link--auth"
                disabled=move || busy.get()
                on:click=move |_| {
                    if busy.get_untracked() {
                        return;
                    }
                    let token = access_token();
                    busy.set(true);
                    let session = session;
                    let busy = busy;
                    spawn_local(async move {
                        let _ = leptos_sign_out(token).await;
                        clear_session();
                        session.set(None);
                        busy.set(false);
                    });
                }
            >
                {move || {
                    let current: Option<AuthSession> = session.get();
                    format!("Đăng xuất {}", current.map(|value| value.user.email).unwrap_or_default())
                }}
            </button>
        </Show>
    }
}
