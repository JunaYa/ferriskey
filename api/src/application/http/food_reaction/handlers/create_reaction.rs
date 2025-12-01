use axum::{
    Extension, Json,
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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateReactionRequest {
    pub analysis_item_id: Option<uuid::Uuid>,
    pub eaten_at: chrono::DateTime<chrono::Utc>,
    pub feeling: String,
    pub symptom_onset: String,
    pub symptoms: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CreateReactionResponse {
    pub id: uuid::Uuid,
    pub realm_id: uuid::Uuid,
    pub device_id: String,
    pub user_id: uuid::Uuid,
    pub analysis_item_id: Option<uuid::Uuid>,
    pub eaten_at: chrono::DateTime<chrono::Utc>,
    pub feeling: String,
    pub symptom_onset: String,
    pub notes: Option<String>,
    pub symptoms: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<FoodReaction> for CreateReactionResponse {
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
    post,
    path = "",
    tag = "food-reaction",
    summary = "Create food reaction",
    description = "Create a new food reaction record",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body = CreateReactionRequest,
    responses(
        (status = 201, body = CreateReactionResponse, description = "Reaction created successfully"),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn create_reaction(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    RequiredIdentity(identity): RequiredIdentity,
    Extension(device_context): Extension<DeviceContext>,
    Json(request): Json<CreateReactionRequest>,
) -> Result<Response<CreateReactionResponse>, ApiError> {
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

    // Create reaction
    let reaction = FoodReaction::new(
        ferriskey_core::domain::food_reaction::entities::FoodReactionConfig {
            realm_id: realm.id,
            device_id: device_context.device_id.clone(),
            user_id: device_context.user_id,
            analysis_item_id: request.analysis_item_id,
            eaten_at: request.eaten_at,
            feeling: request.feeling,
            symptom_onset: request.symptom_onset,
            notes: request.notes,
            symptoms: request.symptoms.clone(),
            created_by: identity.id(),
        },
    );

    let created = state
        .reaction_repository
        .create_reaction(reaction, request.symptoms)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create food reaction: {}", e);
            ApiError::InternalServerError(format!("Failed to create food reaction: {}", e))
        })?;

    Ok(Response::Created(CreateReactionResponse::from(created)))
}
