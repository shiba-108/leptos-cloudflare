//! Shared Visky chat contract. Pure helpers mirror Next.js
//! src/lib/visky/{sse,context,chat-client}.ts so both sides prune and parse alike.

use serde::{Deserialize, Serialize};

pub const MAX_MESSAGE_CHARS: usize = 2000;
pub const MAX_HISTORY_ROUNDS: usize = 12;
pub const MAX_HISTORY_ITEMS: usize = 40;
pub const MAX_CHARS_PER_TURN: usize = 600;
pub const MAX_SESSION_ID_CHARS: usize = 120;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ChatTurn {
    /// "user" | "assistant" — allowlist enforced in `parse_history`.
    pub role: String,
    pub content: String,
}

/// Buffered result returned by the `ViskyChat` server function. The Next.js
/// route streams SSE; a Leptos server fn can only return one serialized value,
/// so `src/server/visky.rs` folds the frames into this struct.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ViskyChatReply {
    pub text: String,
    pub session_id: Option<String>,
    /// Safe code: "rate_limited" | "upstream" | "bad_request" | "timeout".
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ViskyStreamEvent {
    Start,
    Chunk { content: String },
    ToolUse { name: Option<String> },
    Done { session_id: String },
    Error { code: String, message: String },
}

/// splitFrames (`sse.ts:31-35`): complete frames + unfinished tail.
pub fn split_frames(buffer: &str) -> (Vec<String>, String) {
    let parts: Vec<&str> = buffer.split("\n\n").collect();
    let tail = parts.last().copied().unwrap_or("").to_owned();
    let frames = parts[..parts.len().saturating_sub(1)]
        .iter()
        .map(|part| (*part).to_owned())
        .collect();
    (frames, tail)
}

/// parseFrame (`sse.ts:38-46`): first `data:` line, JSON payload.
pub fn parse_frame(frame: &str) -> Option<ViskyStreamEvent> {
    let line = frame.lines().find(|line| line.starts_with("data:"))?;
    let value: serde_json::Value = serde_json::from_str(line[5..].trim()).ok()?;
    let text = |key: &str| {
        value
            .get(key)
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_owned()
    };
    match value.get("type")?.as_str()? {
        "start" => Some(ViskyStreamEvent::Start),
        "chunk" => Some(ViskyStreamEvent::Chunk {
            content: text("content"),
        }),
        "tool_use" => Some(ViskyStreamEvent::ToolUse {
            name: value
                .get("name")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned),
        }),
        "done" => Some(ViskyStreamEvent::Done {
            session_id: text("sessionId"),
        }),
        "error" => Some(ViskyStreamEvent::Error {
            code: text("code"),
            message: text("message"),
        }),
        _ => None,
    }
}

