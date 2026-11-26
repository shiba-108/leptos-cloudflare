//! Creator Dashboard port: `src/app/[locale]/dashboard/page.tsx` +
//! `src/components/dashboard/CreatorFeedback.tsx`.
//!
//! Deviations from the Next.js original are deliberate and noted where they
//! occur: re-audit refreshes immediately instead of polling for 60s, and the
//! login button points at the production Supabase login because the Leptos
//! worker does not own an OAuth client.

use crate::api::{
    get_creator_feedback, get_creator_games, get_feedback_settings, handle_creator_feedback,
    patch_creator_game, patch_feedback_settings, post_creator_re_audit,
};
use crate::creator::{
    category_label, format_count, format_date, format_rating, format_score, is_category_key,
    is_status_key, parse_ai_warnings, product_path, status_badge_class, status_label,
    warning_type_label, CreatorGame, FeedbackItem, MAX_DAILY_AUDITS,
};
use leptos::{ev::SubmitEvent, prelude::*, task::spawn_local};
use leptos_meta::{Meta, Title};
use leptos_router::components::A;

#[component]
pub fn CreatorDashboardPage() -> impl IntoView {
    let games_resource = Resource::new_blocking(|| (), |_| get_creator_games());
    let feedback_resource = Resource::new_blocking(|| (), |_| get_creator_feedback());
    let settings_resource = Resource::new_blocking(|| (), |_| get_feedback_settings());

    let game_override = RwSignal::new(None::<Vec<CreatorGame>>);
    let feedback_override = RwSignal::new(None::<crate::creator::FeedbackResult>);
    let settings_override = RwSignal::new(None::<bool>);
    let toast = RwSignal::new(None::<(String, String)>);

    let editing = RwSignal::new(None::<CreatorGame>);
    let edit_description = RwSignal::new(String::new());
    let edit_play_url = RwSignal::new(String::new());
    let saving = RwSignal::new(false);
    let re_auditing = RwSignal::new(None::<i64>);
    let busy_feedback = RwSignal::new(None::<String>);
    let notify_saving = RwSignal::new(false);

    let games = move || {
        game_override.get().unwrap_or_else(|| {
            games_resource
                .get()
                .and_then(|result| result.ok())
                .map(|result| result.games)
                .unwrap_or_default()
        })
    };
    let feedback = move || {
        feedback_override.get().unwrap_or_else(|| {
            feedback_resource
                .get()
                .and_then(|result| result.ok())
                .unwrap_or_default()
        })
    };
    let notify = move || {
        settings_override.get().unwrap_or_else(|| {
            settings_resource
                .get()
                .and_then(|result| result.ok())
                .unwrap_or(true)
        })
    };
    let stats = move || {
        let list = games();
        (
            list.len(),
            list.iter().filter(|game| game.status == "approved").count(),
            list.iter().map(|game| game.plays.unwrap_or(0)).sum::<i64>(),
        )
    };

    let reload = move || {
        game_override.set(None);
        feedback_override.set(None);
        games_resource.refetch();
        feedback_resource.refetch();
    };

    let save_edit = move |event: SubmitEvent| {
        event.prevent_default();
        if saving.get_untracked() {
            return;
        }
        let Some(game) = editing.get_untracked() else {
            return;
        };
        saving.set(true);
        let id = game.id;
        let description = edit_description.get_untracked();
        let play_url = edit_play_url.get_untracked();
        spawn_local(async move {
            match patch_creator_game(id, description, play_url).await {
                Ok(result) if result.ok => {
                    editing.set(None);
                    let message = if result.re_review {
                        "Đã lưu. Link mới đã gửi AI chấm lại.".to_string()
                    } else {
                        "Đã lưu thay đổi.".to_string()
                    };
                    toast.set(Some(("success".to_string(), message)));
                    reload();
                }
                Ok(result) => toast.set(Some((
                    "error".to_string(),
                    result
                        .error
                        .unwrap_or_else(|| "Không lưu được thay đổi".to_string()),
                ))),
                Err(error) => toast.set(Some(("error".to_string(), error.to_string()))),
            }
            saving.set(false);
        });
    };

    let request_re_audit = move |game_id: i64| {
        if re_auditing.get_untracked().is_some() {
            return;
        }
        re_auditing.set(Some(game_id));
        toast.set(Some((
            "info".to_string(),
            "Đã gửi yêu cầu chấm lại. AI thường hoàn tất trong vài phút.".to_string(),
        )));
        spawn_local(async move {
            match post_creator_re_audit(game_id).await {
                Ok(result) if result.ok => {
                    game_override.set(None);
                    games_resource.refetch();
                    toast.set(Some((
                        "success".to_string(),
                        "AI đang chấm lại sản phẩm này.".to_string(),
                    )));
                }
                Ok(result) => toast.set(Some((
                    "error".to_string(),
                    result
                        .error
                        .unwrap_or_else(|| "Không gửi được yêu cầu".to_string()),
                ))),
                Err(error) => toast.set(Some(("error".to_string(), error.to_string()))),
            }
            re_auditing.set(None);
        });
    };

    let toggle_notify = move |_| {
        if notify_saving.get_untracked() {
            return;
        }
        let next = !notify();
        notify_saving.set(true);
        spawn_local(async move {
            match patch_feedback_settings(next).await {
                Ok(result) if result.ok => {
                    settings_override.set(Some(next));
                    toast.set(Some((
                        "info".to_string(),
                        if next {
                            "Đã bật thông báo góp ý.".to_string()
                        } else {
                            "Đã tắt thông báo góp ý.".to_string()
                        },
                    )));
                }
                Ok(result) => toast.set(Some((
                    "error".to_string(),
                    result
                        .error
                        .unwrap_or_else(|| "Không lưu được cài đặt".to_string()),
                ))),
                Err(error) => toast.set(Some(("error".to_string(), error.to_string()))),
            }
            notify_saving.set(false);
        });
    };

    let mark_handled = move |item: FeedbackItem| {
        let key = format!("{}:{}", item.kind, item.id);
        if busy_feedback.get_untracked().is_some() {
            return;
        }
        busy_feedback.set(Some(key.clone()));
        spawn_local(async move {
            match handle_creator_feedback(item.kind.clone(), item.id).await {
                Ok(result) if result.ok => {
                    feedback_override.update(|current| {
                        if let Some(current) = current {
                            current
                                .items
                                .retain(|row| row.kind != item.kind || row.id != item.id);
                        }
                    });
                    toast.set(Some((
                        "success".to_string(),
                        "Đã đánh dấu xử lý.".to_string(),
                    )));
                }
                Ok(result) => toast.set(Some((
                    "error".to_string(),
                    result
                        .error
                        .unwrap_or_else(|| "Không xử lý được góp ý".to_string()),
                ))),
                Err(error) => toast.set(Some(("error".to_string(), error.to_string()))),
            }
            busy_feedback.set(None);
        });
    };

    view! {
        <Title text="Bảng điều khiển nhà sáng tạo — VionSky" />
        <Meta
            name="description"
            content="Quản lý sản phẩm, xem góp ý và chấm lại sản phẩm bằng AI trên VionSky."
        />
        <div class="page-shell creator-dashboard">
            <header class="creator-dashboard__heading">
                <p class="eyebrow">"Dành cho nhà sáng tạo"</p>
                <h1>"Bảng điều khiển sản phẩm"</h1>
                <p class="page-lede">
                    "Theo dõi trạng thái duyệt, điểm AI và góp ý của người dùng cho mọi sản phẩm bạn đã gửi."
                </p>
                <A attr:class="control-frame control-frame--standard control-frame--primary" href="/submit/">
                    "Gửi sản phẩm mới"
                </A>
            </header>

            {move || {
                toast
                    .get()
                    .map(|(kind, message)| {
                        view! {
                            <p class=format!("creator-toast creator-toast--{kind}") role="status">
                                {message}
                            </p>
                        }
                    })
            }}

            <Suspense fallback=|| view! { <p class="creator-state" role="status">"Đang tải bảng điều khiển…"</p> }>
                {move || match games_resource.get() {
                    None => view! { <p class="creator-state" role="status">"Đang tải bảng điều khiển…"</p> }.into_any(),
                    Some(Ok(initial)) if initial.signed_out => view! {
                        <section class="creator-state" role="status">
                            <h2>"Vui lòng đăng nhập"</h2>
                            <p>"Bảng điều khiển cần phiên đăng nhập Supabase của bạn."</p>
                            <a class="control-frame control-frame--standard control-frame--primary"
                               href="https://vionsky.com/vi/auth/login/">
                                "Đăng nhập trên VionSky"
                            </a>
                        </section>
                    }.into_any(),
                    Some(Ok(initial)) if initial.error.is_some() => {
                        let message = initial.error.unwrap_or_default();
                        view! {
                            <section class="creator-state creator-state--error" role="alert">
                                <h2>"Không tải được sản phẩm"</h2>
                                <p>{message}</p>
                                <button type="button" class="control-frame control-frame--secondary"
                                        on:click=move |_| reload()>
                                    "Thử lại"
                                </button>
                            </section>
                        }.into_any()
                    }
                    Some(Ok(initial)) if initial.games.is_empty() => view! {
                        <section class="creator-state" role="status">
                            <h2>"Chưa có sản phẩm nào"</h2>
                            <p>"Gửi sản phẩm đầu tiên để bắt đầu nhận góp ý và chấm điểm AI."</p>
                            <A attr:class="control-frame control-frame--primary" href="/submit/">"Gửi ngay"</A>
                        </section>
                    }.into_any(),
                    _ => {
                        let (total, approved, plays) = stats();
                        view! {
                            <section class="creator-stats" aria-label="Tổng quan sản phẩm">
                                <article><span>"Sản phẩm"</span><strong>{format_count(total as i64, "vi")}</strong></article>
                                <article><span>"Đã duyệt"</span><strong>{format_count(approved as i64, "vi")}</strong></article>
                                <article><span>"Lượt chơi"</span><strong>{format_count(plays, "vi")}</strong></article>
                            </section>

                            <Show when=move || !games().is_empty() fallback=|| ()>
                                <section class="creator-feedback" id="feedback-checklist">
                                    <div class="creator-feedback__head">
                                        <h2>"Góp ý gần đây"</h2>
                                        <button
                                            type="button"
                                            role="switch"
                                            aria-checked=move || notify().to_string()
                                            disabled=move || notify_saving.get()
                                            class="creator-switch"
                                            on:click=toggle_notify
                                        >
                                            {move || if notify() { "Bật thông báo" } else { "Tắt thông báo" }}
                                        </button>
                                    </div>

                                    {move || {
                                        let result = feedback();
                                        match result.digest {
                                            Some(digest) if digest.feedback_count > 0 => view! {
                                                <p class="creator-digest">
                                                    {format!(
                                                        "{} góp ý mới · {} cảnh báo AI",
                                                        digest.feedback_count, digest.warning_count
                                                    )}
                                                    {digest.warning_message.unwrap_or_default()}
                                                </p>
                                            }.into_any(),
                                            _ => ().into_view().into_any(),
                                        }
                                    }}

                                    {move || {
                                        if feedback().items.is_empty() {
                                            view! { <p class="creator-state">"Không có góp ý cần xử lý."</p> }.into_any()
                                        } else {
                                            view! {
                                                <ul class="creator-feedback__list">
                                                    <For
                                                        each=move || feedback().items
                                                        key=|item| (item.kind.clone(), item.id)
                                                        children=move |item| {
                                                            let key = format!("{}:{}", item.kind, item.id);
                                                            let item_for_button = item.clone();
                                                            view! {
                                                                <li class="creator-feedback__item">
                                                                    <div class="creator-feedback__meta">
                                                                        <strong>{item.user_name}</strong>
                                                                        <span>{item.game_title}</span>
                                                                        <span>{if item.kind == "review" { "Đánh giá" } else { "Bình luận" }}</span>
                                                                        {item.rating.map(|rating| view! { <span>{format!("{rating}★")}</span> })}
                                                                        <time>{format_date(&item.created_at)}</time>
                                                                    </div>
                                                                    <p>{item.content}</p>
                                                                    <A href=product_path(&item.game_slug)>"Xem chi tiết"</A>
                                                                    <button
                                                                        type="button"
                                                                        class="control-frame control-frame--secondary"
                                                                        disabled=move || busy_feedback.get().is_some()
                                                                        on:click=move |_| mark_handled(item_for_button.clone())
                                                                    >
                                                                        "Đánh dấu đã xử lý"
                                                                    </button>
                                                                    {move || if busy_feedback.get().as_deref() == Some(&key) { "…" } else { "" }}
                                                                </li>
                                                            }
                                                        }
                                                    />
                                                </ul>
                                            }.into_any()
                                        }
                                    }}
                                </section>
                            </Show>

                            <div class="creator-grid">
                                <For
                                    each=move || games()
                                    key=|game| game.id
                                    children=move |game| {
                                        let game_for_edit = game.clone();
                                        let game_for_audit = game.clone();
                                        let warnings = parse_ai_warnings(game.ai_warnings.as_deref());
                                        let warning_count = warnings.len();
                                        let status_class = status_badge_class(&game.status);
                                        let status_text = if is_status_key(&game.status) {
                                            status_label(&game.status)
                                        } else {
                                            game.status.clone()
                                        };
                                        let category_text = category_label(
                                            game.category
                                                .as_deref()
                                                .filter(|value| is_category_key(value)),
                                        );
                                        let featured = game.is_featured == Some(1);
                                        let unsafe_game = game.is_safe == Some(0);
                                        let created = format_date(&game.created_at);
                                        let score = format_score(game.content_score);
                                        let tech = format_score(game.tech_score);
                                        let rating = format_rating(game.average_rating);
                                        let plays = format_count(game.plays.unwrap_or(0), "vi");
                                        let description = game.description.clone().unwrap_or_else(|| "Chưa có mô tả".to_string());
                                        let slug = format!("/{}", game.slug);
                                        let ai_review = game.ai_review.clone();
                                        let has_ai_review = ai_review.is_some();
                                        let rejected = game.status == "rejected";
                                        let audit_count = game.daily_audit_count.unwrap_or(0);
                                        view! {
                                            <article class="creator-card" id=format!("game-{}", game.id)>
                                                <div class="creator-card__top">
                                                    <img
                                                        src=game.cover_image.clone().unwrap_or_default()
                                                        alt=game.title.clone()
                                                        loading="lazy"
                                                        decoding="async"
                                                        width="800"
                                                        height="320"
                                                    />
                                                    <div class="creator-card__badges">
                                                        <span class=format!("creator-badge {status_class}")>{status_text}</span>
                                                        <span class="creator-badge creator-badge--neutral">{category_text}</span>
                                                        <Show when=move || featured fallback=|| ()>
                                                            <span class="creator-badge creator-badge--featured">"★ Nổi bật"</span>
                                                        </Show>
                                                        <Show when=move || unsafe_game fallback=|| ()>
                                                            <span class="creator-badge creator-badge--danger">"⛔ Không an toàn"</span>
                                                        </Show>
                                                    </div>
                                                </div>

                                                <h2>{game.title.clone()}</h2>
                                                <p class="creator-card__slug">{slug}</p>
                                                <p class="creator-card__description">{description}</p>

                                                <div class="creator-card__scores">
                                                    <span>"Nội dung: "{score}</span>
                                                    <span>"Kỹ thuật: "{tech}</span>
                                                    <span>{format!("Đánh giá: {} ({})", rating, game.review_count)}</span>
                                                    <span>{format!("Lượt chơi: {}", plays)}</span>
                                                </div>

                                                <Show when=move || warning_count != 0 fallback=|| ()>
                                                    <details class="creator-warnings">
                                                        <summary>{format!("⚠️ Cảnh báo AI ({})", warning_count)}</summary>
                                                        <ul>
                                                            {warnings
                                                                .clone()
                                                                .into_iter()
                                                                .map(|warning| view! {
                                                                    <li>
                                                                        <strong>{warning_type_label(&warning.warning_type)}</strong>
                                                                        ": " {warning.message}
                                                                    </li>
                                                                })
                                                                .collect_view()}
                                                        </ul>
                                                    </details>
                                                </Show>

                                                <Show when=move || has_ai_review fallback=|| ()>
                                                    <p class=if rejected { "creator-review creator-review--rejected" } else { "creator-review" }>
                                                        <strong>"AI review: "</strong>{ai_review.clone().unwrap_or_default()}
                                                    </p>
                                                </Show>

                                                <Show when=move || rejected fallback=|| ()>
                                                    <p class="creator-review--rejected">
                                                        "Sản phẩm bị từ chối. " <A href="/help/01-why-rejected/">"Xem lý do"</A>
                                                    </p>
                                                </Show>

                                                <p class="creator-card__date">"Gửi ngày "{created}</p>

                                                <div class="creator-card__actions">
                                                    <A attr:class="control-frame control-frame--secondary" href=product_path(&game.slug)>
                                                        "Xem trên VionSky"
                                                    </A>
                                                    <button
                                                        type="button"
                                                        class="control-frame control-frame--secondary"
                                                        on:click=move |_| {
                                                            edit_description.set(game_for_edit.description.clone().unwrap_or_default());
                                                            edit_play_url.set(game_for_edit.play_url.clone());
                                                            editing.set(Some(game_for_edit.clone()));
                                                        }
                                                    >
                                                        "Sửa"
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class="control-frame control-frame--primary"
                                                        disabled=move || re_auditing.get().is_some()
                                                        on:click=move |_| request_re_audit(game_for_audit.id)
                                                    >
                                                        {move || if re_auditing.get() == Some(game.id) { "Đang chấm…" } else { "Chấm lại AI" }}
                                                    </button>
                                                    <span class="creator-card__quota">
                                                        {format!("AI hôm nay: {audit_count}/{MAX_DAILY_AUDITS}")}
                                                    </span>
                                                </div>
                                            </article>
                                        }
                                    }
                                />
                            </div>

                            <Show when=move || editing.get().is_some() fallback=|| ()>
                                {move || {
                                    let game = editing.get().expect("editing game");
                                    view! {
                                        <div class="creator-modal" role="dialog" aria-modal="true">
                                            <form class="creator-modal__card" on:submit=save_edit>
                                                <h2>"Sửa "{game.title}</h2>
                                                <label for="creator-play-url">"Link chơi"</label>
                                                <input
                                                    id="creator-play-url"
                                                    type="url"
                                                    required
                                                    prop:value=move || edit_play_url.get()
                                                    on:input=move |event| edit_play_url.set(event_target_value(&event))
                                                />
                                                <label for="creator-description">"Mô tả"</label>
                                                <textarea
                                                    id="creator-description"
                                                    rows="5"
                                                    maxlength="2000"
                                                    prop:value=move || edit_description.get()
                                                    on:input=move |event| edit_description.set(event_target_value(&event))
                                                ></textarea>
                                                <div class="creator-modal__actions">
                                                    <button type="submit" class="control-frame control-frame--primary" disabled=move || saving.get()>
                                                        {move || if saving.get() { "Đang lưu…" } else { "Lưu thay đổi" }}
                                                    </button>
                                                    <button type="button" class="control-frame control-frame--secondary" disabled=move || saving.get()
                                                            on:click=move |_| editing.set(None)>
                                                        "Hủy"
                                                    </button>
                                                </div>
                                            </form>
                                        </div>
                                    }
                                }}
                            </Show>
                        }.into_any()
                    }
                }}
            </Suspense>
        </div>
    }
}
