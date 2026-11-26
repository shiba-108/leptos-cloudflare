//! SEO metadata ported from `src/lib/seo.ts`, `seo-title.ts`,
//! `structured-data.ts` and the `generateMetadata` functions. Every tag set is
//! materialized per route: Leptos/leptos_meta has no Next.js-style shallow
//! metadata inheritance.

use leptos::prelude::*;
use leptos_meta::{Link as MetaLink, Meta, Title};
use serde_json::{json, Value};

pub const BASE_URL: &str = "https://vionsky.com";
pub const OG_IMAGE: &str = "https://vionsky.com/og-image.png";
pub const SITE_NAME: &str = "VIONSKY";
pub const TITLE_SUFFIX: &str = " — VIONSKY";
pub const TITLE_BUDGET: usize = 50;
pub const DESCRIPTION_LIMIT: usize = 160;

/// Port of `localeAlternates`: normalize to a trailing slash and always emit
/// vi, en and x-default (x-default points to the vi variant).
pub fn normalize_path(pathname: &str) -> String {
    if pathname.is_empty() || pathname == "/" {
        return "/".to_string();
    }
    if pathname.ends_with('/') {
        pathname.to_string()
    } else {
        format!("{pathname}/")
    }
}

pub fn canonical_url(pathname: &str, locale: &str) -> String {
    format!("{BASE_URL}/{locale}{}", normalize_path(pathname))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocaleAlternates {
    pub canonical: String,
    pub vi: String,
    pub en: String,
    pub x_default: String,
}

pub fn locale_alternates(pathname: &str, locale: &str) -> LocaleAlternates {
    let path = normalize_path(pathname);
    LocaleAlternates {
        canonical: format!("{BASE_URL}/{locale}{path}"),
        vi: format!("{BASE_URL}/vi{path}"),
        en: format!("{BASE_URL}/en{path}"),
        x_default: format!("{BASE_URL}/vi{path}"),
    }
}

/// `truncateSeoTitle`: 50-char budget before the suffix. Unicode scalar values
/// are used instead of grapheme clusters (no new dependency); Vietnamese text
/// in production is precomposed, so the emitted length matches the Next.js
/// audit for all current titles. Tests cover the ASCII boundary.
pub fn truncate_seo_title(title: &str) -> String {
    let mut chars: Vec<char> = title.chars().collect();
    if chars.len() > TITLE_BUDGET {
        chars.truncate(TITLE_BUDGET - 3);
        chars.extend("...".chars());
    }
    let mut result: String = chars.into_iter().collect();
    result.push_str(TITLE_SUFFIX);
    result
}

/// `truncateHtmlEscaped` approximation: cap at 160 Unicode scalar values. The
/// Next.js original caps at 160 HTML-escaped characters; our emitted strings
/// are escaped by leptos_meta when rendered, and current copy stays below the
/// limit, so no truncation difference appears in production.
pub fn truncate_description(value: &str) -> String {
    value.chars().take(DESCRIPTION_LIMIT).collect()
}

/// Port of `safeJsonLd`: escape in &, <, > order before injecting into a script.
pub fn safe_json_ld(value: &Value) -> String {
    serde_json::to_string(value)
        .unwrap_or_default()
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
}

pub fn site_graph_json_ld(locale: &str) -> String {
    safe_json_ld(&json!({
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "WebSite",
                "@id": "https://vionsky.com/#website",
                "name": SITE_NAME,
                "url": format!("{BASE_URL}/{locale}/"),
                "description": "Nền tảng khám phá và chia sẻ sản phẩm AI: game, tool, web app và art.",
                "inLanguage": locale,
                "potentialAction": {
                    "@type": "SearchAction",
                    "target": {
                        "@type": "EntryPoint",
                        "urlTemplate": format!("{BASE_URL}/{locale}/search/?q={{search_term_string}}")
                    },
                    "query-input": "required name=search_term_string"
                }
            },
            {
                "@type": "Organization",
                "@id": "https://vionsky.com/#organization",
                "name": SITE_NAME,
                "url": BASE_URL,
                "logo": {
                    "@type": "ImageObject",
                    "url": OG_IMAGE,
                    "width": 1200,
                    "height": 630
                },
                "image": OG_IMAGE
            }
        ]
    }))
}

pub fn web_page_json_ld(pathname: &str, locale: &str, name: &str, description: &str) -> String {
    safe_json_ld(&json!({
        "@context": "https://schema.org",
        "@type": "WebPage",
        "@id": format!("{}#webpage", canonical_url(pathname, locale)),
        "url": canonical_url(pathname, locale),
        "name": name,
        "description": description,
        "inLanguage": locale,
        "isPartOf": { "@id": "https://vionsky.com/#website" }
    }))
}

