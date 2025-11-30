use axum::{
    Extension,
    extract::{Path, State},
};

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    device_profile::services::get_or_create_device_profile,
    realm::ports::{GetRealmInput, RealmService},
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
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeviceProfileResponse>, ApiError> {
    // Get realm
    let realm = state
        .service
        .get_realm_by_name(
            identity.clone(),
            GetRealmInput {
                realm_name: realm_name.clone(),
            },
        )
        .await
        .map_err(ApiError::from)?;

    // Get or create device profile
    let device_profile = get_or_create_device_profile(
        &*state.device_profile_repository,
        &*state.user_repository,
        realm,
        &device_id,
        &identity,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to get or create device profile: {}", e);
        ApiError::InternalServerError(format!("Failed to get or create device profile: {}", e))
    })?;

    Ok(Response::OK(DeviceProfileResponse::from(device_profile)))
}
