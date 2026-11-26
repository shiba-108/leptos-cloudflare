use leptos::prelude::*;

use crate::components::home_page::ProductSummary;

#[cfg(feature = "ssr")]
use crate::contact::ContactSubmissionInput;

pub use crate::auth::{AuthConfig, AuthResult};
pub use crate::contact_fields::ContactResponse;
pub use crate::creator::{CreatorGamesResult, FeedbackResult, MutationResult};
pub use crate::product::ProductDetailResult;
pub use crate::search::SearchOutcome;
pub use crate::todos::{normalize_todo_title, TodoItem, TodoUpdateOutcome, TodosResponse};
use leptos::server_fn::{
    codec::{GetUrl, Json},
    Http,
};

#[server(name = GetPublicGames, endpoint = "get_public_games")]
pub async fn get_public_games() -> Result<Vec<ProductSummary>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::games::get_public_games()).await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(
    name = SearchPublicContent,
    endpoint = "search_public_content",
    protocol = Http<GetUrl, Json>
)]
pub async fn search_public_content(query: String) -> Result<SearchOutcome, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::search::search_public_content(query)).await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = SearchTodos, endpoint = "search_todos")]
pub async fn search_todos(query: String, status: String) -> Result<TodosResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::todos::search_todos(query, status)).await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = UpdateTodo, endpoint = "update_todo")]
pub async fn update_todo(
    id: i64,
    title: String,
    completed: bool,
    expected_title: String,
    expected_completed: bool,
) -> Result<TodoUpdateOutcome, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::todos::update_todo(
            id,
            title,
            completed,
            expected_title,
            expected_completed,
        ))
        .await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(ListTodos)]
pub async fn list_todos() -> Result<TodosResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::todos::list_todos()).await
    }

    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(CreateTodo)]
pub async fn create_todo(title: String) -> Result<TodoItem, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::todos::create_todo(title)).await
    }

    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(ToggleTodo)]
pub async fn toggle_todo(id: i64) -> Result<TodoItem, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::todos::toggle_todo(id)).await
    }

    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(DeleteTodo)]
pub async fn delete_todo(id: i64) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::todos::delete_todo(id)).await
    }

    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(GetTodo)]
pub async fn get_todo(id: i64) -> Result<TodoItem, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::todos::get_todo(id)).await
    }

    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(
    name = GetCommunityDetail,
    endpoint = "community_detail",
    protocol = Http<GetUrl, Json>
)]
pub async fn get_community_detail(
    raw_id: String,
    comment_page: u32,
) -> Result<crate::community::CommunityDetailResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::community::fetch_community_detail(
            &raw_id,
            comment_page,
        ))
        .await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = GetCommunityIndex, endpoint = "community_index", protocol = Http<GetUrl, Json>)]
pub async fn get_community_index(
    page: u32,
    limit: u32,
    category: Option<String>,
    feed: String,
) -> Result<crate::community_index::CommunityIndexResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::community_index::fetch_community_index(
            page, limit, category, feed,
        ))
        .await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = CreateCommunityPost, endpoint = "community_post_create")]
pub async fn create_community_post(
    title: String,
    content: String,
    category: String,
    game_id: Option<i64>,
) -> Result<crate::community_index::CreatePostResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::community_index::create_community_post(
            crate::community_index::CreatePostInput {
                title,
                content,
                category,
                game_id,
            },
        ))
        .await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = ViskyChat, endpoint = "visky_chat")]
pub async fn visky_chat(
    message: String,
    session_id: Option<String>,
    history: Vec<crate::visky::ChatTurn>,
) -> Result<crate::visky::ViskyChatReply, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::visky::visky_chat(
            message, session_id, history,
        ))
        .await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(SubmitContact)]
pub async fn submit_contact(
    name: String,
    email: String,
    topic: String,
    message: String,
    website: String,
) -> Result<ContactResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::contact::submit_contact(
            ContactSubmissionInput {
                name,
                email,
                topic,
                message,
                website,
            },
        ))
        .await
    }

    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = GetCreatorGames, endpoint = "get_creator_games")]
pub async fn get_creator_games() -> Result<CreatorGamesResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::creator::fetch_creator_games(
            &crate::server::AppState::from_context()?,
        ))
        .await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = PatchCreatorGame, endpoint = "patch_creator_game")]
pub async fn patch_creator_game(
    game_id: i64,
    description: String,
    play_url: String,
) -> Result<MutationResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::creator::patch_creator_game(
            &crate::server::AppState::from_context()?,
            game_id,
            description,
            play_url,
        ))
        .await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = PostCreatorReAudit, endpoint = "post_creator_re_audit")]
pub async fn post_creator_re_audit(game_id: i64) -> Result<MutationResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::creator::post_re_audit(
            &crate::server::AppState::from_context()?,
            game_id,
        ))
        .await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = GetCreatorFeedback, endpoint = "get_creator_feedback")]
pub async fn get_creator_feedback() -> Result<FeedbackResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::creator::fetch_creator_feedback(
            &crate::server::AppState::from_context()?,
        ))
        .await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = GetFeedbackSettings, endpoint = "get_feedback_settings")]
pub async fn get_feedback_settings() -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::creator::fetch_feedback_settings(
            &crate::server::AppState::from_context()?,
        ))
        .await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = PatchFeedbackSettings, endpoint = "patch_feedback_settings")]
pub async fn patch_feedback_settings(
    notify_feedback: bool,
) -> Result<MutationResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::creator::patch_feedback_settings(
            &crate::server::AppState::from_context()?,
            notify_feedback,
        ))
        .await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = HandleCreatorFeedback, endpoint = "handle_creator_feedback")]
pub async fn handle_creator_feedback(
    kind: String,
    id: i64,
) -> Result<MutationResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::creator::post_handle_feedback(
            &crate::server::AppState::from_context()?,
            kind,
            id,
        ))
        .await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = GetProductDetail, endpoint = "product_detail", protocol = Http<GetUrl, Json>)]
pub async fn get_product_detail(slug: String) -> Result<ProductDetailResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::server::run(crate::server::products::fetch_product_detail(&slug)).await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = GetAuthConfig, endpoint = "auth_config")]
pub async fn get_auth_config() -> Result<AuthConfig, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let state = crate::server::AppState::from_context()?;
        crate::server::run(async move { crate::server::auth::auth_config(&state) }).await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = LeptosSignIn, endpoint = "leptos_sign_in")]
pub async fn leptos_sign_in(email: String, password: String) -> Result<AuthResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let state = crate::server::AppState::from_context()?;
        crate::server::run(crate::server::auth::sign_in(&state, email, password)).await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = LeptosSignOut, endpoint = "leptos_sign_out")]
pub async fn leptos_sign_out(access_token: String) -> Result<AuthResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let state = crate::server::AppState::from_context()?;
        crate::server::run(crate::server::auth::sign_out(&state, access_token)).await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}

#[server(name = LeptosRefreshSession, endpoint = "leptos_refresh_session")]
pub async fn leptos_refresh_session(refresh_token: String) -> Result<AuthResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let state = crate::server::AppState::from_context()?;
        crate::server::run(crate::server::auth::refresh_session(&state, refresh_token)).await
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server functions only execute on the server")
    }
}
