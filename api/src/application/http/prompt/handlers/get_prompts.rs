use crate::application::auth::RequiredIdentity;
use crate::application::http::server::api_entities::api_error::ApiError;
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::extract::{Path, Query, State};
use ferriskey_core::domain::prompt::entities::prompt::Prompt;
use ferriskey_core::domain::prompt::ports::PromptService;
use ferriskey_core::domain::prompt::value_objects::GetPromptsFilter;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct GetPromptsQuery {
    #[schema(example = "My Prompt")]
    pub name: Option<String>,
    #[schema(example = "This is a description of my prompt")]
    pub description: Option<String>,
    #[schema(example = false)]
    pub include_deleted: Option<bool>,
    #[schema(example = 10)]
    pub limit: Option<u32>,
    #[schema(example = 10)]
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetPromptsResponse {
    pub data: Vec<Prompt>,
}

#[utoipa::path(
    get,
    path = "",
    tag = "prompt",
    summary = "Get prompts",
    description = "Retrieves all prompts in the system related to the current realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        GetPromptsQuery
    ),
    responses(
        (status = 200, body = GetPromptsResponse)
    ),
)]
pub async fn get_prompts(
    Path(realm_name): Path<String>,
    Query(query): Query<GetPromptsQuery>,
    State(state): State<AppState>,
    RequiredIdentity(identity): RequiredIdentity,
) -> Result<Response<GetPromptsResponse>, ApiError> {
    let prompts = state
        .service
        .get_prompts(
            identity,
            GetPromptsFilter {
                realm_name,
                name: query.name,
                description: query.description,
                include_deleted: query.include_deleted.unwrap_or(false),
                limit: query.limit,
                offset: query.offset,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetPromptsResponse { data: prompts }))
}
