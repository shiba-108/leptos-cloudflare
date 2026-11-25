use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Read-only result states for the public community detail endpoint.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunityDetailStatus {
    /// The requested post and its comment page were loaded.
    Loaded,
    /// The route ID is not a positive integer.
    InvalidId,
    /// The API returned no post for a valid ID.
    NotFound,
    /// The public API failed or returned an unsupported response.
    Unavailable,
}

/// Minimal public post fields used by the Community detail page.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CommunityPost {
    /// Public numeric post identifier.
    pub id: i64,
    /// Display name returned by the public API.
    pub user_name: String,
    /// Post heading.
    pub title: String,
    /// User-authored post body, rendered as escaped text.
    pub content: String,
    /// Public category label.
    pub category: String,
    /// Creation timestamp supplied by the API.
    pub created_at: String,
    /// Last update timestamp supplied by the API.
    pub updated_at: String,
    /// Public total comment count.
    pub comment_count: u64,
    /// Public total upvote count.
    pub upvote_count: u64,
}

/// Minimal public comment fields used by the read-only detail page.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CommunityComment {
    /// Public numeric comment identifier.
    pub id: i64,
    /// Display name returned by the public API.
    pub user_name: String,
    /// User-authored comment body, rendered as escaped text.
    pub content: String,
    /// Creation timestamp supplied by the API.
    pub created_at: String,
    /// Parent comment when this record is a reply.
    pub parent_id: Option<i64>,
    /// Whether the post author pinned this comment.
    pub is_pinned: bool,
}

/// Public detail payload and pagination state from `/api/posts/{id}`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CommunityDetailResult {
    /// Load outcome used by the page to select a real HTTP status and state.
    pub status: CommunityDetailStatus,
    /// Post data, present only when `status` is `Loaded`.
    pub post: Option<CommunityPost>,
    /// Current page of public comments.
    pub comments: Vec<CommunityComment>,
    /// Normalized page number returned by the API.
    pub comment_page: u32,
    /// Normalized page size returned by the API.
    pub comment_limit: u32,
    /// Whether another public comment page is available.
    pub has_more_comments: bool,
}

impl CommunityDetailResult {
    fn empty(status: CommunityDetailStatus, page: u32) -> Self {
        Self {
            status,
            post: None,
            comments: Vec::new(),
            comment_page: page,
            comment_limit: 20,
            has_more_comments: false,
        }
    }
}

#[cfg(feature = "ssr")]
pub(crate) async fn fetch_community_detail(
    raw_id: &str,
    comment_page: u32,
) -> crate::server::AppResult<CommunityDetailResult> {
    use crate::server::AppError;

    let page = comment_page.max(1);
    let Some(id) = raw_id.parse::<i64>().ok().filter(|id| *id > 0) else {
        return Ok(CommunityDetailResult::empty(
            CommunityDetailStatus::InvalidId,
            page,
        ));
    };

    let url = format!("https://vionsky.com/api/posts/{id}?comment_page={page}&comment_limit=20");
    let mut request_init = worker::RequestInit::new();
    request_init.with_cache(worker::CacheMode::NoStore);
    request_init
        .headers
        .set("Accept", "application/json")
        .map_err(|_| AppError::Internal)?;
    request_init
        .headers
        .set("Cache-Control", "no-store")
        .map_err(|_| AppError::Internal)?;

    let request =
        worker::Request::new_with_init(&url, &request_init).map_err(|_| AppError::Internal)?;
    let mut response = worker::Fetch::Request(request)
        .send()
        .await
        .map_err(|_| AppError::Internal)?;

    match response.status_code() {
        400 => {
            return Ok(CommunityDetailResult::empty(
                CommunityDetailStatus::InvalidId,
                page,
            ));
        }
        404 => {
            return Ok(CommunityDetailResult::empty(
                CommunityDetailStatus::NotFound,
                page,
            ));
        }
        200 => {}
        _ => return Err(AppError::Internal),
    }

    const MAX_RESPONSE_BYTES: usize = 262_144;
    if response
        .headers()
        .get("content-length")
        .map_err(|_| AppError::Internal)?
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|length| length > MAX_RESPONSE_BYTES)
    {
        return Err(AppError::Internal);
    }

    let body = response.text().await.map_err(|_| AppError::Internal)?;
    if body.len() > MAX_RESPONSE_BYTES {
        return Err(AppError::Internal);
    }

    parse_public_payload(&body, id, page).ok_or(AppError::Internal)
}

