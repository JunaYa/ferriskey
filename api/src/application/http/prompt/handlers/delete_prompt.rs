use crate::application::auth::RequiredIdentity;
use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::{Path, State};
use ferriskey_core::domain::prompt::ports::PromptService;
use ferriskey_core::domain::prompt::value_objects::DeletePromptInput;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeletePromptResponse {
    pub message: String,
}

#[utoipa::path(
    delete,
    path = "/{prompt_id}",
    tag = "prompt",
    summary = "Delete prompt",
    description = "Soft deletes a prompt in the system related to the current realm.",
    responses(
        (status = 200, body = DeletePromptResponse)
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("prompt_id" = Uuid, Path, description = "Prompt ID"),
    ),
)]
pub async fn delete_prompt(
    Path(realm_name): Path<String>,
    Path(prompt_id): Path<Uuid>,
    State(state): State<AppState>,
    RequiredIdentity(identity): RequiredIdentity,
) -> Result<Response<DeletePromptResponse>, ApiError> {
    state
        .service
        .delete_prompt(
            identity,
            DeletePromptInput {
                realm_name,
                prompt_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(DeletePromptResponse {
        message: "Prompt deleted successfully".to_string(),
    }))
}
