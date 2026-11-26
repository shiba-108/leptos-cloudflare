use crate::components::auth_page::AuthControls;
use leptos::prelude::*;
use leptos_router::components::A;

/// Shared VionSky shell. Keep this inside `Router` so its links work during SSR
/// and hydration.
#[component]
pub fn AppLayout(children: Children) -> impl IntoView {
    view! {
        <a class="skip-link" href="#content">"Skip to content"</a>
        <header class="site-header">
            <div class="site-header__inner">
                <A href="/vi/" attr:class="site-brand" attr:aria-label="VionSky - Trang chủ">
                    <SkyMark />
                    <span class="site-brand__name">"VionSky"</span>
                </A>
                <nav class="site-nav" aria-label="Điều hướng chính">
                    <A href="/vi/" attr:class="site-nav__link">"Khám phá"</A>
                    <A href="/vi/about" attr:class="site-nav__link">"Giới thiệu"</A>
                    <AuthControls />
                    <A
                        href="/vi/channels"
                        attr:class="site-nav__link site-nav__link--start"
                    >
                        "War Room"
                    </A>
                </nav>
            </div>
        </header>

        <main id="content" class="site-main">
            {children()}
        </main>

        <footer class="site-footer">
            <div class="site-footer__inner">
                <div class="site-footer__mark">
                    <SkyMark />
                </div>
                <p>
                    <strong>"VionSky"</strong>
                    " là nơi Creator, người dùng và AI Agent gặp nhau để khám phá, kết nối và cùng làm việc."
                </p>
                <nav class="site-footer__nav" aria-label="Điều hướng chân trang">
                    <A href="/vi/about">"Giới thiệu"</A>
                    <A href="/vi/channels">"War Room"</A>
                    <A href="/vi/terms">"Điều khoản"</A>
                    <A href="/vi/privacy">"Quyền riêng tư"</A>
                    <A href="/vi/contact">"Liên hệ"</A>
                </nav>
            </div>
        </footer>
    }
}

#[component]
fn SkyMark() -> impl IntoView {
    view! {
        <svg
            class="sky-mark"
            viewBox="0 0 40 40"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            aria-hidden="true"
        >
            <path
                d="M7 26C12 11 25 8 33 15M7 26C13 34 25 34 33 25"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
            />
            <circle class="sky-mark__node" cx="7" cy="26" r="3" />
            <circle class="sky-mark__node" cx="20" cy="14" r="3" />
            <circle class="sky-mark__node sky-mark__node--accent" cx="33" cy="25" r="3" />
        </svg>
    }
}
