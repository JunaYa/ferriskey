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
    device_profile::ports::DeviceProfileRepository, realm::ports::RealmRepository,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeviceProfileResponse {
    pub id: uuid::Uuid,
    pub realm_id: uuid::Uuid,
    pub device_id: String,
    pub user_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ferriskey_core::domain::device_profile::entities::DeviceProfile>
    for DeviceProfileResponse
{
    fn from(profile: ferriskey_core::domain::device_profile::entities::DeviceProfile) -> Self {
        Self {
            id: profile.id,
            realm_id: profile.realm_id,
            device_id: profile.device_id,
            user_id: profile.user_id,
            created_at: profile.created_at,
            updated_at: profile.updated_at,
        }
    }
}

#[utoipa::path(
    get,
    path = "/{device_id}",
    tag = "device",
    summary = "Get device profile",
    description = "Retrieves a device profile by device_id. If the device doesn't exist, it will be automatically created with an anonymous user.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("device_id" = String, Path, description = "Device ID"),
    ),
    responses(
        (status = 200, body = DeviceProfileResponse, description = "Device profile retrieved or created successfully"),
        (status = 404, description = "Realm not found"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn get_device(
    Path((realm_name, device_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Extension(device_context): Extension<DeviceContext>,
) -> Result<Response<DeviceProfileResponse>, ApiError> {
    // Verify that the device_id in path matches the one in DeviceContext
    // (device_middleware already validated and created the device profile)
    if device_context.device_id != device_id {
        return Err(ApiError::BadRequest(
            "Device ID in path does not match X-Device-Id header".to_string(),
        ));
    }

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

    // Get device profile (device_middleware already created it if it didn't exist)
    let device_profile = state
        .device_profile_repository
        .get_by_realm_and_device(realm.id, &device_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get device profile: {}", e);
            ApiError::InternalServerError(format!("Failed to get device profile: {}", e))
        })?
        .ok_or_else(|| {
            tracing::error!("Device profile not found: {}", device_id);
            ApiError::NotFound(format!("Device profile '{}' not found", device_id))
        })?;

    Ok(Response::OK(DeviceProfileResponse::from(device_profile)))
}