pub fn discussion_json_ld(
    post_id: &str,
    title: &str,
    description: &str,
    author: &str,
    date_published: &str,
    comment_count: i64,
) -> String {
    let url = canonical_url(&format!("/community/{post_id}"), "vi");
    safe_json_ld(&json!({
        "@context": "https://schema.org",
        "@type": "DiscussionForumPosting",
        "@id": url,
        "url": url,
        "mainEntityOfPage": { "@type": "WebPage", "@id": url },
        "headline": title.chars().take(110).collect::<String>(),
        "name": title,
        "description": description,
        "author": { "@type": "Person", "name": author },
        "publisher": { "@id": "https://vionsky.com/#organization" },
        "datePublished": date_published,
        "articleSection": "Community",
        "commentCount": comment_count,
        "inLanguage": "vi"
    }))
}

pub fn breadcrumb_json_ld(title: &str, locale: &str, pathname: &str) -> String {
    safe_json_ld(&json!({
        "@context": "https://schema.org",
        "@type": "BreadcrumbList",
        "itemListElement": [
            { "@type": "ListItem", "position": 1, "name": "VionSky", "item": format!("{BASE_URL}/{locale}/") },
            { "@type": "ListItem", "position": 2, "name": title, "item": canonical_url(pathname, locale) }
        ]
    }))
}

pub fn product_json_ld(
    slug: &str,
    locale: &str,
    title: &str,
    description: &str,
    cover_image: Option<&str>,
    category: &str,
    tags: &[String],
    rating: Option<f64>,
    review_count: i64,
) -> String {
    let url = canonical_url(&format!("/product/{slug}"), locale);
    let image = cover_image
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(OG_IMAGE);
    let schema_type = match category {
        "game" => "VideoGame",
        "tool" | "app" => "SoftwareApplication",
        "other" => "Product",
        _ => "CreativeWork",
    };
    let mut value = json!({
        "@context": "https://schema.org",
        "@type": schema_type,
        "@id": url,
        "name": title,
        "description": description,
        "url": url,
        "image": image,
        "inLanguage": locale,
        "publisher": { "@id": "https://vionsky.com/#organization" },
        "offers": {
            "@type": "Offer",
            "price": "0",
            "priceCurrency": "USD",
            "availability": "https://schema.org/InStock",
            "url": url
        }
    });
    if schema_type == "VideoGame" {
        value["playMode"] = json!("SinglePlayer");
        value["applicationCategory"] = json!("Game");
        value["operatingSystem"] = json!("Web");
    } else if schema_type == "SoftwareApplication" {
        value["applicationCategory"] = json!(if category == "app" {
            "WebApplication"
        } else {
            "UtilitiesApplication"
        });
        value["operatingSystem"] = json!("Web");
    } else if schema_type == "CreativeWork" {
        value["artform"] = json!("Digital");
    } else {
        value["category"] = json!(category);
    }
    if !tags.is_empty() {
        value["genre"] = json!(tags[0]);
        value["keywords"] = json!(tags.join(", "));
    }
    if let (Some(rating), true) = (rating.filter(|r| *r > 0.0), review_count > 0) {
        value["aggregateRating"] = json!({
            "@type": "AggregateRating",
            "ratingValue": (rating * 10.0).round() / 10.0,
            "reviewCount": review_count,
            "bestRating": 5,
            "worstRating": 1
        });
    }
    safe_json_ld(&value)
}

#[derive(Clone, Debug)]
pub struct PageSeo {
    pub locale: String,
    pub title: String,
    pub description: String,
    pub pathname: String,
    pub og_type: String,
    pub og_title: String,
    pub og_description: String,
    pub og_image: String,
    pub robots: String,
    pub json_ld: Vec<String>,
}

impl PageSeo {
    pub fn new(locale: &str, pathname: &str, title: &str, description: &str) -> Self {
        let alternates = locale_alternates(pathname, locale);
        let title = truncate_seo_title(title);
        let description = truncate_description(description);
        let og_title = title_from_og(&alternates.canonical, &description);
        Self {
            locale: locale.to_string(),
            description: description.clone(),
            title,
            pathname: alternates.canonical,
            og_type: "website".to_string(),
            og_title,
            og_description: description.clone(),
            og_image: OG_IMAGE.to_string(),
            robots: "index, follow".to_string(),
            json_ld: vec![site_graph_json_ld(locale)],
        }
    }

    pub fn with_article(mut self) -> Self {
        self.og_type = "article".to_string();
        self
    }

    pub fn with_noindex(mut self) -> Self {
        self.robots = "noindex, follow".to_string();
        self
    }

