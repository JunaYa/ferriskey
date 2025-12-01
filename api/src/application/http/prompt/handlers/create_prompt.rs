use crate::application::auth::RequiredIdentity;
use crate::application::http::prompt::validators::CreatePromptValidator;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::{Path, State};
use ferriskey_core::domain::prompt::entities::prompt::Prompt;
use ferriskey_core::domain::prompt::ports::PromptService;
use ferriskey_core::domain::prompt::value_objects::CreatePromptInput;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CreatePromptResponse {
    pub data: Prompt,
}

#[utoipa::path(
    post,
    path = "",
    tag = "prompt",
    summary = "Create prompt",
    description = "Creates a new prompt in the system related to the current realm.",
    responses(
        (status = 200, body = CreatePromptResponse)
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body = CreatePromptValidator
)]
pub async fn create_prompt(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    RequiredIdentity(identity): RequiredIdentity,
    ValidateJson(payload): ValidateJson<CreatePromptValidator>,
) -> Result<Response<CreatePromptResponse>, ApiError> {
    let prompt = state
        .service
        .create_prompt(
            identity,
            CreatePromptInput {
                realm_name,
                name: payload.name,
                description: payload.description,
                template: payload.template,
                version: payload.version,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(CreatePromptResponse { data: prompt }))
}
