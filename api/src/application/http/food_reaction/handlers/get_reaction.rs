use axum::{
    Extension,
    extract::{Path, State},
};

use crate::application::{
    auth::RequiredIdentity,
    device_middleware::DeviceContext,
    http::server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};
use ferriskey_core::domain::{
    food_reaction::{entities::FoodReaction, ports::FoodReactionRepository},
    realm::ports::RealmRepository,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetReactionResponse {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub device_id: String,
    pub user_id: Uuid,
    pub analysis_item_id: Option<Uuid>,
    pub eaten_at: chrono::DateTime<chrono::Utc>,
    pub feeling: String,
    pub symptom_onset: String,
    pub notes: Option<String>,
    pub symptoms: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<FoodReaction> for GetReactionResponse {
    fn from(reaction: FoodReaction) -> Self {
        Self {
            id: reaction.id,
            realm_id: reaction.realm_id,
            device_id: reaction.device_id,
            user_id: reaction.user_id,
            analysis_item_id: reaction.analysis_item_id,
            eaten_at: reaction.eaten_at,
            feeling: reaction.feeling,
            symptom_onset: reaction.symptom_onset,
            notes: reaction.notes,
            symptoms: reaction.symptoms,
            created_at: reaction.created_at,
            updated_at: reaction.updated_at,
        }
    }
}

#[utoipa::path(
    get,
    path = "/{reaction_id}",
    tag = "food-reaction",
    summary = "Get a food reaction",
    description = "Get a single food reaction by ID",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("reaction_id" = Uuid, Path, description = "Reaction ID"),
    ),
    responses(
        (status = 200, body = GetReactionResponse),
        (status = 404, description = "Reaction not found")
    )
)]
pub async fn get_reaction(
    Path((realm_name, reaction_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    RequiredIdentity(_identity): RequiredIdentity,
    Extension(device_context): Extension<DeviceContext>,
) -> Result<Response<GetReactionResponse>, ApiError> {
    // Get realm
    let realm = state
        .realm_repository
        .get_by_name(realm_name.clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to get realm: {}", e);
            ApiError::InternalServerError(format!("Failed to get realm: {}", e))
        })?
        .ok_or_else(|| {
            tracing::error!("Realm not found: {}", realm_name);
            ApiError::NotFound(format!("Realm '{}' not found", realm_name))
        })?;

    // Get reaction
    let reaction = state
        .reaction_repository
        .get_by_id(reaction_id, realm.id, device_context.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get food reaction: {}", e);
            ApiError::InternalServerError(format!("Failed to get food reaction: {}", e))
        })?
        .ok_or_else(|| ApiError::NotFound("Reaction not found".to_string()))?;

    Ok(Response::OK(GetReactionResponse::from(reaction)))
}