    pub fn with_json_ld(mut self, value: String) -> Self {
        self.json_ld.push(value);
        self
    }
}

/// Home uses the layout OG title verbatim (no truncated SEO title).
fn title_from_og(canonical: &str, _description: &str) -> String {
    if canonical.ends_with("/vi/") || canonical.ends_with("/en/") {
        "VIONSKY — AI Creations Platform".to_string()
    } else {
        "VionSky — Khám phá sản phẩm, kết nối cộng đồng".to_string()
    }
}

/// Build the full, materialized tag set for one route.
pub fn seo_view(meta: PageSeo) -> impl IntoView {
    // `pathname` on PageSeo is already an absolute canonical URL. Derive the
    // relative path again so hreflang URLs are not double-prefixed.
    let prefix = format!("{BASE_URL}/{}/", meta.locale);
    let relative_path = meta
        .pathname
        .strip_prefix(&prefix)
        .map(|value| format!("/{value}"))
        .unwrap_or_else(|| "/".to_string());
    let alternates = locale_alternates(&relative_path, &meta.locale);
    let canonical = meta.pathname;
    let scripts = meta
        .json_ld
        .into_iter()
        .map(|json| {
            view! {
                <script type="application/ld+json" inner_html=json></script>
            }
        })
        .collect_view();

    view! {
        <Title text=meta.title.clone()/>
        <Meta name="description" content=meta.description.clone()/>
        <Meta name="robots" content=meta.robots.clone()/>
        <MetaLink rel="canonical" href=canonical.clone()/>
        <MetaLink rel="alternate" hreflang="vi" href=alternates.vi.clone()/>
        <MetaLink rel="alternate" hreflang="en" href=alternates.en.clone()/>
        <MetaLink rel="alternate" hreflang="x-default" href=alternates.x_default.clone()/>
        <Meta property="og:type" content=meta.og_type.clone()/>
        <Meta property="og:site_name" content=SITE_NAME/>
        <Meta property="og:title" content=meta.og_title.clone()/>
        <Meta property="og:description" content=meta.og_description.clone()/>
        <Meta property="og:url" content=canonical.clone()/>
        <Meta property="og:image" content=meta.og_image.clone()/>
        <Meta name="twitter:card" content="summary_large_image"/>
        <Meta name="twitter:title" content=meta.og_title.clone()/>
        <Meta name="twitter:description" content=meta.og_description.clone()/>
        <Meta name="twitter:image" content=meta.og_image.clone()/>
        {scripts}
    }
}

pub fn home_seo(locale: &str) -> PageSeo {
    let mut meta = PageSeo::new(
        locale,
        "/",
        "VIONSKY — AI Creations Platform",
        "VIONSKY là nền tảng khám phá sản phẩm AI miễn phí: game, tool, web app và art do AI tạo. Chơi, dùng thử, chia sẻ và đánh giá sản phẩm AI mỗi ngày!",
    );
    // Home has no generateMetadata: layout's `default` is used verbatim.
    meta.title = "VIONSKY — AI Creations Platform".to_string();
    meta.og_title = "VIONSKY — AI Creations Platform".to_string();
    meta.og_description = "Khám phá game, tool, web app và art do AI tạo: chơi, dùng thử, chia sẻ và đánh giá trên VIONSKY.".to_string();
    meta
}

pub fn search_seo(locale: &str, query: &str) -> PageSeo {
    let query = query.trim();
    let (title, description): (String, String) = if query.is_empty() {
        if locale == "en" {
            (
                "Search AI Products".to_string(),
                "Search AI games, tools, web apps and art, plus community posts on VIONSKY. Find AI-generated products and discussions by keyword across the whole platform fast.".to_string(),
            )
        } else {
            (
                "Tìm kiếm sản phẩm AI".to_string(),
                "Tìm kiếm game, tool, web app và art do AI tạo, cùng bài viết cộng đồng trên VIONSKY. Tra cứu sản phẩm AI và thảo luận bằng từ khóa trên toàn nền tảng.".to_string(),
            )
        }
    } else if locale == "en" {
        (
            format!("Search: {query}"),
            format!("Search results for \"{query}\" on VIONSKY: AI products and community posts."),
        )
    } else {
        (
            format!("Tìm kiếm: {query}"),
            format!(
                "Kết quả tìm kiếm cho \"{query}\" trên VIONSKY: sản phẩm AI và bài viết cộng đồng."
            ),
        )
    };
    PageSeo::new(locale, "/search", &title, &description)
        .with_noindex()
        .with_json_ld(web_page_json_ld("/search", locale, &title, &description))
}

