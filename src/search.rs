use serde::{Deserialize, Serialize};

pub const MAX_SEARCH_RESULTS_PER_KIND: usize = 10;

/// Public product fields returned by `GET /api/search`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SearchProduct {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub cover_image: String,
    pub category: String,
    pub plays: i64,
    pub rating: f64,
    pub created_at: String,
}

/// Public community-post fields returned by `GET /api/search`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SearchPost {
    pub id: i64,
    pub title: String,
    pub content_preview: String,
    pub user_name: String,
    pub created_at: String,
    pub comment_count: i64,
}

/// The API contract returns up to ten products and ten posts per query.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SearchResponse {
    pub games: Vec<SearchProduct>,
    pub posts: Vec<SearchPost>,
}

/// Safe UI outcomes for the public search API; raw upstream error text is not
/// forwarded into the rendered page.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum SearchOutcome {
    Loaded(SearchResponse),
    RateLimited { retry_after_seconds: Option<u64> },
    Unavailable,
}

#[cfg(any(feature = "ssr", test))]
pub(crate) fn parse_search_response(body: &str) -> Option<SearchResponse> {
    let mut response = serde_json::from_str::<SearchResponse>(body).ok()?;
    response.games.truncate(MAX_SEARCH_RESULTS_PER_KIND);
    response.posts.truncate(MAX_SEARCH_RESULTS_PER_KIND);
    Some(response)
}

#[cfg(any(feature = "ssr", test))]
pub(crate) fn retry_after_seconds(body: &str, retry_after_header: Option<&str>) -> Option<u64> {
    let payload = serde_json::from_str::<serde_json::Value>(body).ok();
    if let Some(payload) = &payload {
        // Read the contract's `error` field before the optional `message` field.
        let _api_error = payload
            .get("error")
            .and_then(serde_json::Value::as_str)
            .or_else(|| payload.get("message").and_then(serde_json::Value::as_str));
        if let Some(seconds) = payload
            .get("retry_after_seconds")
            .and_then(serde_json::Value::as_u64)
            .filter(|seconds| *seconds > 0)
        {
            return Some(seconds);
        }
    }

    retry_after_header
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|seconds| *seconds > 0)
}

#[cfg(test)]
mod tests {
    use super::{parse_search_response, retry_after_seconds, MAX_SEARCH_RESULTS_PER_KIND};

    fn product(id: i64) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "title": format!("Product {id}"),
            "slug": format!("product-{id}"),
            "description": "A public product.",
            "cover_image": "https://cdn.example.test/product.webp",
            "category": "tool",
            "plays": 12,
            "rating": 4.5,
            "created_at": "2026-11-25T10:00:00Z"
        })
    }

    fn post(id: i64) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "title": format!("Post {id}"),
            "content_preview": "A public post.",
            "user_name": "Creator",
            "created_at": "2026-11-25T10:00:00Z",
            "comment_count": 3
        })
    }

    #[test]
    fn parses_contract_fields_and_ignores_future_fields() {
        let body = serde_json::json!({
            "games": [product(1)],
            "posts": [post(2)],
            "future_field": "ignored"
        })
        .to_string();

        let response = parse_search_response(&body).expect("contract response should parse");
        assert_eq!(response.games.len(), 1);
        assert_eq!(response.games[0].slug, "product-1");
        assert_eq!(response.posts.len(), 1);
        assert_eq!(response.posts[0].comment_count, 3);
    }

    #[test]
    fn rejects_missing_contract_arrays_or_wrong_field_types() {
        assert!(parse_search_response(r#"{"games":[]}"#).is_none());
        let invalid = serde_json::json!({
            "games": [{ "id": "not-an-integer" }],
            "posts": []
        })
        .to_string();
        assert!(parse_search_response(&invalid).is_none());
    }

    #[test]
    fn caps_each_result_list_at_the_contract_limit() {
        let games = (0..(MAX_SEARCH_RESULTS_PER_KIND + 2))
            .map(|id| product(id as i64 + 1))
            .collect::<Vec<_>>();
        let posts = (0..(MAX_SEARCH_RESULTS_PER_KIND + 1))
            .map(|id| post(id as i64 + 1))
            .collect::<Vec<_>>();
        let body = serde_json::json!({ "games": games, "posts": posts }).to_string();

        let response = parse_search_response(&body).expect("valid response");
        assert_eq!(response.games.len(), MAX_SEARCH_RESULTS_PER_KIND);
        assert_eq!(response.posts.len(), MAX_SEARCH_RESULTS_PER_KIND);
    }

    #[test]
    fn reads_retry_after_from_the_contract_error_body() {
        assert_eq!(
            retry_after_seconds(r#"{"error":"limited","retry_after_seconds":60}"#, None),
            Some(60)
        );
        assert_eq!(
            retry_after_seconds(r#"{"error":"quota unavailable"}"#, None),
            None
        );
        assert_eq!(retry_after_seconds("not-json", Some(" 45 ")), Some(45));
        assert_eq!(
            retry_after_seconds(r#"{"retry_after_seconds":60}"#, Some("10")),
            Some(60)
        );
        assert_eq!(retry_after_seconds("not-json", Some("0")), None);
    }
}
