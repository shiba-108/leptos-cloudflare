use crate::api::get_community_detail;
use crate::community::{CommunityComment, CommunityDetailStatus};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::{components::A, hooks::use_params_map};

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

/// Read-only public post detail for `/vi/community/:id`.
#[component]
pub fn CommunityDetailPage() -> impl IntoView {
    let params = use_params_map();
    let raw_id = move || params.get().get("id").unwrap_or_default();
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

    let detail = Resource::new(
        move || raw_id(),
        |raw_id| async move { get_community_detail(raw_id, 1).await },
    );

    let additional_comments = RwSignal::new(Vec::<CommunityComment>::new());
    let current_comment_page = RwSignal::new(1u32);
    let has_more_override = RwSignal::new(None::<bool>);
    let loading_more = RwSignal::new(false);
    let load_more_failed = RwSignal::new(false);

    let load_more = move |_| {
        if loading_more.get_untracked() {
            return;
        }
        loading_more.set(true);
        load_more_failed.set(false);
        let requested_id = raw_id();
        let next_page = current_comment_page.get_untracked().saturating_add(1);

        leptos::task::spawn_local(async move {
            match get_community_detail(requested_id, next_page).await {
                Ok(result) if result.status == CommunityDetailStatus::Loaded => {
                    let mut known_ids = additional_comments
                        .get_untracked()
                        .iter()
                        .map(|comment| comment.id)
                        .collect::<std::collections::HashSet<_>>();
                    additional_comments.update(|loaded| {
                        loaded.extend(
                            result
                                .comments
                                .into_iter()
                                .filter(|comment| comment.id > 0 && known_ids.insert(comment.id)),
                        );
                    });
                    current_comment_page.set(result.comment_page.max(next_page));
                    has_more_override.set(Some(result.has_more_comments));
                }
                _ => load_more_failed.set(true),
            }
            loading_more.set(false);
        });
    };

    view! {
        <div class="page-shell page-shell--compact section-stack section-stack--related community-detail">
            <header class="community-detail__navigation">
                <A href="/vi/community" attr:class="community-detail__back">
                    "← Quay lại Cộng đồng"
                </A>
            </header>

            <Suspense fallback=|| view! {
                <section class="community-detail__state" role="status" aria-live="polite">
                    <span class="community-detail__state-mark" aria-hidden="true"></span>
                    <p>"Đang tải bài viết cộng đồng…"</p>
                </section>
            }>
                {move || match detail.get() {
                    None => view! {
                        <section class="community-detail__state" role="status" aria-live="polite">
                            <span class="community-detail__state-mark" aria-hidden="true"></span>
                            <p>"Đang tải bài viết cộng đồng…"</p>
                        </section>
                    }.into_any(),
                    Some(Err(_)) => {
                        set_page_status(&page_response, 502);
                        view! {
                            <section class="community-detail__state community-detail__state--error" role="alert">
                                <h1>"Tạm thời chưa tải được bài viết"</h1>
                                <p>"Máy chủ Cộng đồng chưa phản hồi. Hãy thử tải lại."</p>
                                <button type="button" class="community-detail__retry" on:click=move |_| detail.refetch()>
                                    "Thử lại"
                                </button>
                            </section>
                        }.into_any()
                    }
                    Some(Ok(result)) => match result.status {
                        CommunityDetailStatus::InvalidId => {
                            set_page_status(&page_response, 400);
                            view! {
                                <Title text="ID bài viết không hợp lệ — VionSky"/>
                                <Meta name="description" content="ID bài viết cộng đồng không hợp lệ."/>
                                <section class="community-detail__state community-detail__state--error" role="alert">
                                    <h1>"ID bài viết không hợp lệ"</h1>
                                    <p>"Đường dẫn cần một ID số nguyên dương."</p>
                                    <A href="/vi/community" attr:class="community-detail__retry">"Về Cộng đồng"</A>
                                </section>
                            }.into_any()
                        }
                        CommunityDetailStatus::NotFound => {
                            set_page_status(&page_response, 404);
                            view! {
                                <Title text="Không tìm thấy bài viết — VionSky"/>
                                <Meta name="description" content="Bài viết cộng đồng này không tồn tại hoặc đã được gỡ."/>
                                <section class="community-detail__state community-detail__state--error" role="status">
                                    <h1>"Không tìm thấy bài viết"</h1>
                                    <p>"Bài viết này không tồn tại hoặc đã được gỡ khỏi Cộng đồng."</p>
                                    <A href="/vi/community" attr:class="community-detail__retry">"Về Cộng đồng"</A>
                                </section>
                            }.into_any()
                        }
                        CommunityDetailStatus::Unavailable => {
                            set_page_status(&page_response, 502);
                            view! {
                                <Title text="Cộng đồng tạm thời gián đoạn — VionSky"/>
                                <Meta name="description" content="Không thể tải bài viết cộng đồng lúc này."/>
                                <section class="community-detail__state community-detail__state--error" role="alert">
                                    <h1>"Tạm thời chưa tải được bài viết"</h1>
                                    <p>"Máy chủ Cộng đồng chưa phản hồi. Hãy thử lại sau."</p>
                                    <button type="button" class="community-detail__retry" on:click=move |_| detail.refetch()>
                                        "Thử lại"
                                    </button>
                                </section>
                            }.into_any()
                        }
                        CommunityDetailStatus::Loaded => {
                            let Some(post) = result.post.clone() else {
                                set_page_status(&page_response, 502);
                                return view! {
                                    <section class="community-detail__state community-detail__state--error" role="alert">
                                        <h1>"Dữ liệu bài viết không đầy đủ"</h1>
                                        <p>"Hãy thử tải lại sau."</p>
                                    </section>
                                }.into_any();
                            };
                            set_page_status(&page_response, 200);
                            let description = meta_description(&post.content);
                            let initial_comments = result.comments.clone();
                            let initial_has_more = result.has_more_comments;
                            let post_comment_count = post.comment_count;

                            view! {
                                <Title text=format!("{} — VionSky", post.title)/>
                                <Meta name="description" content=description/>
                                <Meta property="og:type" content="article"/>
                                <Meta property="og:title" content=post.title.clone()/>
                                <Meta property="og:description" content=meta_description(&post.content)/>

                                <article class="community-detail__post">
                                    <header class="community-detail__post-header">
                                        <p class="community-detail__category">{post.category.clone()}</p>
                                        <h1>{post.title.clone()}</h1>
                                        <div class="community-detail__byline">
                                            <span>{post.user_name.clone()}</span>
                                            {if post.created_at.is_empty() {
                                                None
                                            } else {
                                                Some(view! {
                                                    <time datetime=post.created_at.clone()>{post.created_at.clone()}</time>
                                                })
                                            }}
                                        </div>
                                    </header>

                                    <div class="community-detail__body">
                                        <p>{post.content.clone()}</p>
                                    </div>

                                    <footer class="community-detail__post-footer">
                                        <dl class="community-detail__stats">
                                            <div><dt>"Bình luận"</dt><dd>{post.comment_count}</dd></div>
                                            <div><dt>"Lượt ủng hộ"</dt><dd>{post.upvote_count}</dd></div>
                                        </dl>
                                    </footer>
                                </article>

                                <section class="community-detail__comments" aria-labelledby="community-comments-title" aria-busy=move || loading_more.get()>
                                    <header class="community-detail__comments-heading">
                                        <h2 id="community-comments-title">"Bình luận"</h2>
                                        <span>{format!("{}", post_comment_count)}</span>
                                    </header>

                                    {if initial_comments.is_empty() && !initial_has_more {
                                        view! { <p class="community-detail__empty">"Chưa có bình luận. Bạn có thể là người đầu tiên tham gia."</p> }.into_any()
                                    } else {
                                        view! {
                                            <ol class="community-detail__comment-list">
                                                {move || {
                                                    let mut comments = initial_comments.clone();
                                                    let mut known_ids = comments.iter().map(|comment| comment.id).collect::<std::collections::HashSet<_>>();
                                                    comments.extend(additional_comments.get().into_iter().filter(|comment| known_ids.insert(comment.id)));
                                                    comments.into_iter().map(|comment| view! { <CommunityCommentView comment=comment/> }).collect_view()
                                                }}
                                            </ol>

                                            <Show when=move || has_more_override.get().unwrap_or(initial_has_more)>
                                                <div class="community-detail__more">
                                                    <button type="button" class="community-detail__load-more" disabled=move || loading_more.get() on:click=load_more>
                                                        {move || if loading_more.get() { "Đang tải…" } else { "Tải thêm bình luận" }}
                                                    </button>
                                                    <Show when=move || load_more_failed.get()>
                                                        <p class="community-detail__load-error" role="alert">"Không tải thêm được bình luận. Hãy thử lại."</p>
                                                    </Show>
                                                </div>
                                            </Show>
                                        }.into_any()
                                    }}
                                </section>
                            }.into_any()
                        }
                    },
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn CommunityCommentView(comment: CommunityComment) -> impl IntoView {
    let class = if comment.parent_id.is_some() {
        "community-detail__comment community-detail__comment--reply"
    } else {
        "community-detail__comment"
    };
    view! {
        <li class=class>
            <article>
                <header class="community-detail__comment-byline">
                    <strong>{comment.user_name}</strong>
                    {if comment.is_pinned {
                        Some(view! { <span class="community-detail__pinned">"Được ghim"</span> })
                    } else {
                        None
                    }}
                    {if comment.created_at.is_empty() {
                        None
                    } else {
                        let datetime = comment.created_at.clone();
                        let display_time = comment.created_at.clone();
                        Some(view! { <time datetime=datetime>{display_time}</time> })
                    }}
                </header>
                <p>{comment.content}</p>
            </article>
        </li>
    }
}

fn meta_description(content: &str) -> String {
    let compact = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        return "Bài viết công khai trong Cộng đồng VionSky.".to_owned();
    }
    let mut chars = compact.chars();
    let description = chars.by_ref().take(160).collect::<String>();
    if chars.next().is_some() {
        format!("{description}…")
    } else {
        description
    }
}
