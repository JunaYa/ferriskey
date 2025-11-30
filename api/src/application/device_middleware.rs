use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    device_profile::{ports::DeviceProfileRepository, services::get_or_create_device_profile},
    realm::ports::RealmRepository,
    user::ports::UserRepository,
};
use tracing::error;
use uuid::Uuid;

use crate::application::http::server::app_state::AppState;

/// Device profile context stored in request extensions
#[derive(Clone, Debug)]
pub struct DeviceContext {
    pub device_id: String,
    pub user_id: Uuid,
}

/// Middleware to handle device identification
/// Extracts X-Device-Id header and gets/creates device profile
/// Note: This middleware should be applied after auth middleware
pub async fn device_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // Extract device_id from header
    let device_id = req
        .headers()
        .get("x-device-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // If no device_id, generate a default one (fallback)
            // In production, you might want to return an error instead
            "default_device".to_string()
        });

    // Extract realm_name from path
    let realm_name =
        extract_realm_from_path(req.uri().path()).ok_or(axum::http::StatusCode::BAD_REQUEST)?;

    // Get realm - try to use Identity if available, otherwise get realm directly
    use ferriskey_core::domain::realm::ports::{GetRealmInput, RealmService};

    let identity = req.extensions().get::<Identity>().cloned();
    let realm = if let Some(ref identity) = identity {
        // Token-based authentication: use Identity to get realm
        state
            .service
            .get_realm_by_name(
                identity.clone(),
                GetRealmInput {
                    realm_name: realm_name.clone(),
                },
            )
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        // Device-based authentication: get realm directly from repository
        state
            .realm_repository
            .get_by_name(realm_name.clone())
            .await
            .map_err(|e| {
                error!("Failed to get realm: {}", e);
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(axum::http::StatusCode::NOT_FOUND)?
    };

    // Get or create device profile
    // If Identity exists, use it; otherwise, create a temporary identity for anonymous users
    let device_profile = if let Some(ref identity) = identity {
        // Token-based authentication: use existing Identity
        get_or_create_device_profile(
            &*state.device_profile_repository,
            &*state.user_repository,
            realm,
            &device_id,
            identity,
        )
        .await
        .map_err(|e| {
            error!("Failed to get or create device profile: {}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?
    } else {
        // Device-based authentication: create anonymous user first, then device profile
        // 1. Check if device profile already exists
        if let Some(profile) = state
            .device_profile_repository
            .get_by_realm_and_device(realm.id, &device_id)
            .await
            .map_err(|e| {
                error!("Failed to get device profile: {}", e);
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            })?
        {
            profile
        } else {
            // 2. Create anonymous user
            use ferriskey_core::domain::device_profile::services::{
                generate_anonymous_email, generate_anonymous_name, generate_anonymous_username,
            };
            use ferriskey_core::domain::user::value_objects::CreateUserRequest;

            let username = generate_anonymous_username(&device_id);
            let email = generate_anonymous_email(&device_id);
            let firstname = generate_anonymous_name(&device_id, "firstname");
            let lastname = generate_anonymous_name(&device_id, "lastname");

            let user = state
                .user_repository
                .create_user(CreateUserRequest {
                    realm_id: realm.id,
                    username: username.clone(),
                    email,
                    firstname,
                    lastname,
                    email_verified: false,
                    enabled: true,
                    client_id: None,
                })
                .await
                .map_err(|e| {
                    error!("Failed to create anonymous user: {}", e);
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR
                })?;

            // 3. Create device profile with user.id as created_by
            use ferriskey_core::domain::device_profile::entities::DeviceProfile;
            let device_profile = DeviceProfile::new(
                realm.id,
                device_id.to_string(),
                user.id,
                Some(user.id), // Use user.id as created_by for anonymous users
            );

            state
                .device_profile_repository
                .create(device_profile)
                .await
                .map_err(|e| {
                    error!("Failed to create device profile: {}", e);
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR
                })?
        }
    };

    // Store device context in request extensions
    req.extensions_mut().insert(DeviceContext {
        device_id: device_profile.device_id.clone(),
        user_id: device_profile.user_id,
    });

    Ok(next.run(req).await)
}

/// Extract realm name from path like "/realms/{realm_name}/..."
fn extract_realm_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if let Some(realm_idx) = parts.iter().position(|&p| p == "realms")
        && realm_idx + 1 < parts.len()
    {
        return Some(parts[realm_idx + 1].to_string());
    }
    None
}