pub fn community_seo(
    post_id: &str,
    title: &str,
    description: &str,
    author: &str,
    date_published: &str,
    comment_count: i64,
) -> PageSeo {
    let path = format!("/community/{post_id}");
    PageSeo::new("vi", &path, title, description)
        .with_article()
        .with_json_ld(discussion_json_ld(
            post_id,
            title,
            description,
            author,
            date_published,
            comment_count,
        ))
}

pub fn community_index_seo(locale: &str) -> PageSeo {
    let (title, description) = if locale == "en" {
        (
            "Community — AI Creativity Forum",
            "Join the VIONSKY community to discuss AI-made games, tools, web apps and art. Share ideas, showcase products, ask questions and give feedback with other creators.",
        )
    } else {
        (
            "Cộng đồng — Diễn đàn AI Sáng tạo",
            "Tham gia cộng đồng VIONSKY để thảo luận về game, tool, web app và art do AI tạo. Chia sẻ ý tưởng, khoe sản phẩm, hỏi đáp và góp ý cùng các nhà sáng tạo khác.",
        )
    };
    PageSeo::new(locale, "/community", title, description).with_json_ld(web_page_json_ld(
        "/community",
        locale,
        title,
        description,
    ))
}

pub fn product_seo(
    slug: &str,
    locale: &str,
    title: &str,
    description: &str,
    cover_image: Option<&str>,
    category: &str,
    tags: &[String],
    rating: Option<f64>,
    review_count: i64,
) -> PageSeo {
    let path = format!("/product/{slug}");
    let mut meta = PageSeo::new(locale, &path, title, description);
    meta.og_title = format!("{title} - {category} AI miễn phí");
    if let Some(cover) = cover_image.filter(|value| !value.trim().is_empty()) {
        meta.og_image = if cover.starts_with("http") {
            cover.to_string()
        } else {
            format!("{BASE_URL}{cover}")
        };
    }
    meta.with_json_ld(product_json_ld(
        slug,
        locale,
        title,
        description,
        cover_image,
        category,
        tags,
        rating,
        review_count,
    ))
    .with_json_ld(breadcrumb_json_ld(title, locale, &path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternates_follow_nextjs_rules() {
        let a = locale_alternates("/search", "vi");
        assert_eq!(a.canonical, "https://vionsky.com/vi/search/");
        assert_eq!(a.vi, "https://vionsky.com/vi/search/");
        assert_eq!(a.en, "https://vionsky.com/en/search/");
        assert_eq!(a.x_default, "https://vionsky.com/vi/search/");
        assert_eq!(normalize_path("/"), "/");
    }

    #[test]
    fn json_ld_escapes_script_terminators() {
        let json = json!({ "name": "a</script><b>&</b>" });
        let safe = safe_json_ld(&json);
        assert!(!safe.contains("</script>"));
        assert!(safe.contains("\\u003c"));
        assert!(safe.contains("\\u003e"));
        assert!(safe.contains("\\u0026"));
    }

    #[test]
    fn title_budget_and_suffix() {
        let short = truncate_seo_title("Game hay");
        assert_eq!(short, "Game hay — VIONSKY");
        let long = truncate_seo_title(&"a".repeat(80));
        assert_eq!(long.chars().count(), 60);
        assert!(long.ends_with("... — VIONSKY"));
    }

    #[test]
    fn product_json_ld_is_typed_and_priced() {
        let json = product_json_ld(
            "my-game",
            "vi",
            "My Game",
            "Mô tả",
            None,
            "game",
            &["puzzle".to_string()],
            Some(4.5),
            3,
        );
        assert!(json.contains("VideoGame"));
        assert!(json.contains("AggregateRating"));
        assert!(json.contains("https://vionsky.com/vi/product/my-game/"));
        assert!(!json.contains("</script>"));
    }

    #[test]
    fn home_metadata_is_materialized() {
        let meta = home_seo("vi");
        assert_eq!(meta.title, "VIONSKY — AI Creations Platform");
        assert_eq!(meta.pathname, "https://vionsky.com/vi/");
        assert_eq!(meta.og_title, "VIONSKY — AI Creations Platform");
        assert!(meta.json_ld.iter().any(|json| json.contains("WebSite")));
        assert!(meta
            .json_ld
            .iter()
            .any(|json| json.contains("Organization")));
    }

    #[test]
    fn search_metadata_is_noindex_and_query_aware() {
        let meta = search_seo("vi", "game ai");
        assert_eq!(meta.robots, "noindex, follow");
        assert!(meta.title.contains("Tìm kiếm: game ai"));
        assert!(meta.description.contains("game ai"));
        assert!(meta.json_ld.iter().any(|json| json.contains("WebPage")));
    }
}
