use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
};

use crate::application::{
    device_middleware::DeviceContext,
    http::server::{api_entities::api_error::ApiError, app_state::AppState},
};
use ferriskey_core::domain::{
    food_reaction::ports::FoodReactionRepository, realm::ports::RealmRepository,
};
use uuid::Uuid;

#[utoipa::path(
    delete,
    path = "/food-reactions/{reaction_id}",
    tag = "food-reaction",
    summary = "Delete food reaction",
    description = "Delete a food reaction",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("reaction_id" = Uuid, Path, description = "Reaction ID"),
    ),
    responses(
        (status = 204, description = "Reaction deleted successfully"),
        (status = 404, description = "Reaction not found")
    )
)]
pub async fn delete_reaction(
    Path((realm_name, reaction_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(device_context): Extension<DeviceContext>,
) -> Result<StatusCode, ApiError> {
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

    // Verify reaction exists
    let _reaction = state
        .reaction_repository
        .get_by_id(reaction_id, realm.id, device_context.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get food reaction: {}", e);
            ApiError::InternalServerError(format!("Failed to get food reaction: {}", e))
        })?
        .ok_or_else(|| ApiError::NotFound("Reaction not found".to_string()))?;

    // Delete reaction
    state
        .reaction_repository
        .delete_reaction(reaction_id, realm.id, device_context.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete food reaction: {}", e);
            ApiError::InternalServerError(format!("Failed to delete food reaction: {}", e))
        })?;

    Ok(StatusCode::NO_CONTENT)
}
