//! Shared Creator Dashboard wire types and presentation helpers.
//!
//! Ported from `src/app/[locale]/dashboard/page.tsx` and
//! `src/components/dashboard/CreatorFeedback.tsx`. The field names and
//! nullability follow `docs/API_CONTRACT.md` §3.3 exactly — do not rename them.

use serde::{Deserialize, Serialize};

pub const MAX_DAILY_AUDITS: i64 = 3;
const MAX_WARNINGS: usize = 5;
const WARNING_MESSAGE_CHARS: usize = 220;

/// 24 field wire row returned by `GET /api/creator/games`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreatorGame {
    pub id: i64,
    pub title: String,
    pub slug: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub cover_image: Option<String>,
    pub play_url: String,
    #[serde(default)]
    pub creator: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    pub status: String,
    #[serde(default)]
    pub plays: Option<i64>,
    #[serde(default)]
    pub rating: Option<f64>,
    pub created_at: String,
    #[serde(default)]
    pub ai_score: Option<f64>,
    #[serde(default)]
    pub ai_review: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub is_featured: Option<i64>,
    #[serde(default)]
    pub content_score: Option<f64>,
    #[serde(default)]
    pub tech_score: Option<f64>,
    #[serde(default)]
    pub is_safe: Option<i64>,
    #[serde(default)]
    pub last_audit_date: Option<String>,
    #[serde(default)]
    pub daily_audit_count: Option<i64>,
    /// JSON-encoded `Vec<AiWarning>`, not an array. Missing before migration 008.
    #[serde(default)]
    pub ai_warnings: Option<String>,
    #[serde(default)]
    pub average_rating: Option<f64>,
    #[serde(default)]
    pub review_count: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct CreatorGamesResult {
    pub games: Vec<CreatorGame>,
    pub signed_out: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MutationResult {
    pub ok: bool,
    pub error: Option<String>,
    #[serde(default)]
    pub re_review: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FeedbackItem {
    pub kind: String,
    pub id: i64,
    pub game_id: i64,
    pub game_title: String,
    pub game_slug: String,
    pub user_name: String,
    pub rating: Option<i64>,
    pub content: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DigestInfo {
    pub feedback_count: i64,
    pub warning_count: i64,
    pub warning_message: Option<String>,
    pub game_title: Option<String>,
    pub game_slug: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FeedbackResult {
    pub items: Vec<FeedbackItem>,
    pub digest: Option<DigestInfo>,
    pub signed_out: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiWarning {
    pub warning_type: String,
    pub message: String,
}

pub fn is_status_key(value: &str) -> bool {
    matches!(value, "approved" | "pending" | "rejected")
}

pub fn is_category_key(value: &str) -> bool {
    matches!(value, "game" | "tool" | "app" | "art" | "other" | "auto")
}

pub fn status_label(value: &str) -> String {
    match value {
        "approved" => "Đã duyệt".to_string(),
        "pending" => "Chờ duyệt".to_string(),
        "rejected" => "Từ chối".to_string(),
        other => other.to_string(),
    }
}

pub fn status_badge_class(value: &str) -> &'static str {
    match value {
        "approved" => "creator-badge--approved",
        "pending" => "creator-badge--pending",
        "rejected" => "creator-badge--rejected",
        _ => "creator-badge--neutral",
    }
}

pub fn category_label(value: Option<&str>) -> &'static str {
    match value {
        Some("game") => "Game",
        Some("tool") => "Công cụ",
        Some("app") => "Ứng dụng",
        Some("art") => "Nghệ thuật",
        Some("auto") => "Đang phân loại…",
        _ => "Khác",
    }
}

pub fn warning_type_label(value: &str) -> &'static str {
    match value {
        "security" => "Bảo mật",
        "seo" => "SEO",
        "tech" => "Kỹ thuật",
        _ => "Kỹ thuật",
    }
}

/// Normalize the JSON string stored in `ai_warnings` into at most five warnings.
///
/// Truncation uses Unicode scalar values, matching the existing Rust worker's
/// `truncate_graphemes` call site (`rust-workers/src/lib.rs:267-274`). This is a
/// deliberate deviation from the TypeScript grapheme-aware limit: both cap
/// user-visible text without splitting a scalar value.
pub fn parse_ai_warnings(raw: Option<&str>) -> Vec<AiWarning> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return Vec::new();
    };
    let Some(rows) = value.as_array() else {
        return Vec::new();
    };

    let mut seen = std::collections::HashSet::new();
    let mut warnings = Vec::new();
    for row in rows {
        let Some(object) = row.as_object() else {
            continue;
        };
        let message = object
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let message: String = message.chars().take(WARNING_MESSAGE_CHARS).collect();
        if message.is_empty() {
            continue;
        }

        let normalized_type = object
            .get("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        let warning_type = match normalized_type.as_str() {
            "security" | "seo" | "tech" => normalized_type,
            _ => "tech".to_string(),
        };

        let key = format!("{}:{}", warning_type, message.to_lowercase());
        if !seen.insert(key) {
            continue;
        }
        warnings.push(AiWarning {
            warning_type,
            message,
        });
        if warnings.len() == MAX_WARNINGS {
            break;
        }
    }
    warnings
}

pub fn format_count(value: i64, locale: &str) -> String {
    let separator = if locale == "en" { ',' } else { '.' };
    let digits = value.abs().to_string();
    let chars: Vec<char> = digits.chars().collect();
    let mut output = String::new();
    for (index, character) in chars.iter().enumerate() {
        if index > 0 && (chars.len() - index) % 3 == 0 {
            output.push(separator);
        }
        output.push(*character);
    }
    if value < 0 {
        output.insert(0, '-');
    }
    output
}

