use super::handlers::create_prompt::{__path_create_prompt, create_prompt};
use super::handlers::delete_prompt::{__path_delete_prompt, delete_prompt};
use super::handlers::get_prompt::{__path_get_prompt, get_prompt};
use super::handlers::get_prompts::{__path_get_prompts, get_prompts};
use super::handlers::update_prompt::{__path_update_prompt, update_prompt};
use crate::application::{auth::auth, http::server::app_state::AppState};

use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(get_prompts, get_prompt, create_prompt, update_prompt, delete_prompt))]
pub struct PromptApiDoc;

pub fn prompt_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/prompts",
                state.args.server.root_path
            ),
            get(get_prompts),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/prompts/{{prompt_id}}",
                state.args.server.root_path
            ),
            get(get_prompt),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/prompts",
                state.args.server.root_path
            ),
            post(create_prompt),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/prompts/{{prompt_id}}",
                state.args.server.root_path
            ),
            put(update_prompt),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/prompts/{{prompt_id}}",
                state.args.server.root_path
            ),
            delete(delete_prompt),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
