use crate::api::search_public_content;
use crate::search::{SearchOutcome, SearchPost, SearchProduct, SearchResponse};
use crate::seo::{search_seo, seo_view};
use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_query_map};

#[cfg(feature = "ssr")]
type PageResponseOptions = Option<leptos_axum::ResponseOptions>;
#[cfg(not(feature = "ssr"))]
type PageResponseOptions = Option<()>;

#[cfg(feature = "ssr")]
fn set_page_status(options: &PageResponseOptions, code: u16) {
    use axum::http::StatusCode;

    if let (Some(options), Ok(status)) = (options, StatusCode::from_u16(code)) {
        options.set_status(status);
    }
}

#[cfg(not(feature = "ssr"))]
fn set_page_status(_: &PageResponseOptions, _: u16) {}

#[cfg(feature = "ssr")]
fn set_page_no_store(options: &PageResponseOptions) {
    use axum::http::{header::CACHE_CONTROL, HeaderValue};

    if let Some(options) = options {
        options.insert_header(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
}

#[cfg(not(feature = "ssr"))]
fn set_page_no_store(_: &PageResponseOptions) {}

/// Search the public product catalogue and community posts at `/vi/search`.
#[component]
pub fn SearchPage() -> impl IntoView {
    let page_response = {
        #[cfg(feature = "ssr")]
        {
            use_context::<leptos_axum::ResponseOptions>()
        }
        #[cfg(not(feature = "ssr"))]
        {
            None
        }
    };
    set_page_no_store(&page_response);

    let query = use_query_map();
    let raw_query = move || query.get().get("q").unwrap_or_default();
    let normalized_query = move || raw_query().trim().to_owned();

    // `new_blocking` plus the route's Async SSR mode puts the current URL's
    // results in the initial HTML. A normal GET form keeps search usable without JS.
    let results = Resource::new_blocking(
        move || normalized_query(),
        |query| async move {
            if query.is_empty() {
                Ok(SearchOutcome::Loaded(SearchResponse::default()))
            } else {
                search_public_content(query).await
            }
        },
    );

    view! {
        {move || seo_view(search_seo("vi", &normalized_query()))}
        <div class="page-shell search-page">
            <header class="search-page__heading">
                <p class="search-page__eyebrow">"Khám phá VionSky"</p>
                <h1 id="search-title">"Tìm trong VionSky"</h1>
                <p class="search-page__lede">
                    "Khám phá sản phẩm và những cuộc trò chuyện đang diễn ra trong cộng đồng."
                </p>
            </header>

            <form class="search-form" method="get" action="/vi/search" role="search" aria-labelledby="search-title">
                <label class="search-form__field" for="vionsky-search">
                    <span>"Từ khóa tìm kiếm"</span>
                    <input
                        id="vionsky-search"
                        name="q"
                        type="search"
                        autocomplete="off"
                        value=move || raw_query()
                        prop:value=move || raw_query()
                        required
                    />
                </label>
                <button class="search-form__submit" type="submit">
                    <svg viewBox="0 0 24 24" fill="none" aria-hidden="true" xmlns="http://www.w3.org/2000/svg">
                        <circle cx="10.8" cy="10.8" r="6.4" stroke="currentColor" stroke-width="1.8" />
                        <path d="m15.5 15.5 4.2 4.2" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
                    </svg>
                    <span>"Tìm kiếm"</span>
                </button>
            </form>

            <section class="search-page__results" aria-label="Kết quả tìm kiếm">
                <Suspense fallback=|| view! { <SearchLoadingState/> }>
                    {move || {
                        let current_query = normalized_query();
                        if current_query.is_empty() {
                            return view! { <SearchPrompt/> }.into_any();
                        }

                        match results.get() {
                            None => view! { <SearchLoadingState/> }.into_any(),
                            Some(Err(_)) => {
                                set_page_status(&page_response, 502);
                                view! { <SearchErrorState query=current_query retry_after_seconds=None is_rate_limited=false/> }.into_any()
                            }
                            Some(Ok(SearchOutcome::Unavailable)) => {
                                set_page_status(&page_response, 502);
                                view! { <SearchErrorState query=current_query retry_after_seconds=None is_rate_limited=false/> }.into_any()
                            }
                            Some(Ok(SearchOutcome::RateLimited { retry_after_seconds })) => {
                                set_page_status(&page_response, 429);
                                view! { <SearchErrorState query=current_query retry_after_seconds=retry_after_seconds is_rate_limited=true/> }.into_any()
                            }
                            Some(Ok(SearchOutcome::Loaded(response))) => {
                                set_page_status(&page_response, 200);
                                search_results(response, current_query).into_any()
                            }
                        }
                    }}
                </Suspense>
            </section>
        </div>
    }
}

#[component]
fn SearchPrompt() -> impl IntoView {
    view! {
        <div class="search-state" role="status">
            <h2>"Nhập từ khóa để bắt đầu"</h2>
            <p>"Tìm theo tên sản phẩm, chủ đề hoặc nội dung bài viết."</p>
        </div>
    }
}

#[component]
fn SearchLoadingState() -> impl IntoView {
    view! {
        <div class="search-loading" role="status" aria-live="polite" aria-busy="true">
            <p>"Đang tìm sản phẩm và bài viết…"</p>
            <div class="search-loading__rows" aria-hidden="true">
                <span class="skeleton skeleton--row"></span>
                <span class="skeleton skeleton--row"></span>
            </div>
        </div>
    }
}

#[component]
fn SearchErrorState(
    query: String,
    retry_after_seconds: Option<u64>,
    is_rate_limited: bool,
) -> impl IntoView {
    let retry_url = format!("/vi/search?q={}", encode_query_component(&query));
    let detail = match retry_after_seconds {
        Some(seconds) => {
            format!("Tìm kiếm đang giới hạn yêu cầu. Hãy chờ {seconds} giây rồi thử lại.")
        }
        None if is_rate_limited => {
            "Tìm kiếm đang giới hạn yêu cầu. Hãy thử lại sau ít phút.".to_owned()
        }
        None => "Hiện chưa tải được kết quả. Hãy thử lại sau ít phút.".to_owned(),
    };

    view! {
        <div class="search-state search-state--error" role="alert">
            <h2>"Tìm kiếm đang tạm gián đoạn"</h2>
            <p>{detail}</p>
            {(!is_rate_limited).then(|| view! {
                <A href=retry_url attr:class="search-state__retry">"Thử lại"</A>
            })}
        </div>
    }
}

fn search_results(response: SearchResponse, query: String) -> impl IntoView {
    let game_count = response.games.len();
    let post_count = response.posts.len();
    if game_count == 0 && post_count == 0 {
        return view! {
            <div class="search-state search-state--empty" role="status">
                <h2>{format!("Chưa thấy kết quả cho “{query}”")}</h2>
                <p>"Thử từ khóa ngắn hơn, đổi cách viết hoặc tìm theo một chủ đề khác."</p>
                <A href="/vi/" attr:class="search-state__link">"Khám phá sản phẩm"</A>
            </div>
        }
        .into_any();
    }

    let visible_games = response.games;
    let visible_posts = response.posts;
    view! {
        <div class="search-results">
            <p class="search-results__summary" role="status">
                "Kết quả cho " <strong>{format!("“{query}”")}</strong>
            </p>

            {if !visible_games.is_empty() {
                view! {
                    <section class="search-section" aria-labelledby="search-products-title">
                        <header class="search-section__heading">
                            <h2 id="search-products-title">"Sản phẩm"</h2>
                            <span>{format!("{game_count}")}</span>
                        </header>
                        <ol class="search-list">
                            {visible_games.into_iter().map(|game| view! { <SearchProductResult game=game/> }).collect_view()}
                        </ol>
                    </section>
                }.into_any()
            } else {
                ().into_any()
            }}

            {if !visible_posts.is_empty() {
                view! {
                    <section class="search-section" aria-labelledby="search-posts-title">
                        <header class="search-section__heading">
                            <h2 id="search-posts-title">"Bài viết cộng đồng"</h2>
                            <span>{format!("{post_count}")}</span>
                        </header>
                        <ol class="search-list">
                            {visible_posts.into_iter().map(|post| view! { <SearchPostResult post=post/> }).collect_view()}
                        </ol>
                    </section>
                }.into_any()
            } else {
                ().into_any()
            }}
        </div>
    }
    .into_any()
}

#[component]
fn SearchProductResult(game: SearchProduct) -> impl IntoView {
    let href = format!("/vi/game/{}/", encode_path_segment(&game.slug));
    let title = game.title;
    let description = game.description;
    let category = game.category;
    let plays = game.plays;
    let rating = format!("{:.1}", game.rating);
    let created_at = game.created_at;
    let created_date = created_at.clone();
    let id = format!("search-product-{}", game.id);
    let cover_image = safe_image_url(&game.cover_image);

    view! {
        <li class="search-list__item" id=id>
            <article class="search-result search-result--product">
                <div class="search-result__visual" aria-hidden="true">
                    {cover_image.clone().map(|source| view! {
                        <img
                            src=source
                            alt=""
                            width="160"
                            height="100"
                            loading="lazy"
                            decoding="async"
                            referrerpolicy="no-referrer"
                        />
                    })}
                    {cover_image.is_none().then(|| view! {
                        <svg viewBox="0 0 96 72" fill="none" xmlns="http://www.w3.org/2000/svg">
                            <path d="M15 44C25 17 57 13 80 29M15 44C29 59 56 62 80 43" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                            <circle cx="15" cy="44" r="4" fill="currentColor" />
                            <circle cx="48" cy="23" r="4" fill="currentColor" />
                            <circle class="search-result__visual-accent" cx="80" cy="36" r="4" fill="currentColor" />
                        </svg>
                    })}
                </div>
                <div class="search-result__body">
                    <p class="search-result__kind">"Sản phẩm"</p>
                    <h3><A href=href attr:class="search-result__title">{title}</A></h3>
                    {(!description.trim().is_empty()).then(|| view! {
                        <p class="search-result__description">{description}</p>
                    })}
                    <dl class="search-result__meta">
                        <div><dt>"Danh mục"</dt><dd>{category}</dd></div>
                        <div><dt>"Lượt chơi"</dt><dd>{plays}</dd></div>
                        <div><dt>"Đánh giá"</dt><dd>{rating}</dd></div>
                        <div><dt>"Ngày đăng"</dt><dd><time datetime=created_at>{created_date}</time></dd></div>
                    </dl>
                </div>
            </article>
        </li>
    }
}

#[component]
fn SearchPostResult(post: SearchPost) -> impl IntoView {
    let href = format!("/vi/community/{}", post.id);
    let title = post.title;
    let preview = post.content_preview;
    let user_name = post.user_name;
    let created_at = post.created_at;
    let created_date = created_at.clone();
    let comment_count = post.comment_count;
    let id = format!("search-post-{}", post.id);

    view! {
        <li class="search-list__item" id=id>
            <article class="search-result search-result--post">
                <div class="search-result__body">
                    <p class="search-result__kind">"Bài viết cộng đồng"</p>
                    <h3><A href=href attr:class="search-result__title">{title}</A></h3>
                    <p class="search-result__description">{preview}</p>
                    <footer class="search-result__meta search-result__meta--post">
                        <span>{user_name}</span>
                        <time datetime=created_at>{created_date}</time>
                        <span>{format!("{comment_count} bình luận")}</span>
                    </footer>
                </div>
            </article>
        </li>
    }
}

fn safe_image_url(value: &str) -> Option<String> {
    let value = value.trim();
    let lower = value.to_ascii_lowercase();
    lower.starts_with("https://").then(|| value.to_owned())
}

fn encode_path_segment(value: &str) -> String {
    encode_uri_component(value)
}

fn encode_query_component(value: &str) -> String {
    encode_uri_component(value)
}

fn encode_uri_component(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(value.len());

    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(HEX[(byte >> 4) as usize] as char);
            encoded.push(HEX[(byte & 0x0f) as usize] as char);
        }
    }

    encoded
}

#[cfg(test)]
mod tests {
    use super::{encode_path_segment, encode_query_component, safe_image_url};

    #[test]
    fn encodes_product_slugs_and_search_queries_as_single_components() {
        assert_eq!(
            encode_path_segment("tool kit/Đẹp"),
            "tool%20kit%2F%C4%90%E1%BA%B9p"
        );
        assert_eq!(
            encode_query_component("áo & gió"),
            "%C3%A1o%20%26%20gi%C3%B3"
        );
    }

    #[test]
    fn accepts_only_http_image_urls() {
        assert_eq!(
            safe_image_url(" https://cdn.example.test/a.webp ").as_deref(),
            Some("https://cdn.example.test/a.webp")
        );
        assert_eq!(safe_image_url("http://cdn.example.test/a.webp"), None);
        assert_eq!(safe_image_url("javascript:alert(1)"), None);
        assert_eq!(safe_image_url("/relative/image.webp"), None);
    }
}
