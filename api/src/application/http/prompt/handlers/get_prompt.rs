use crate::application::auth::RequiredIdentity;
use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::{Path, State};
use ferriskey_core::domain::prompt::entities::prompt::Prompt;
use ferriskey_core::domain::prompt::ports::PromptService;
use ferriskey_core::domain::prompt::value_objects::GetPromptInput;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/{prompt_id}",
    tag = "prompt",
    summary = "Get prompt",
    description = "Retrieves one prompt in the system related to the current realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("prompt_id" = Uuid, Path, description = "Prompt ID"),
    ),
    responses(
        (status = 200, body = Prompt)
    ),
)]
pub async fn get_prompt(
    Path(realm_name): Path<String>,
    Path(prompt_id): Path<Uuid>,
    State(state): State<AppState>,
    RequiredIdentity(identity): RequiredIdentity,
) -> Result<Response<Option<Prompt>>, ApiError> {
    let prompt = state
        .service
        .get_prompt(
            identity,
            GetPromptInput {
                realm_name,
                prompt_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(prompt))
}
