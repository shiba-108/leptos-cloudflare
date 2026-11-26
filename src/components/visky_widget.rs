// ============================================================================
use crate::api::visky_chat;
use crate::visky::{build_history, chat_error_key, ChatTurn, MAX_HISTORY_ROUNDS};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[cfg(not(feature = "ssr"))]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

#[cfg(not(feature = "ssr"))]
fn load_turns() -> Vec<ChatTurn> {
    let raw = storage()
        .and_then(|s| s.get_item("visky_history").ok().flatten())
        .unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

#[cfg(feature = "ssr")]
fn load_turns() -> Vec<ChatTurn> {
    Vec::new()
}

#[cfg(not(feature = "ssr"))]
fn load_session() -> Option<String> {
    storage().and_then(|s| s.get_item("visky_sessionId").ok().flatten())
}

#[cfg(feature = "ssr")]
fn load_session() -> Option<String> {
    None
}

fn persist(turns: &[ChatTurn], session: Option<&str>) {
    #[cfg(not(feature = "ssr"))]
    if let Some(store) = storage() {
        let saved: Vec<ChatTurn> = turns
            .iter()
            .filter(|turn| !turn.content.trim().is_empty())
            .cloned()
            .collect();
        if let Ok(json) = serde_json::to_string(&saved) {
            let _ = store.set_item("visky_history", &json);
        }
        match session {
            Some(id) => {
                let _ = store.set_item("visky_sessionId", id);
            }
            None => {
                let _ = store.remove_item("visky_sessionId");
            }
        }
    }
    #[cfg(feature = "ssr")]
    let _ = (turns, session);
}

/// Visky — trợ lý AI nổi góc dưới phải. Mặc định đóng; lịch sử lưu localStorage;
/// gọi server function `ViskyChat` (buffered, không SSE trực tiếp ở canary).
#[component]
pub fn ViskyWidget() -> impl IntoView {
    let open = RwSignal::new(false);
    let turns = RwSignal::new(load_turns());
    let input = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let session_id = RwSignal::new(load_session());

    let send = move || {
        let text = input.get().trim().to_owned();
        if text.is_empty() || busy.get() {
            return;
        }
        input.set(String::new());
        error.set(None);
        busy.set(true);
        let history = serde_json::to_string(&build_history(&turns.get(), MAX_HISTORY_ROUNDS))
            .unwrap_or_else(|_| "[]".to_string());
        turns.update(|items| {
            items.push(ChatTurn {
                role: "user".to_owned(),
                content: text.clone(),
            })
        });
        let sid = session_id.get();
        spawn_local(async move {
            match visky_chat(text, sid, history).await {
                Ok(reply) => {
                    if let Some(code) = reply.error {
                        error.set(Some(chat_error_key(&code).to_owned()));
                    } else if reply.text.trim().is_empty() {
                        error.set(Some("Chưa có phản hồi. Thử lại sau.".to_owned()));
                    } else {
                        if let Some(sid) = reply.session_id {
                            session_id.set(Some(sid));
                        }
                        turns.update(|items| {
                            items.push(ChatTurn {
                                role: "assistant".to_owned(),
                                content: reply.text,
                            })
                        });
                    }
                }
                Err(failure) => error.set(Some(failure.to_string())),
            }
            busy.set(false);
            persist(&turns.get(), session_id.get().as_deref());
        });
    };

    view! {
        <button
            type="button"
            class="visky-fab"
            aria-expanded=move || open.get()
            aria-label="Mở Visky AI"
            on:click=move |_| open.update(|value| *value = !*value)
        >
            "✦"
        </button>

        <Show when=move || open.get() fallback=|| ()>
            <div class="visky-dialog" role="dialog" aria-modal="true" aria-label="Visky AI">
                <header class="visky-dialog__head">
                    <strong>"Visky AI"</strong>
                    <button type="button" aria-label="Đóng"
                        on:click=move |_| open.set(false)>"✕"</button>
                </header>

                <div class="visky-log" role="log" aria-live="polite">
                    <Show when=move || turns.get().is_empty() && !busy.get() fallback=|| ()>
                        <p class="visky-empty">"Đặt câu hỏi để bắt đầu trò chuyện với Visky."</p>
                    </Show>
                    <For each=move || turns.get() key=|turn| turn.content.len() let:turn>
                        <div class=if turn.role == "user" { "visky-turn visky-turn--user" } else { "visky-turn visky-turn--assistant" }>
                            <div class="visky-bubble">{turn.content.clone()}</div>
                        </div>
                    </For>
                    <Show when=move || busy.get() fallback=|| ()>
                        <p class="visky-typing" role="status">"Đang suy nghĩ…"</p>
                    </Show>
                    {move || error.get().map(|message| view! {
                        <p class="visky-error" role="alert">{message}</p>
                    })}
                </div>

                <div class="visky-composer">
                    <input
                        maxlength="2000"
                        placeholder="Nhắn cho Visky…"
                        aria-label="Nhắn cho Visky"
                        prop:value=move || input.get()
                        on:input=move |event| input.set(leptos::tachys::dom::event_target_value(&event))
                    />
                    <button type="button" disabled=move || busy.get()
                        on:click=move |_| send()>"➤"</button>
                </div>
            </div>
        </Show>
    }
}
