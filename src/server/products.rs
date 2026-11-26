use serde::Deserialize;
use worker::D1Type;

use super::{d1::request_database, AppError, AppResult};
use crate::product::{ProductDetail, ProductDetailResult, ProductDetailStatus};

#[derive(Debug, Deserialize)]
struct ProductRow {
    id: i64,
    title: String,
    slug: String,
    description: Option<String>,
    cover_image: Option<String>,
    play_url: String,
    creator: Option<String>,
    tags: Option<String>,
    plays: Option<i64>,
    rating: Option<f64>,
    category: Option<String>,
    seo_description: Option<String>,
    average_rating: Option<f64>,
    review_count: Option<i64>,
}

/// Read one approved product by slug from D1 — the same query path used by the
/// Next.js `getApprovedGameBySlug`, with review aggregates added for JSON-LD.
pub async fn fetch_product_detail(raw_slug: &str) -> AppResult<ProductDetailResult> {
    let slug = raw_slug.trim();
    if !crate::product::valid_slug(slug) {
        return Ok(ProductDetailResult {
            status: ProductDetailStatus::InvalidSlug,
            product: None,
        });
    }

    let (db, _) = request_database()?;
    let slug_arg = D1Type::Text(slug);
    let row = db
        .prepare(
            "SELECT
                id, title, slug, description, cover_image, play_url, creator, tags,
                plays, rating, category, seo_description,
                (SELECT ROUND(AVG(r.rating), 1) FROM reviews r
                  WHERE r.game_id = games.id AND r.is_current = 1) AS average_rating,
                (SELECT COUNT(*) FROM reviews r
                  WHERE r.game_id = games.id AND r.is_current = 1) AS review_count
             FROM games
             WHERE slug = ?1 AND status = 'approved'
             LIMIT 1",
        )
        .bind_refs([&slug_arg])
        .map_err(|_| AppError::Internal)?
        .first::<ProductRow>(None)
        .await
        .map_err(|_| AppError::Internal)?;

    let Some(row) = row else {
        return Ok(ProductDetailResult {
            status: ProductDetailStatus::NotFound,
            product: None,
        });
    };

    Ok(ProductDetailResult {
        status: ProductDetailStatus::Loaded,
        product: Some(ProductDetail {
            id: row.id,
            title: row.title,
            slug: row.slug,
            description: row.description.unwrap_or_default(),
            cover_image: row.cover_image.unwrap_or_default(),
            play_url: row.play_url,
            creator: row.creator.unwrap_or_default(),
            tags: row.tags.unwrap_or_default(),
            plays: row.plays.unwrap_or(0),
            rating: row.rating.unwrap_or(0.0),
            category: row.category.unwrap_or_else(|| "other".to_string()),
            seo_description: row.seo_description.unwrap_or_default(),
            average_rating: row.average_rating,
            review_count: row.review_count.unwrap_or(0),
        }),
    })
}