pub fn format_score(value: Option<f64>) -> String {
    match value {
        Some(score) => format!("{:.1}/10", (score * 10.0).round() / 10.0),
        None => "—".to_string(),
    }
}

pub fn format_rating(value: Option<f64>) -> String {
    match value {
        Some(rating) => format!("{:.1} ★", (rating * 10.0).round() / 10.0),
        None => "—".to_string(),
    }
}

/// Format `YYYY-MM-DD HH:MM:SS`, ISO dates and plain dates as `dd/mm/yyyy`.
/// Unparseable values are returned unchanged, matching `formatDate` in the UI.
pub fn format_date(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "—".to_string();
    }

    let date_part = if trimmed.len() >= 10 {
        let bytes = trimmed.as_bytes();
        if bytes[4] == b'-' && bytes[7] == b'-' {
            &trimmed[..10]
        } else {
            return trimmed.to_string();
        }
    } else {
        return trimmed.to_string();
    };

    let digits: Vec<u8> = date_part
        .bytes()
        .filter(|byte| byte.is_ascii_digit())
        .collect();
    if digits.len() != 8 {
        return trimmed.to_string();
    }
    let year = format!(
        "{}{}{}{}",
        digits[0] as char, digits[1] as char, digits[2] as char, digits[3] as char
    );
    let month = (digits[4] - b'0') * 10 + (digits[5] - b'0');
    let day = (digits[6] - b'0') * 10 + (digits[7] - b'0');
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return trimmed.to_string();
    }
    format!("{day:02}/{month:02}/{year}")
}

pub fn product_path(slug: &str) -> String {
    format!("/product/{slug}/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn creator_game_accepts_the_documented_wire_shape() {
        let row = json!({
            "id": 42,
            "title": "Game",
            "slug": "game",
            "description": null,
            "cover_image": null,
            "play_url": "https://example.com",
            "creator": null,
            "tags": null,
            "status": "approved",
            "plays": 12,
            "rating": 4.5,
            "created_at": "2026-01-02 03:04:05",
            "ai_score": 8.0,
            "ai_review": "ok",
            "category": "game",
            "is_featured": 1,
            "content_score": 8.0,
            "tech_score": 9.0,
            "is_safe": 1,
            "last_audit_date": "2026-01-02",
            "daily_audit_count": 0,
            "ai_warnings": null,
            "average_rating": 4.5,
            "review_count": 2
        });
        let game: CreatorGame = serde_json::from_value(row).expect("documented row");
        assert_eq!(game.review_count, 2);
        assert_eq!(game.is_featured, Some(1));
        assert!(game.ai_warnings.is_none());
    }

    #[test]
    fn ai_warnings_tolerates_missing_malformed_and_unknown_values() {
        assert!(parse_ai_warnings(None).is_empty());
        assert!(parse_ai_warnings(Some("not json")).is_empty());
        assert!(parse_ai_warnings(Some("{}")).is_empty());

        let raw = r#"[
            {"type":"SECURITY ","message":"  insecure   script  "},
            {"type":"weird","message":"check csp"},
            {"type":"security","message":"INSECURE SCRIPT"},
            {"type":"seo","message":""},
            "not an object"
        ]"#;
        let warnings = parse_ai_warnings(Some(raw));
        assert_eq!(warnings.len(), 2);
        assert_eq!(warnings[0].warning_type, "security");
        assert_eq!(warnings[0].message, "insecure script");
        assert_eq!(warnings[1].warning_type, "tech");
        assert_eq!(warnings[1].message, "check csp");
    }

    #[test]
    fn ai_warnings_caps_at_five_items_and_220_characters() {
        let rows: Vec<String> = (0..7)
            .map(|index| format!(r#"{{"type":"seo","message":"m{index}"}}"#))
            .collect();
        let raw = format!("[{}]", rows.join(","));
        assert_eq!(parse_ai_warnings(Some(&raw)).len(), 5);

        let long = "a".repeat(300);
        let raw = format!(r#"[{{"type":"tech","message":"{long}"}}]"#);
        let warnings = parse_ai_warnings(Some(&raw));
        assert_eq!(warnings[0].message.chars().count(), 220);
    }

    #[test]
    fn status_and_category_keys_are_closed_sets() {
        assert!(is_status_key("approved"));
        assert!(!is_status_key("unknown"));
        assert!(is_category_key("auto"));
        assert!(!is_category_key("unknown"));
        assert_eq!(category_label(None), "Khác");
        assert_eq!(status_label("pending"), "Chờ duyệt");
        assert_eq!(status_label("mystery"), "mystery");
    }

    #[test]
    fn formatting_matches_the_dashboard_rules() {
        assert_eq!(format_count(1_234_567, "vi"), "1.234.567");
        assert_eq!(format_count(1_234_567, "en"), "1,234,567");
        assert_eq!(format_score(Some(8.25)), "8.3/10");
        assert_eq!(format_score(None), "—");
        assert_eq!(format_rating(Some(4.5)), "4.5 ★");
        assert_eq!(format_date("2026-01-02 03:04:05"), "02/01/2026");
        assert_eq!(format_date("2026-01-02T03:04:05Z"), "02/01/2026");
        assert_eq!(format_date("bad"), "bad");
        assert_eq!(format_date(""), "—");
        assert_eq!(product_path("my-game"), "/product/my-game/");
    }
}
