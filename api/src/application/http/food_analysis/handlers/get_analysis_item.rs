use axum::{
    Extension,
    extract::{Path, State},
};

use crate::application::{
    device_middleware::DeviceContext,
    http::server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};
use ferriskey_core::domain::{
    food_analysis::{entities::FoodAnalysisItem, ports::FoodAnalysisItemRepository},
    realm::ports::RealmRepository,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetAnalysisItemResponse {
    pub item: FoodAnalysisItem,
}

#[utoipa::path(
    get,
    path = "/items/{item_id}",
    tag = "food-analysis",
    summary = "Get a food analysis item",
    description = "Get a single food analysis item by ID",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("item_id" = Uuid, Path, description = "Item ID"),
    ),
    responses(
        (status = 200, body = GetAnalysisItemResponse),
        (status = 404, description = "Item not found")
    )
)]
pub async fn get_analysis_item(
    Path((realm_name, item_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(_device_context): Extension<DeviceContext>, // Required for device middleware
) -> Result<Response<GetAnalysisItemResponse>, ApiError> {
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

    // Get item
    let item = state
        .item_repository
        .get_by_id(item_id, realm.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get food analysis item: {}", e);
            ApiError::InternalServerError(format!("Failed to get food analysis item: {}", e))
        })?
        .ok_or_else(|| ApiError::NotFound("Item not found".to_string()))?;

    Ok(Response::OK(GetAnalysisItemResponse { item }))
}
