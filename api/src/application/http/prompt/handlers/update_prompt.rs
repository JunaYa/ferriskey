use crate::application::auth::RequiredIdentity;
use crate::application::http::prompt::validators::UpdatePromptValidator;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::{Path, State};
use ferriskey_core::domain::prompt::entities::prompt::Prompt;
use ferriskey_core::domain::prompt::ports::PromptService;
use ferriskey_core::domain::prompt::value_objects::UpdatePromptInput;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdatePromptResponse {
    pub data: Prompt,
}

#[utoipa::path(
    put,
    path = "/{prompt_id}",
    tag = "prompt",
    summary = "Update prompt",
    description = "Updates an existing prompt in the system related to the current realm.",
    responses(
        (status = 200, body = UpdatePromptResponse)
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("prompt_id" = Uuid, Path, description = "Prompt ID"),
    ),
    request_body = UpdatePromptValidator
)]
pub async fn update_prompt(
    Path(realm_name): Path<String>,
    Path(prompt_id): Path<Uuid>,
    State(state): State<AppState>,
    RequiredIdentity(identity): RequiredIdentity,
    ValidateJson(payload): ValidateJson<UpdatePromptValidator>,
) -> Result<Response<UpdatePromptResponse>, ApiError> {
    let prompt = state
        .service
        .update_prompt(
            identity,
            UpdatePromptInput {
                realm_name,
                prompt_id,
                name: payload.name,
                description: payload.description,
                template: payload.template,
                version: payload.version,
                is_active: payload.is_active,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(UpdatePromptResponse { data: prompt }))
}
