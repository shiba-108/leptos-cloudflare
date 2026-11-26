//! Public Community feed for `/community` and `/vi/community`.
//! Ported from Next.js CommunityFeed: Latest/Following, category filters,
//! 'Đăng bài' composer and 'Xem thêm' pagination.
use crate::api::{create_community_post, get_community_index};
use crate::auth_client::auth_signal;
use crate::community_index::{CommunityIndexStatus, CATEGORIES};
use crate::seo::{community_index_seo, seo_view};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;

#[component]
pub fn CommunityIndexPage() -> impl IntoView {
    let session = auth_signal();
    let logged_in = move || session.get().is_some();
    let feed = RwSignal::new("latest".to_string());
    let category = RwSignal::new(None::<String>);
    let page = RwSignal::new(1u32);
    let composer_open = RwSignal::new(false);
    let submit_error = RwSignal::new(None::<String>);

    let active_feed = move || {
        if logged_in() {
            feed.get()
        } else {
            "latest".to_string()
        }
    };

    let resource = Resource::new_blocking(
        move || (active_feed(), category.get(), page.get()),
        move |(current_feed, current_category, current_page)| {
            get_community_index(current_page, 10, current_category, current_feed)
        },
    );

    let choose_feed = move |value: &'static str| {
        page.set(1);
        feed.set(value.to_string());
    };
    let choose_category = move |value: Option<String>| {
        page.set(1);
        category.set(value);
    };
    let reset = move || {
        page.set(1);
        feed.set("latest".to_string());
        category.set(None);
    };

    let post_title = RwSignal::new(String::new());
    let post_content = RwSignal::new(String::new());
    let post_category = RwSignal::new("discussion".to_string());
    let submit = move || {
        let title = post_title.get();
        let content = post_content.get();
        let chosen = post_category.get();
        submit_error.set(None);
        spawn_local(async move {
            match create_community_post(title, content, chosen, None).await {
                Ok(result) if result.success => {
                    composer_open.set(false);
                    post_title.set(String::new());
                    post_content.set(String::new());
                    reset();
                    resource.refetch();
                }
                Ok(result) => submit_error.set(Some(
                    result
                        .error
                        .unwrap_or_else(|| "Đăng bài thất bại.".to_string()),
                )),
                Err(failure) => submit_error.set(Some(failure.to_string())),
            }
        });
    };

    view! {
        {seo_view(community_index_seo("vi"))}
        <div class="community-shell">
            <header class="community-topbar">
                <div class="community-topbar__inner">
                    <A href="/vi/" attr:class="community-brand">"VIONSKY"</A>
                    <A href="/vi/search" attr:class="community-search-link">"🔍"</A>
                </div>
            </header>
            <main class="community-main">
                <section class="community-intro">
                    <p class="community-intro__eyebrow">"// cộng đồng"</p>
                    <h1 class="community-intro__title">"Cộng đồng VIONSKY"</h1>
                    <p class="community-intro__subtitle">
                        "Nơi các nhà sáng tạo và người chơi thảo luận về sản phẩm AI."
                    </p>
                </section>

                <Suspense fallback=|| view! { <p class="home-status" role="status">"Đang tải cộng đồng…"</p> }>
                <div class="feed-root">
                    <div class="feed-toolbar">
                        <span class="feed-count">
                            {move || format!("{} bài viết", resource.get().and_then(|r| r.ok()).map(|r| r.posts.len()).unwrap_or(0))}
                        </span>
                        <button
                            type="button"
                            class="feed-create-button"
                            on:click=move |_| {
                                if logged_in() { composer_open.set(true); }
                                else { let _ = leptos_router::hooks::use_navigate()("/vi/login", Default::default()); }
                            }
                        >
                            {move || if logged_in() { "+ Đăng bài" } else { "Đăng nhập để đăng" }}
                        </button>
                    </div>

                    <div role="tablist" attr:aria-label="Chọn nguồn bài viết">
                        <button type="button" role="tab"
                            attr:aria-selected=move || feed.get() == "latest"
                            on:click=move |_| choose_feed("latest")>"Mới nhất"</button>
                        <Show when=move || logged_in() fallback=|| ()>
                            <button type="button" role="tab"
                                attr:aria-selected=move || feed.get() == "following"
                                on:click=move |_| choose_feed("following")>"Đang theo dõi"</button>
                        </Show>
                    </div>

                    <div class="feed-tabs" role="tablist" attr:aria-label="Lọc theo chủ đề">
                        <button type="button" role="tab"
                            attr:aria-selected=move || category.get().is_none()
                            on:click=move |_| choose_category(None)>Tất cả</button>
                        {CATEGORIES.into_iter().map(|value| {
                            let label = match value {
                                "showcase" => "Khoe sản phẩm",
                                "tutorial" => "Hướng dẫn",
                                "question" => "Hỏi đáp",
                                _ => "Thảo luận",
                            };
                            view! {
                                <button type="button" role="tab"
                                    attr:aria-selected=move || category.get().as_deref() == Some(value)
                                    on:click=move |_| choose_category(Some(value.to_string()))>
                                    {label}
                                </button>
                            }
                        }).collect_view()}
                    </div>

                    {move || match resource.get().and_then(|r| r.ok()) {
                        None => view! {
                            <div role="status" attr:aria-busy="true">
                                <span class="sr-only">"Đang tải"</span>
                                {(0..3).map(|_| view! { <div class="feed-skeleton"></div> }).collect_view()}
                            </div>
                        }.into_any(),
                        Some(result) if result.status == CommunityIndexStatus::Loaded => {
                            if result.posts.is_empty() {
                                view! {
                                    <div class="feed-empty">
                                        <h3>"Chưa có bài viết nào"</h3>
                                        <p>"Hãy trở thành người đầu tiên chia sẻ ý tưởng của bạn trên VIONSKY."</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="feed-list">
                                        <For
                                            each=move || result.posts.clone()
                                            key=|post| post.id
                                            let:post
                                        >
                                            <article class="post-record">
                                                <span class="post-record__category">{post.category.clone()}</span>
                                                <A href=format!("/vi/community/{}", post.id)
                                                    attr:class="post-record__title">
                                                    {post.title.clone()}
                                                </A>
                                                <p class="post-record__preview">{post.content_preview.clone()}</p>
                                                <div class="post-record__meta">
                                                    <span>{post.user_name.clone()}</span>
                                                    <span>{format!("{} upvote", post.upvote_count)}</span>
                                                    <span>{format!("{} bình luận", post.comment_count)}</span>
                                                </div>
                                            </article>
                                        </For>
                                        <Show when=move || result.has_more fallback=|| ()>
                                            <button type="button"
                                                on:click=move |_| page.update(|value| *value += 1)>
                                                "Xem thêm bài viết"
                                            </button>
                                        </Show>
                                    </div>
                                }.into_any()
                            }
                        }
                        Some(_) => view! {
                            <div class="feed-error" role="alert">"Không tải được bài viết."</div>
                        }.into_any(),
                    }}
                </div>
                </Suspense>
            </main>
        </div>

        <Show when=move || composer_open.get() fallback=|| ()>
            <div class="composer-backdrop">
                <div class="composer-form">
                    <h2>"Đăng bài mới"</h2>
                    <label>
                        "Tiêu đề"
                        <input maxlength="60" prop:value=move || post_title.get()
                            on:input=move |event| post_title.set(leptos::tachys::dom::event_target_value(&event)) />
                    </label>
                    <label>
                        "Nội dung"
                        <textarea maxlength="5000" prop:value=move || post_content.get()
                            on:input=move |event| post_content.set(leptos::tachys::dom::event_target_value(&event)) />
                    </label>
                    <label>
                        "Chủ đề"
                        <select prop:value=move || post_category.get()
                            on:change=move |event| post_category.set(leptos::tachys::dom::event_target_value(&event))>
                            {CATEGORIES.into_iter().map(|value| view! {
                                <option value=value>{value}</option>
                            }).collect_view()}
                        </select>
                    </label>
                    {move || submit_error.get().map(|message| view! {
                        <p class="feed-error" role="alert">{message}</p>
                    })}
                    <div class="feed-toolbar">
                        <button type="button" on:click=move |_| composer_open.set(false)>"Hủy"</button>
                        <button type="button" on:click=move |_| submit()>"Đăng bài"</button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
