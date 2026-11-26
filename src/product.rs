//! Wire types for the `/vi/product/:slug` SSR page.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProductDetailStatus {
    Loaded,
    InvalidSlug,
    NotFound,
    Unavailable,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ProductDetail {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub cover_image: String,
    pub play_url: String,
    pub creator: String,
    pub tags: String,
    pub plays: i64,
    pub rating: f64,
    pub category: String,
    pub seo_description: String,
    pub average_rating: Option<f64>,
    pub review_count: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProductDetailResult {
    pub status: ProductDetailStatus,
    pub product: Option<ProductDetail>,
}

pub fn valid_slug(slug: &str) -> bool {
    let trimmed = slug.trim();
    !trimmed.is_empty()
        && trimmed.len() <= 120
        && trimmed
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

const NOISE_TAGS: [&str; 5] = [
    "tự động",
    "tu dong",
    "auto",
    "đang phân loại...",
    "dang phan loai...",
];

/// Port of `parseTags`: drop emails and system-generated filler tags.
pub fn parse_tags(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .filter(|tag| {
            let looks_like_email = tag.contains('@') && tag.contains('.');
            !looks_like_email
                && !NOISE_TAGS
                    .iter()
                    .any(|noise| tag.eq_ignore_ascii_case(noise))
        })
        .map(str::to_owned)
        .collect()
}

pub fn category_label(category: &str) -> &'static str {
    match category {
        "game" => "Game",
        "tool" => "Công cụ",
        "app" => "Ứng dụng",
        "art" => "Nghệ thuật",
        _ => "Khác",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_validation_blocks_paths_and_empty_values() {
        assert!(valid_slug("my-game_2"));
        assert!(!valid_slug(""));
        assert!(!valid_slug("../admin"));
        assert!(!valid_slug("a/b"));
        assert!(!valid_slug(&"x".repeat(121)));
    }

    #[test]
    fn tags_drop_noise_and_email() {
        let tags = parse_tags("puzzle, auto, user@example.com, AI, ,");
        assert_eq!(tags, vec!["puzzle".to_string(), "AI".to_string()]);
    }
}
