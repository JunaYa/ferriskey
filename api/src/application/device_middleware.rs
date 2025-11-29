use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity, device_profile::services::get_or_create_device_profile,
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

    // Extract identity from extensions (set by auth middleware)
    let identity = req
        .extensions()
        .get::<Identity>()
        .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

    // Extract realm_name from path
    let realm_name =
        extract_realm_from_path(req.uri().path()).ok_or(axum::http::StatusCode::BAD_REQUEST)?;

    // Get realm using service
    use ferriskey_core::domain::realm::ports::{GetRealmInput, RealmService};

    let realm = state
        .service
        .get_realm_by_name(
            identity.clone(),
            GetRealmInput {
                realm_name: realm_name.clone(),
            },
        )
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get or create device profile
    // We need user_repository, but it's private in Service
    // Let's add it to AppState or create a helper method
    // For now, let's add user_repository to AppState
    let device_profile = get_or_create_device_profile(
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
    })?;

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
