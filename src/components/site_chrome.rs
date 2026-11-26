//! src/components/site_chrome.rs
//! Port of Next.js: src/app/[locale]/components/TopNav.tsx,
//! src/app/[locale]/components/chrome-islands.tsx (MobileHeaderBar/MobileDrawer)
//! and src/components/LanguageSwitcher.tsx.
//! Leptos 0.8; uses leptos_router `A` + `attr:class`. Self-contained: append to
//! src/components/mod.rs (`pub mod site_chrome;`) and replace the <header> block
//! in src/components/app_layout.rs with `<SiteHeader/>` (keep main + footer).

use leptos::prelude::*;
#[cfg(not(feature = "ssr"))]
use leptos::task::spawn_local;
use leptos_router::{components::A, hooks::use_location};

use crate::components::auth_page::AuthControls;

/* ------------------------------------------------------------------ */
/* Locale: URL-prefix based, mirrors next-intl localePrefix: "always".  */
/* ------------------------------------------------------------------ */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    Vi,
    En,
}

impl Locale {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Vi => "vi",
            Self::En => "en",
        }
    }
}

pub fn locale_from_path(path: &str) -> Locale {
    if path == "/en" || path.starts_with("/en/") {
        Locale::En
    } else {
        Locale::Vi
    }
}

fn strip_locale(path: &str) -> &str {
    for prefix in ["/vi", "/en"] {
        if let Some(rest) = path.strip_prefix(prefix) {
            if rest.is_empty() || rest.starts_with('/') {
                return rest;
            }
        }
    }
    path
}

/// "/about" -> "/vi/about"; "/" -> "/vi/".
pub fn localized_href(locale: Locale, path: &str) -> String {
    if path.is_empty() || path == "/" {
        format!("/{}/", locale.code())
    } else {
        format!("/{}{}", locale.code(), path)
    }
}

/// Same pathname, other locale = router.replace(pathname, { locale }) in TS.
pub fn swap_locale_path(path: &str, target: Locale) -> String {
    let rest = strip_locale(path);
    localized_href(target, if rest.is_empty() { "/" } else { rest })
}

/* ------------------------------------------------------------------ */
/* Admin: server-authoritative (ADMIN_EMAIL), read from GET /api/users/me */
/* ------------------------------------------------------------------ */

pub type AdminSignal = RwSignal<bool>;

pub fn provide_admin_signal() -> AdminSignal {
    let admin = RwSignal::new(false);
    provide_context(admin);
    admin
}

pub fn admin_signal() -> AdminSignal {
    use_context::<AdminSignal>().unwrap_or_else(|| RwSignal::new(false))
}

/// Faithful port of admin/page.tsx:284-289: `isAdmin = role === "admin"`.
/// The role is decided by the SERVER (src/lib/auth.ts:87 isAdminEmail);
/// never compare emails in the browser. Token comes from auth_client.
pub fn load_admin_role(admin: AdminSignal) {
    #[cfg(not(feature = "ssr"))]
    spawn_local(async move {
        if let Ok(value) = crate::auth_client::fetch_with_auth("/api/users/me").await {
            let is_admin = value
                .get("user")
                .and_then(|u| u.get("role"))
                .and_then(|r| r.as_str())
                == Some("admin");
            admin.set(is_admin);
        }
    });
    #[cfg(feature = "ssr")]
    let _ = admin;
}

/* ------------------------------------------------------------------ */
/* Language switcher (VI / EN)                                         */
/* ------------------------------------------------------------------ */

#[component]
pub fn LocaleToggle(#[prop(default = false)] compact: bool) -> impl IntoView {
    let location = use_location();
    let locale = Memo::new(move |_| locale_from_path(&location.pathname.get()));

    if compact {
        // MobileHeaderBar compact button (LanguageSwitcher.tsx:32-45).
        let next_href = move || {
            let next = match locale.get() {
                Locale::Vi => Locale::En,
                Locale::En => Locale::Vi,
            };
            swap_locale_path(&location.pathname.get(), next)
        };
        let label = move || {
            let current = locale.get().code().to_uppercase();
            let next = match locale.get() {
                Locale::Vi => "EN",
                Locale::En => "VI",
            };
            format!("Ngôn ngữ: {current} → {next}")
        };
        view! {
            <A href=next_href attr:class="lang-toggle lang-toggle--compact" attr:aria-label=label>
                <span class="lang-toggle__globe" aria-hidden="true">"🌐"</span>
                <span class="lang-toggle__code">{move || locale.get().code().to_uppercase()}</span>
            </A>
        }
        .into_any()
    } else {
        // Full VI / EN group (LanguageSwitcher.tsx:47-84).
        let vi_href = move || swap_locale_path(&location.pathname.get(), Locale::Vi);
        let en_href = move || swap_locale_path(&location.pathname.get(), Locale::En);
        view! {
            <div class="lang-toggle" role="group" aria-label="Ngôn ngữ">
                <span class="lang-toggle__globe" aria-hidden="true">"🌐"</span>
                <A href=vi_href attr:class="lang-toggle__option" attr:aria-pressed=move || (locale.get() == Locale::Vi).to_string() attr:aria-label="VI — Tiếng Việt">"VI"</A>
                <span class="lang-toggle__sep" aria-hidden="true">"/"</span>
                <A href=en_href attr:class="lang-toggle__option" attr:aria-pressed=move || (locale.get() == Locale::En).to_string() attr:aria-label="EN — English">"EN"</A>
            </div>
        }.into_any()
    }
}