fn parse_public_payload(
    body: &str,
    requested_id: i64,
    requested_page: u32,
) -> Option<CommunityDetailResult> {
    let root: Value = serde_json::from_str(body).ok()?;
    if root.get("success")?.as_bool()? != true {
        return None;
    }

    let data = root.get("data")?.as_object()?;
    let post_fields = data.get("post")?.as_object()?;
    let response_id = integer_field(post_fields.get("id")).unwrap_or(requested_id);
    if response_id != requested_id {
        return None;
    }

    let post = CommunityPost {
        id: response_id,
        user_name: string_field(post_fields, "user_name", "Người dùng ẩn danh"),
        title: string_field(post_fields, "title", ""),
        content: string_field(post_fields, "content", ""),
        category: string_field(post_fields, "category", "discussion"),
        created_at: string_field(post_fields, "created_at", ""),
        updated_at: string_field(post_fields, "updated_at", ""),
        comment_count: unsigned_field(post_fields.get("comment_count")).unwrap_or(0),
        upvote_count: unsigned_field(post_fields.get("upvote_count")).unwrap_or(0),
    };

    let comments = data
        .get("comments")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(parse_comment).collect())
        .unwrap_or_default();
    let comment_page = unsigned_field(data.get("comment_page"))
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .unwrap_or(requested_page);
    let comment_limit = unsigned_field(data.get("comment_limit"))
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .map(|value| value.min(20))
        .unwrap_or(20);

    Some(CommunityDetailResult {
        status: CommunityDetailStatus::Loaded,
        post: Some(post),
        comments,
        comment_page,
        comment_limit,
        has_more_comments: boolean_field(data.get("has_more_comments")).unwrap_or(false),
    })
}

fn parse_comment(value: &Value) -> Option<CommunityComment> {
    let fields = value.as_object()?;
    let id = integer_field(fields.get("id")).filter(|id| *id > 0)?;
    let parent_id = fields
        .get("parent_id")
        .filter(|value| !value.is_null())
        .and_then(|value| integer_field(Some(value)));

    Some(CommunityComment {
        id,
        user_name: string_field(fields, "user_name", "Người dùng ẩn danh"),
        content: string_field(fields, "content", ""),
        created_at: string_field(fields, "created_at", ""),
        parent_id,
        is_pinned: boolean_field(fields.get("is_pinned")).unwrap_or(false),
    })
}

fn string_field(fields: &Map<String, Value>, key: &str, fallback: &str) -> String {
    fields
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_owned()
}

fn integer_field(value: Option<&Value>) -> Option<i64> {
    let value = value?;
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|number| i64::try_from(number).ok()))
        .or_else(|| value.as_str().and_then(|number| number.parse::<i64>().ok()))
}

fn unsigned_field(value: Option<&Value>) -> Option<u64> {
    let value = value?;
    value
        .as_u64()
        .or_else(|| value.as_i64().and_then(|number| u64::try_from(number).ok()))
        .or_else(|| value.as_str().and_then(|number| number.parse::<u64>().ok()))
}

fn boolean_field(value: Option<&Value>) -> Option<bool> {
    let value = value?;
    value.as_bool().or_else(|| match value {
        Value::Number(number) => number.as_i64().map(|number| number != 0),
        Value::String(string) => match string.as_str() {
            "true" | "1" => Some(true),
            "false" | "0" => Some(false),
            _ => None,
        },
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_public_payload, CommunityDetailStatus};

    #[test]
    fn parses_the_public_detail_envelope_and_ignores_extra_production_fields() {
        let body = r#"{
          "success": true,
          "data": {
            "post": {
              "id": 42,
              "user_id": "not-rendered",
              "user_name": "A creator",
              "title": "A community post",
              "content": "<script>alert('not executed')</script>\nSecond line",
              "category": "showcase",
              "created_at": "2026-11-26T00:00:00.000Z",
              "updated_at": "2026-11-26T00:00:00.000Z",
              "comment_count": 1,
              "upvote_count": 3,
              "is_upvoted": true,
              "role": "creator",
              "is_creator": true,
              "future_field": {"ignored": true}
            },
            "comments": [{
              "id": 501,
              "post_id": 42,
              "user_id": "not-rendered",
              "user_name": "A commenter",
              "content": "A reply",
              "created_at": "2026-11-26T00:01:00.000Z",
              "parent_id": 500,
              "is_pinned": 1,
              "role": "user",
              "is_creator": false,
              "replies": [],
              "future_field": "ignored"
            }],
            "comment_page": 1,
            "comment_limit": 20,
            "has_more_comments": true,
            "future_payload_field": "ignored"
          }
        }"#;

        let result = parse_public_payload(body, 42, 1).expect("contract should parse");
        assert_eq!(result.status, CommunityDetailStatus::Loaded);
        assert_eq!(result.post.as_ref().expect("post").id, 42);
        assert_eq!(result.post.as_ref().expect("post").category, "showcase");
        assert_eq!(result.comments.len(), 1);
        assert_eq!(result.comments[0].parent_id, Some(500));
        assert!(result.comments[0].is_pinned);
        assert!(result.has_more_comments);
        assert_eq!(result.comment_limit, 20);
    }

    #[test]
    fn rejects_a_payload_for_a_different_post_id() {
        let body = r#"{"success":true,"data":{"post":{"id":43},"comments":[]}}"#;
        assert!(parse_public_payload(body, 42, 1).is_none());
    }
}
