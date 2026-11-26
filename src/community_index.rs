//! Wire types for the public Community index/feed (`GET /api/posts`).
//! Mirrors `ForumPost` / `ForumPostList` in vionsky `src/lib/forum.ts`.
//! Every optional/upstream-variable field uses `#[serde(default)]` so both the
//! Next.js envelope (`role` stripped) and the Rust-worker envelope parse.
use serde::{Deserialize, Serialize};

pub const POSTS_PAGE_SIZE: u32 = 10;
pub const MAX_PAGE_SIZE: u32 = 20;
pub const CATEGORIES: [&str; 4] = ["discussion", "showcase", "tutorial", "question"];

fn one() -> u32 {
    1
}
fn ten() -> u32 {
    POSTS_PAGE_SIZE
}
fn anon() -> String {
    "Người dùng ẩn danh".to_string()
}
fn discussion() -> String {
    "discussion".to_string()
}
fn latest() -> String {
    "latest".to_string()
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunityIndexStatus {
    Loaded,
    AuthRequired,
    #[default]
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CommunityFeedPost {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub user_id: String,
    #[serde(default = "anon")]
    pub user_name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content_preview: String, // server truncates to 400 chars
    #[serde(default = "discussion")]
    pub category: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub comment_count: u64,
    #[serde(default)]
    pub upvote_count: u64,
    #[serde(default)]
    pub is_upvoted: bool,
    #[serde(default)]
    pub role: String, // "" when Next.js GET strips it
    #[serde(default)]
    pub is_creator: bool,
    #[serde(default)]
    pub game_id: Option<i64>,
    #[serde(default)]
    pub game_title: Option<String>,
    #[serde(default)]
    pub game_slug: Option<String>,
    #[serde(default)]
    pub game_cover: Option<String>,
    #[serde(default)]
    pub game_rating: Option<f64>,
    #[serde(default)]
    pub via_ai_agent: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CommunityIndexResult {
    #[serde(default)]
    pub status: CommunityIndexStatus,
    #[serde(default)]
    pub posts: Vec<CommunityFeedPost>,
    #[serde(default = "one")]
    pub page: u32,
    #[serde(default = "ten")]
    pub limit: u32,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default = "latest")]
    pub feed: String,
    #[serde(default)]
    pub following_count: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CreatePostInput {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[serde(default = "discussion")]
    pub category: String,
    #[serde(default)]
    pub game_id: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CreatePostResult {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub status: CommunityIndexStatus,
}