/* ------------------------------------------------------------------ */
/* Shared desktop + drawer links                                       */
/* ------------------------------------------------------------------ */

#[component]
fn SiteNavLinks(#[prop(default = false)] in_drawer: bool) -> impl IntoView {
    let location = use_location();
    let locale = Memo::new(move |_| locale_from_path(&location.pathname.get()));
    let admin = admin_signal();

    let home = move || localized_href(locale.get(), "/");
    let community = move || localized_href(locale.get(), "/community");
    let dashboard = move || localized_href(locale.get(), "/dashboard");
    let admin_href = move || localized_href(locale.get(), "/admin");

    let class: &'static str = if in_drawer {
        "site-nav__link site-nav__link--drawer"
    } else {
        "site-nav__link"
    };

    view! {
        <A href=home attr:class=class>"Trang chủ"</A>
        <A href=community attr:class=class>"Cộng đồng"</A>
        <A href=dashboard attr:class=class>"Bảng điều khiển"</A>
        <Show when=move || admin.get() fallback=|| ()>
            <A href=admin_href attr:class=class class:site-nav__link--admin=move || admin.get()>"Admin"</A>
        </Show>
    }
}

#[component]
fn SkyMark() -> impl IntoView {
    view! {
        <svg class="sky-mark" viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M7 26C12 11 25 8 33 15M7 26C13 34 25 34 33 25" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <circle class="sky-mark__node" cx="7" cy="26" r="3"/>
            <circle class="sky-mark__node" cx="20" cy="14" r="3"/>
            <circle class="sky-mark__node sky-mark__node--accent" cx="33" cy="25" r="3"/>
        </svg>
    }
}

/* ------------------------------------------------------------------ */
/* Full chrome: desktop header + mobile bar + drawer                  */
/* ------------------------------------------------------------------ */

#[component]
pub fn SiteHeader() -> impl IntoView {
    let location = use_location();
    let locale = Memo::new(move |_| locale_from_path(&location.pathname.get()));
    let admin = admin_signal();
    #[cfg(not(feature = "ssr"))]
    load_admin_role(admin);

    let drawer_open = RwSignal::new(false);
    let brand_href = move || localized_href(locale.get(), "/");

    // Body scroll lock (MobileDrawer.tsx:277-278, 308). Document/HtmlElement/
    // CssStyleDeclaration web-sys features are unified in via leptos->tachys,
    // so no Cargo.toml change is required. CSS :has() fallback is in the CSS.
    #[cfg(not(feature = "ssr"))]
    Effect::new(move |_| {
        let open = drawer_open.get();
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(body) = document.body() {
                let _ = body
                    .style()
                    .set_property("overflow", if open { "hidden" } else { "" });
            }
        }
    });

    let on_drawer_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Escape" {
            drawer_open.set(false);
        }
    };

    view! {
        <a class="skip-link" href="#content">"Skip to content"</a>
        <header class="site-header site-header--chrome">
            // ---- desktop: `md:flex` in TopNav.tsx:54 ----
            <div class="site-header__inner site-header__inner--desktop">
                <A href=brand_href attr:class="site-brand" attr:aria-label="VionSky - Trang chủ">
                    <SkyMark/>
                    <span class="site-brand__name">"VionSky"</span>
                </A>
                <nav class="site-nav site-nav--desktop" aria-label="Điều hướng chính">
                    <SiteNavLinks/>
                </nav>
                <div class="site-nav site-nav--actions">
                    <LocaleToggle/>
                    <AuthControls/>
                </div>
            </div>

            // ---- mobile: `md:hidden` in chrome-islands.tsx:372 ----
            <div class="site-header__inner site-header__inner--mobile">
                <div class="site-header__mobile-left">
                    <button
                        type="button"
                        class="site-nav__hamburger"
                        aria-label="Menu"
                        aria-expanded=move || drawer_open.get().to_string()
                        on:click=move |_| drawer_open.update(|open| *open = !*open)
                    >
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                            <path d="M3 6h18M3 12h18M3 18h18"/>
                        </svg>
                    </button>
                    <A href=brand_href attr:class="site-brand site-brand--mobile">
                        <SkyMark/>
                        <span class="site-brand__name">"VionSky"</span>
                    </A>
                </div>
                <div class="site-nav site-nav--mobile-actions">
                    <LocaleToggle compact=true/>
                    <div class="site-nav__mobile-auth"><AuthControls/></div>
                </div>
            </div>
        </header>

        // ---- drawer: MobileDrawer; no portal needed in Leptos ----
        <div class="site-nav__scrim" class:is-open=move || drawer_open.get() aria-hidden="true" on:click=move |_| drawer_open.set(false)></div>
        <div
            class="site-nav__drawer"
            class:is-open=move || drawer_open.get()
            role="dialog"
            aria-modal="true"
            aria-label="Menu"
            tabindex="-1"
            on:keydown=on_drawer_keydown
        >
            <div class="site-nav__drawer-head">
                <span class="site-brand__name">"VionSky"</span>
                <button type="button" class="site-nav__close" autofocus aria-label="Đóng menu" on:click=move |_| drawer_open.set(false)>"✕"</button>
            </div>
            <nav class="site-nav site-nav--drawer" aria-label="Điều hướng chính">
                <SiteNavLinks in_drawer=true/>
                <LocaleToggle/>
                <AuthControls/>
            </nav>
        </div>
    }
}