/// Fold a full SSE body (including a final frame without `\n\n`) into a reply.
pub fn collect_stream(body: &str) -> ViskyChatReply {
    let mut text = String::new();
    let mut session_id = None;
    let mut error = None;
    for frame in body.split("\n\n") {
        match parse_frame(frame) {
            Some(ViskyStreamEvent::Chunk { content }) => text.push_str(&content),
            Some(ViskyStreamEvent::Done { session_id: sid }) => session_id = Some(sid),
            Some(ViskyStreamEvent::Error { .. }) => error = Some("upstream".to_owned()),
            _ => {}
        }
    }
    ViskyChatReply {
        text,
        session_id,
        error,
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrunedRound {
    pub user: String,
    pub assistant: String,
}

pub fn is_usable_reply(content: &str) -> bool {
    !content.trim().is_empty()
}

/// pruneContext (`context.ts:67-119`): only adjacent, non-empty [user,assistant]
/// pairs; keep the newest `max_rounds`.
pub fn prune_context(history: &[ChatTurn], max_rounds: usize) -> Vec<PrunedRound> {
    let mut rounds = Vec::new();
    let mut i = 0;
    while i < history.len() {
        if history[i].role != "user" {
            i += 1;
            continue;
        }
        let Some(reply) = history.get(i + 1) else {
            i += 1;
            continue;
        };
        if reply.role != "assistant" {
            i += 1;
            continue;
        }
        if !is_usable_reply(&history[i].content) || !is_usable_reply(&reply.content) {
            i += 2;
            continue;
        }
        rounds.push(PrunedRound {
            user: history[i].content.trim().to_owned(),
            assistant: reply.content.trim().to_owned(),
        });
        i += 2;
    }
    let stale = rounds.len().saturating_sub(max_rounds);
    if stale > 0 {
        rounds.drain(0..stale);
    }
    rounds
}

/// buildHistory (`chat-client.ts:134-145`): flatten pruned rounds to turns.
pub fn build_history(turns: &[ChatTurn], max_rounds: usize) -> Vec<ChatTurn> {
    prune_context(turns, max_rounds)
        .into_iter()
        .flat_map(|round| {
            [
                ChatTurn {
                    role: "user".to_owned(),
                    content: round.user,
                },
                ChatTurn {
                    role: "assistant".to_owned(),
                    content: round.assistant,
                },
            ]
        })
        .collect()
}

/// shouldReuseSession (`context.ts:181-190`).
pub fn should_reuse_session(
    session_id: Option<&str>,
    declared_turns: usize,
    kept_rounds: usize,
) -> bool {
    session_id.is_some_and(|id| !id.trim().is_empty()) && declared_turns == kept_rounds * 2
}

/// Safe, localized-key mapping (chat-client.ts:326-345). Never show upstream
/// raw text to the browser.
pub fn chat_error_key(code: &str) -> &'static str {
    match code {
        "rate_limited" => "errorRateLimited",
        "timeout" => "errorTimeout",
        "network" => "errorNetwork",
        "bad_request" => "errorBadRequest",
        _ => "errorGeneric",
    }
}

/// POST body validation codes from `chat/route.ts:111-134`.
pub fn validate_chat_input(message: &str) -> Result<(), &'static str> {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return Err("EMPTY_MESSAGE");
    }
    if trimmed.chars().count() > MAX_MESSAGE_CHARS {
        return Err("MESSAGE_TOO_LONG");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_frames_keeps_unfinished_tail() {
        let (frames, rest) = split_frames("data: {\"type\":\"start\"}\n\ndata: partial");
        assert_eq!(frames.len(), 1);
        assert_eq!(rest, "data: partial");
    }

    #[test]
    fn parse_frame_reads_all_visky_event_types() {
        assert_eq!(
            parse_frame("data: {\"type\":\"chunk\",\"content\":\"chào\"}"),
            Some(ViskyStreamEvent::Chunk {
                content: "chào".to_owned()
            })
        );
        assert_eq!(
            parse_frame("data: {\"type\":\"done\",\"sessionId\":\"visky-1\"}"),
            Some(ViskyStreamEvent::Done {
                session_id: "visky-1".to_owned()
            })
        );
        assert!(parse_frame("event: ping\ndata: not-json").is_none());
    }

    #[test]
    fn collect_stream_folds_chunks_and_session_without_trailing_separator() {
        let body = concat!(
            "data: {\"type\":\"start\"}\n\n",
            "data: {\"type\":\"chunk\",\"content\":\"Xin \"}\n\n",
            "data: {\"type\":\"chunk\",\"content\":\"chào\"}\n\n",
            "data: {\"type\":\"done\",\"sessionId\":\"s-9\"}"
        );
        assert_eq!(
            collect_stream(body),
            ViskyChatReply {
                text: "Xin chào".to_owned(),
                session_id: Some("s-9".to_owned()),
                error: None,
            }
        );
    }

    #[test]
    fn prune_context_drops_empty_and_orphan_rounds() {
        let history = vec![
            ChatTurn {
                role: "user".into(),
                content: "  ".into(),
            },
            ChatTurn {
                role: "assistant".into(),
                content: "answer".into(),
            },
            ChatTurn {
                role: "user".into(),
                content: "second".into(),
            },
            ChatTurn {
                role: "assistant".into(),
                content: "ok".into(),
            },
            ChatTurn {
                role: "user".into(),
                content: "pending".into(),
            },
        ];
        assert_eq!(
            prune_context(&history, 12),
            vec![PrunedRound {
                user: "second".to_owned(),
                assistant: "ok".to_owned()
            }]
        );
    }

    #[test]
    fn session_reuse_requires_exact_round_parity() {
        assert!(should_reuse_session(Some("s"), 4, 2));
        assert!(!should_reuse_session(Some("s"), 5, 2));
        assert!(!should_reuse_session(None, 4, 2));
    }
}
