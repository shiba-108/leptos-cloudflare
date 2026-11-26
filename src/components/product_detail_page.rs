//! `/vi/product/:slug` — SSR product detail with a full materialized SEO tag set.

use crate::api::get_product_detail;
use crate::product::{category_label, parse_tags, ProductDetailStatus};
use crate::seo::{product_seo, seo_view, truncate_description};
use leptos::prelude::*;
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

fn seo_description(
    seo_description: &str,
    description: &str,
    title: &str,
    category: &str,
) -> String {
    let placeholder = description
        .trim()
        .eq_ignore_ascii_case("sản phẩm số gửi qua vionsky")
        || description
            .trim()
            .eq_ignore_ascii_case("product submitted via vionsky");
    let raw = if !seo_description.trim().is_empty() {
        seo_description.trim()
    } else if placeholder {
        ""
    } else {
        description.trim()
    };
    let fallback = format!(
        "Khám phá {title} — {category} do AI tạo trên VIONSKY. Chơi, dùng thử, chia sẻ và đánh giá sản phẩm AI mỗi ngày!"
    );
    let enhanced = if raw.is_empty() {
        fallback
    } else if raw.chars().count() < 50 {
        format!("{raw} {fallback}")
    } else {
        raw.to_string()
    };
    truncate_description(&enhanced)
}

#[component]
pub fn ProductDetailPage() -> impl IntoView {
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
    let params = use_params_map();
    let raw_slug = move || params.get().get("slug").unwrap_or_default();
    let detail = Resource::new(move || raw_slug(), |slug| get_product_detail(slug));

    view! {
        <Suspense fallback=|| view! { <p class="product-detail__state" role="status">"Đang tải sản phẩm…"</p> }>
            {move || match detail.get() {
                None => view! { <p class="product-detail__state" role="status">"Đang tải sản phẩm…"</p> }.into_any(),
                Some(Err(_)) => {
                    set_page_status(&page_response, 502);
                    view! {
                        <section class="product-detail__state" role="alert">
                            <h1>"Không tải được sản phẩm"</h1>
                            <button type="button" on:click=move |_| detail.refetch()>"Thử lại"</button>
                        </section>
                    }.into_any()
                }
                Some(Ok(result)) => {
                    let slug = raw_slug();
                    match result.status {
                        ProductDetailStatus::InvalidSlug => {
                            set_page_status(&page_response, 400);
                            view! {
                                <section class="product-detail__state" role="alert">
                                    <h1>"Đường dẫn sản phẩm không hợp lệ"</h1>
                                    <A href="/vi/">"Về trang chủ"</A>
                                </section>
                            }.into_any()
                        }
                        ProductDetailStatus::NotFound => {
                            set_page_status(&page_response, 404);
                            view! {
                                <section class="product-detail__state" role="alert">
                                    <h1>"Sản phẩm không tồn tại"</h1>
                                    <p>"Trang sản phẩm bạn yêu cầu không tồn tại. Khám phá các sản phẩm AI khác trên VIONSKY."</p>
                                    <A href="/vi/">"Khám phá sản phẩm"</A>
                                </section>
                            }.into_any()
                        }
                        ProductDetailStatus::Unavailable => {
                            set_page_status(&page_response, 502);
                            view! { <p class="product-detail__state" role="alert">"Dịch vụ tạm thời không khả dụng."</p> }.into_any()
                        }
                        ProductDetailStatus::Loaded => {
                            let Some(product) = result.product else {
                                set_page_status(&page_response, 502);
                                return view! { <p class="product-detail__state" role="alert">"Dữ liệu sản phẩm không đầy đủ."</p> }.into_any();
                            };
                            set_page_status(&page_response, 200);
                            let description = seo_description(
                                &product.seo_description,
                                &product.description,
                                &product.title,
                                category_label(&product.category),
                            );
                            let tags = parse_tags(&product.tags);
                            let meta = product_seo(
                                &slug,
                                "vi",
                                &product.title,
                                &description,
                                Some(&product.cover_image),
                                &product.category,
                                &tags,
                                product.average_rating,
                                product.review_count,
                            );
                            let title = product.title.clone();
                            let cover = product.cover_image.clone();
                            let play_url = product.play_url.clone();
                            let creator = product.creator.clone();
                            let category_text = category_label(&product.category);
                            let tags_text = tags.join(", ");
                            view! {
                                {seo_view(meta)}
                                <div class="page-shell product-detail">
                                    <nav aria-label="Breadcrumb">
                                        <A href="/vi/">"VionSky"</A>" / "<span>{title.clone()}</span>
                                    </nav>
                                    <div class="product-detail__grid">
                                        <figure>
                                            <img src=cover alt=title.clone() loading="eager" decoding="async"/>
                                        </figure>
                                        <section>
                                            <p class="eyebrow">{category_text}</p>
                                            <h1>{title.clone()}</h1>
                                            <p>{description.clone()}</p>
                                            <dl>
                                                <div><dt>"Tác giả"</dt><dd>{creator}</dd></div>
                                                <div><dt>"Lượt chơi"</dt><dd>{product.plays}</dd></div>
                                                <div><dt>"Đánh giá"</dt><dd>{format!("{} / 5", product.rating)}</dd></div>
                                                {(!tags_text.is_empty()).then(|| view! { <div><dt>"Tags"</dt><dd>{tags_text.clone()}</dd></div> })}
                                            </dl>
                                            <a class="control-frame control-frame--primary" href=play_url rel="noopener noreferrer">
                                                "Chơi ngay"
                                            </a>
                                        </section>
                                    </div>
                                </div>
                            }.into_any()
                        }
                    }
                }
            }}
        </Suspense>
    }
}
