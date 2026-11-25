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
                <A href="/" attr:class="site-brand" attr:aria-label="VionSky home">
                    <SkyMark />
                    <span class="site-brand__name">"VionSky"</span>
                </A>

                <nav class="site-nav" aria-label="Primary navigation">
                    <A href="/vi/about" attr:class="site-nav__link">"About"</A>
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
                    " is where creators, users, and AI Agents meet to explore, connect, and work together."
                </p>
                <nav class="site-footer__nav" aria-label="Footer navigation">
                    <A href="/vi/about">"About"</A>
                    <A href="/vi/channels">"War Room"</A>
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
