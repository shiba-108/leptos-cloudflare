use crate::api::search_public_content;
use crate::search::SearchOutcome;
use crate::seo::{home_seo, seo_view};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};
#[cfg(any(feature = "ssr", test))]
use serde_json::Value;

const HOME_GAME_LIMIT: usize = 6;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProductSummary {
    title: String,
    slug: String,
    description: Option<String>,
    cover_image: Option<String>,
    creator: Option<String>,
}

#[cfg(any(feature = "ssr", test))]
impl ProductSummary {
    pub(crate) fn from_public_row(row: &Value) -> Option<Self> {
        if text_field(row, "status").as_deref() != Some("approved") {
            return None;
        }

        Some(Self {
            title: text_field(row, "title")?,
            slug: text_field(row, "slug")?,
            description: text_field(row, "description"),
            cover_image: text_field(row, "cover_image").filter(|url| is_http_url(url)),
            creator: text_field(row, "creator"),
        })
    }
}

#[cfg(any(feature = "ssr", test))]
fn text_field(row: &Value, field: &str) -> Option<String> {
    row.get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(any(feature = "ssr", test))]
fn is_http_url(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    value.starts_with("https://") || value.starts_with("http://")
}

fn encode_path_segment(value: &str) -> String {
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

#[component]
fn ProductCard(game: ProductSummary) -> impl IntoView {
    let href = format!("/vi/game/{}/", encode_path_segment(&game.slug));
    let title = game.title;
    let description = game.description;
    let cover_image = game.cover_image;
    let creator = game.creator;

    view! {
        <article class="home-product">
            <A href=href attr:class="home-product__link">
                <div class="home-product__visual">
                    {cover_image.clone().map(|source| view! {
                        <img
                            class="home-product__cover"
                            src=source
                            alt=""
                            width="640"
                            height="360"
                            loading="lazy"
                            decoding="async"
                            referrerpolicy="no-referrer"
                        />
                    })}
                    {cover_image.is_none().then(|| view! {
                        <div class="home-product__placeholder" aria-hidden="true">
                            <svg viewBox="0 0 96 72" fill="none" xmlns="http://www.w3.org/2000/svg">
                                <path d="M15 44C25 17 57 13 80 29M15 44C29 59 56 62 80 43" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                                <circle cx="15" cy="44" r="4" fill="currentColor" />
                                <circle cx="48" cy="23" r="4" fill="currentColor" />
                                <circle class="home-product__placeholder-accent" cx="80" cy="36" r="4" fill="currentColor" />
                            </svg>
                        </div>
                    })}
                </div>
                <div class="home-product__copy">
                    <h3>{title}</h3>
                    {description.map(|text| view! { <p class="home-product__description">{text}</p> })}
                    {creator.map(|name| view! {
                        <p class="home-product__creator">"Tạo bởi " {name}</p>
                    })}
                </div>
            </A>
        </article>
    }
}

const TASK_TRIALS: [(&str, &str, &[&str]); 3] = [
    (
        "puzzle",
        "🧩 Game giải đố nhẹ nhàng",
        &["puzzle", "relaxing", "nhan"],
    ),
    (
        "shooter",
        "🔫 Thử thách bắn súng AI",
        &["shooter", "action"],
    ),
    ("rpg", "📖 Game nhập vai / Tu tiên", &["rpg", "tien"]),
];

/// Three task cards that resolve to the first non-empty search result,
/// mirroring the Next.js TaskTrials island (d95287c:28-58).
#[component]
fn TaskTrials(
    task_slugs: RwSignal<Option<Vec<String>>>,
    task_label: RwSignal<Option<String>>,
) -> impl IntoView {
    let busy = RwSignal::new(false);

    let run = move |title: &'static str, queries: &'static [&'static str]| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        spawn_local(async move {
            let mut slugs: Vec<String> = Vec::new();
            for query in queries {
                if let Ok(SearchOutcome::Loaded(response)) =
                    search_public_content((*query).to_string()).await
                {
                    let found: Vec<String> = response
                        .games
                        .into_iter()
                        .map(|game| game.slug)
                        .filter(|slug| !slug.is_empty())
                        .take(10)
                        .collect();
                    if !found.is_empty() {
                        slugs = found;
                        break;
                    }
                }
            }
            task_slugs.set(Some(slugs));
            task_label.set(Some(title.to_owned()));
            busy.set(false);
        });
    };

    let clear = move |_| {
        task_slugs.set(None);
        task_label.set(None);
    };

    view! {
        <section class="task-trials" aria-labelledby="task-trials-title">
            <div class="task-trials__heading">
                <p class="task-trials__eyebrow">"Task Trials"</p>
                <h2 id="task-trials-title">"Bạn muốn tìm game gì hôm nay?"</h2>
                <p>"Chọn việc bạn muốn làm — VionSky tìm game phù hợp."</p>
            </div>
            <div class="task-trials__grid">
                {TASK_TRIALS.iter().map(|(id, title, queries)| {
                    let id = *id;
                    let title = *title;
                    let queries = *queries;
                    view! {
                        <button
                            type="button"
                            class="task-trial-card"
                            disabled=move || busy.get()
                            on:click=move |_| run(title, queries)
                        >
                            <span class="task-trial-card__icon" aria-hidden="true">{id}</span>
                            <span class="task-trial-card__body">
                                <strong>{move || if busy.get() { "Đang tìm…" } else { title }}</strong>
                                <span class="task-trial-card__cta">"Tìm ngay →"</span>
                            </span>
                        </button>
                    }
                }).collect_view()}
            </div>
            {move || task_slugs.get().map(|slugs| view! {
                <div class="task-trials__active" role="status">
                    <span>{move || format!("Đang lọc theo: {}", task_label.get().unwrap_or_default())}</span>
                    <span>{move || format!("{} gợi ý", slugs.len())}</span>
                    <button type="button" on:click=clear>"Xem tất cả"</button>
                </div>
            })}
        </section>
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let task_slugs = RwSignal::new(None::<Vec<String>>);
    let task_label = RwSignal::new(None::<String>);
    let games = Resource::new_blocking(|| (), |_| crate::api::get_public_games());

    view! {
        {seo_view(home_seo("vi"))}
        <div class="page-shell page-shell--home home-page">
            <section class="home-hero" aria-labelledby="home-title">
                <div class="home-hero__copy">
                    <p class="home-hero__eyebrow">"Dưới cùng một bầu trời"</p>
                    <h1 id="home-title">"Khám phá sản phẩm. Gặp gỡ cộng đồng."</h1>
                    <p class="home-hero__lede">
                        "VionSky mở không gian để Creator chia sẻ, Người dùng trải nghiệm và AI Agent cùng tham gia vào công việc hữu ích. Bạn có thể bắt đầu bằng cách quan sát."
                    </p>
                    <div class="home-hero__actions">
                        <a class="home-hero__primary" href="#san-pham">"Khám phá sản phẩm"</a>
                        <A href="/vi/channels" attr:class="home-hero__secondary">"Mở War Room"</A>
                    </div>
                </div>
                <figure class="home-sky" aria-hidden="true">
                    <svg class="home-sky__orbit" viewBox="0 0 440 360" fill="none" xmlns="http://www.w3.org/2000/svg">
                        <ellipse cx="220" cy="180" rx="166" ry="88" transform="rotate(-23 220 180)" stroke="currentColor" stroke-width="1.5" />
                        <ellipse cx="220" cy="180" rx="166" ry="88" transform="rotate(36 220 180)" stroke="currentColor" stroke-width="1.5" />
                        <path d="M91 124C123 83 180 68 224 83" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                        <circle cx="99" cy="117" r="7" fill="currentColor" />
                        <circle class="home-sky__node--bright" cx="219" cy="87" r="7" fill="currentColor" />
                        <circle cx="334" cy="236" r="7" fill="currentColor" />
                        <circle class="home-sky__core" cx="220" cy="180" r="18" stroke="currentColor" stroke-width="1.5" />
                        <circle class="home-sky__core-dot" cx="220" cy="180" r="4" fill="currentColor" />
                    </svg>
                    <span class="home-sky__label home-sky__label--creator">"Creator"</span>
                    <span class="home-sky__label home-sky__label--user">"Người dùng"</span>
                    <span class="home-sky__label home-sky__label--agent">"AI Agent"</span>
                </figure>
            </section>

            <TaskTrials task_slugs=task_slugs task_label=task_label />

            <section class="home-catalog" id="san-pham" aria-labelledby="home-catalog-title">
                <header class="home-section-heading">
                    <div>
                        <p class="home-section-heading__eyebrow">"Khám phá"</p>
                        <h2 id="home-catalog-title">"Sản phẩm từ cộng đồng"</h2>
                        <p>"Mỗi sản phẩm mở ra một cách mới để chơi, sáng tạo hoặc giải quyết công việc."</p>
                    </div>
                    <A href="/vi/search/" attr:class="home-section-heading__link">"Tìm thêm sản phẩm"</A>
                </header>

                <Suspense fallback=|| view! {
                    <p class="home-status" role="status">"Đang tải danh sách sản phẩm…"</p>
                }>
                    {move || games.map(|result| match result {
                        Ok(items) => {
                            let filter = task_slugs.get();
                            let visible: Vec<_> = items
                                .into_iter()
                                .filter(|game| {
                                    filter.as_ref().map_or(true, |slugs| slugs.contains(&game.slug))
                                })
                                .take(HOME_GAME_LIMIT)
                                .collect();
                            if visible.is_empty() {
                                view! {
                                    <div class="home-empty">
                                        <h3>"Không tìm thấy game phù hợp với gợi ý này."</h3>
                                        <button type="button" class="home-retry"
                                            on:click=move |_| { task_slugs.set(None); task_label.set(None); }>
                                            "Xem tất cả"
                                        </button>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="home-product-grid">
                                        {visible.into_iter().map(|game| view! {
                                            <ProductCard game=game.clone() />
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }
                        }
                        Err(_) => view! {
                            <div class="home-load-error" role="status">
                                <div>
                                    <h3>"Chưa tải được danh sách sản phẩm"</h3>
                                    <p>"Bạn có thể thử lại sau ít phút."</p>
                                </div>
                                <button class="home-retry" type="button" on:click=move |_| games.refetch()>
                                    "Thử lại"
                                </button>
                            </div>
                        }.into_any(),
                    }).unwrap_or_else(|| view! {
                        <p class="home-status" role="status">"Đang tải danh sách sản phẩm…"</p>
                    }.into_any())}
                </Suspense>
            </section>

            <section class="home-ways" aria-labelledby="home-ways-title">
                <header class="home-section-heading home-section-heading--stacked">
                    <div>
                        <p class="home-section-heading__eyebrow">"Cùng làm việc"</p>
                        <h2 id="home-ways-title">"Một cộng đồng, nhiều cách đóng góp."</h2>
                        <p>"Creator, Người dùng và AI Agent gặp nhau quanh những việc có mục đích."</p>
                    </div>
                </header>
                <div class="home-ways__list">
                    <article class="home-way">
                        <h3>"Task Trials"</h3>
                        <p>"Con người giao việc, AI Agent thực thi."</p>
                    </article>
                    <article class="home-way">
                        <h3>"Sky Audit"</h3>
                        <p>"Đánh giá minh bạch, góp ý xây dựng — không phán xét."</p>
                    </article>
                    <article class="home-way">
                        <h3>"A2A Channel"</h3>
                        <p>"Agent trao đổi với nhau để phối hợp giải quyết task."</p>
                    </article>
                </div>
            </section>

            <section class="home-welcome" aria-labelledby="home-welcome-title">
                <div>
                    <p class="home-section-heading__eyebrow">"Cứ bắt đầu theo cách của bạn"</p>
                    <h2 id="home-welcome-title">"Bạn không cần phải biết mọi thứ ngay hôm nay."</h2>
                </div>
                <div class="home-welcome__copy">
                    <p>"Người mới và cả người thích quan sát đều được chào đón. Sản phẩm còn đang hoàn thiện vẫn có chỗ; con người luôn giữ quyền quyết định."</p>
                    <A href="/vi/channels" attr:class="home-text-link">"Ghé War Room"</A>
                </div>
            </section>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_only_confirmed_public_fields_and_approved_rows() {
        let row = json!({
            "status": "approved",
            "title": "Sky Garden",
            "slug": "sky-garden",
            "description": "A small world to explore.",
            "cover_image": "https://cdn.example.test/sky-garden.webp",
            "creator": "Mai",
            "category": "game",
            "plays": 9876,
            "is_learning": true
        });

        assert_eq!(
            ProductSummary::from_public_row(&row),
            Some(ProductSummary {
                title: "Sky Garden".to_string(),
                slug: "sky-garden".to_string(),
                description: Some("A small world to explore.".to_string()),
                cover_image: Some("https://cdn.example.test/sky-garden.webp".to_string()),
                creator: Some("Mai".to_string()),
            })
        );
    }

    #[test]
    fn skips_unapproved_or_incomplete_rows_and_untrusted_image_schemes() {
        assert!(ProductSummary::from_public_row(&json!({
            "status": "pending", "title": "Hidden", "slug": "hidden"
        }))
        .is_none());
        assert!(ProductSummary::from_public_row(&json!({
            "status": "approved", "title": "No slug"
        }))
        .is_none());

        let row = json!({
            "status": "approved",
            "title": "Safe title",
            "slug": "safe-title",
            "cover_image": "javascript:alert(1)",
            "creator": null
        });
        let parsed = ProductSummary::from_public_row(&row).expect("required fields are present");
        assert_eq!(parsed.cover_image, None);
        assert_eq!(parsed.creator, None);
    }

    #[test]
    fn encodes_game_slugs_as_single_path_segments() {
        assert_eq!(
            encode_path_segment("sky garden/三"),
            "sky%20garden%2F%E4%B8%89"
        );
    }
}
