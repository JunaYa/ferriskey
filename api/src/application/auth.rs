use axum::{
    RequestPartsExt,
    extract::{FromRef, FromRequestParts, Request, State},
    http::{StatusCode, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use base64::{Engine, engine::general_purpose};
use ferriskey_core::domain::{
    authentication::{
        entities::AuthorizeRequestInput, ports::AuthService, value_objects::Identity,
    },
    device_profile::{
        entities::DeviceProfile,
        ports::DeviceProfileRepository,
        services::{
            generate_anonymous_email, generate_anonymous_name, generate_anonymous_username,
        },
    },
    jwt::entities::JwtClaim,
    realm::ports::RealmRepository,
    user::{ports::UserRepository, value_objects::CreateUserRequest},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;
use uuid::Uuid;

use super::http::server::{api_entities::api_error::ApiError, app_state::AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwt {
    claims: JwtClaim,
    token: String,
}

#[derive(Debug, Error, Deserialize, Serialize, PartialEq, Eq)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Token not found")]
    TokenNotFound,
    #[error("Invalid signature")]
    InvalidSignature,
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    code: String,
    message: String,
    status: i64,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AuthError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "E_UNAUTHORIZED", "Invalid token")
            }
            AuthError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, "E_UNAUTHORIZED", "Token expired")
            }
            AuthError::TokenNotFound => (
                StatusCode::UNAUTHORIZED,
                "E_UNAUTHORIZED",
                "Token not found",
            ),
            AuthError::InvalidSignature => (
                StatusCode::UNAUTHORIZED,
                "E_UNAUTHORIZED",
                "Invalid signature",
            ),
        };

        let error_response = ErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
            status: status.as_u16() as i64,
        };

        let body = serde_json::to_string(&error_response).unwrap_or_else(|_| {
            r#"{"code":"INTERNAL_SERVER_ERROR","message":"Failed to serialize error response"}"#
                .to_string()
        });

        axum::response::Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(body.clone().into())
            .unwrap_or_else(|_| axum::response::Response::new(body.clone().into()))
    }
}

impl<S> FromRequestParts<S> for Jwt
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_token_from_bearer(parts).await?;

        let t: Vec<&str> = token.split('.').collect();
        if t.len() != 3 {
            return Err(AuthError::InvalidToken);
        }

        let payload = t[1];

        let decoded = general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|e| {
                tracing::error!("Failed to decode JWT payload: {:?}", e);
                AuthError::InvalidToken
            })?;

        let payload_str = String::from_utf8(decoded).map_err(|e| {
            tracing::error!("Failed to decode JWT payload: {:?}", e);
            AuthError::InvalidToken
        })?;
        let claims: JwtClaim = serde_json::from_str(&payload_str).map_err(|e| {
            tracing::error!("Failed to deserialize JWT claims: {:?}", e);
            AuthError::InvalidToken
        })?;

        Ok(Jwt {
            claims,
            token: token.clone(),
        })
    }
}

pub async fn extract_token_from_bearer(parts: &mut Parts) -> Result<String, AuthError> {
    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| AuthError::TokenNotFound)?;

    Ok(bearer.token().to_string())
}

/// Optional auth middleware that supports both token-based and device-based authentication
/// - If Bearer token is provided: validates token and sets Identity
/// - If no token but X-Device-Id is provided: allows request to continue (device_middleware will handle)
pub async fn auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Try to extract token from Authorization header (optional)
    if let Some(auth_header) = req.headers().get("authorization")
        && let Ok(auth_str) = auth_header.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
        && !token.is_empty()
    {
        // Token-based authentication
        let t: Vec<&str> = token.split('.').collect();
        if t.len() == 3 {
            let payload = t[1];

            if let Ok(decoded) = general_purpose::URL_SAFE_NO_PAD.decode(payload)
                && let Ok(payload_str) = String::from_utf8(decoded)
                && let Ok(claims) = serde_json::from_str::<JwtClaim>(&payload_str)
            {
                let output = state
                    .service
                    .authorize_request(AuthorizeRequestInput {
                        claims,
                        token: token.to_string(),
                    })
                    .await;

                if let Ok(output) = output {
                    req.extensions_mut().insert(output.identity);
                }
            }
        }
    }
    // If no token or token validation failed, continue without Identity
    // device_middleware will handle device-based authentication

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

/// Get admin user ID for a given realm
async fn get_admin_user_id(
    user_repository: &ferriskey_core::infrastructure::user::repository::PostgresUserRepository,
    realm_id: Uuid,
) -> Result<Uuid, ApiError> {
    let admin_user = user_repository
        .get_by_username("admin".to_string(), realm_id)
        .await
        .map_err(|e| {
            error!("Failed to get admin user: {}", e);
            ApiError::InternalServerError("Failed to get admin user".to_string())
        })?;

    Ok(admin_user.id)
}

/// Create Identity from device_id
async fn create_identity_from_device(
    state: &AppState,
    device_id: &str,
    realm_name: &str,
) -> Result<Identity, ApiError> {
    // 1. Get realm
    let realm = state
        .realm_repository
        .get_by_name(realm_name.to_string())
        .await
        .map_err(|e| {
            error!("Failed to get realm: {}", e);
            ApiError::InternalServerError("Failed to get realm".to_string())
        })?
        .ok_or_else(|| {
            error!("Realm not found: {}", realm_name);
            ApiError::NotFound(format!("Realm '{}' not found", realm_name))
        })?;

    // 2. Get or create device profile
    let device_profile = if let Some(profile) = state
        .device_profile_repository
        .get_by_realm_and_device(realm.id, device_id)
        .await
        .map_err(|e| {
            error!("Failed to get device profile: {}", e);
            ApiError::InternalServerError("Failed to get device profile".to_string())
        })? {
        profile
    } else {
        // Device profile doesn't exist, create anonymous user and device profile
        // 2.1 Get admin user ID for created_by
        let admin_user_id = get_admin_user_id(&state.user_repository, realm.id).await?;

        // 2.2 Create anonymous user
        let username = generate_anonymous_username(device_id);
        let email = generate_anonymous_email(device_id);
        let firstname = generate_anonymous_name(device_id, "firstname");
        let lastname = generate_anonymous_name(device_id, "lastname");

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
                ApiError::InternalServerError("用户初始化失败".to_string())
            })?;

        // 2.3 Create device profile
        let device_profile = DeviceProfile::new(
            realm.id,
            device_id.to_string(),
            user.id,
            Some(admin_user_id),
        );

        state
            .device_profile_repository
            .create(device_profile)
            .await
            .map_err(|e| {
                error!("Failed to create device profile: {}", e);
                ApiError::InternalServerError("用户初始化失败".to_string())
            })?
    };

    // 3. Get user from device profile
    let user = state
        .user_repository
        .get_by_id(device_profile.user_id)
        .await
        .map_err(|e| {
            error!("Failed to get user: {}", e);
            ApiError::InternalServerError("用户初始化失败".to_string())
        })?;

    // 4. Create Identity
    Ok(Identity::User(user))
}

/// Custom extractor for required Identity
/// Supports both Bearer token and X-Device-Id authentication
/// Priority: Bearer token > X-Device-Id
pub struct RequiredIdentity(pub Identity);

impl<S> FromRequestParts<S> for RequiredIdentity
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. First, check if Identity already exists (from Bearer token)
        if let Some(identity) = parts.extensions.get::<Identity>().cloned() {
            return Ok(RequiredIdentity(identity));
        }

        // 2. If no Identity, try to get from X-Device-Id
        let device_id = parts
            .headers
            .get("x-device-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let device_id = match device_id {
            Some(id) if !id.is_empty() => id,
            _ => {
                return Err(ApiError::Unauthorized(
                    "Authentication required: provide either Authorization header or X-Device-Id header".to_string(),
                ));
            }
        };

        // 3. Extract realm_name from path
        let realm_name = extract_realm_from_path(parts.uri.path()).ok_or_else(|| {
            ApiError::BadRequest("Invalid path: realm_name not found in path".to_string())
        })?;

        // 4. Get AppState
        let app_state = AppState::from_ref(state);

        // 5. Create Identity from device_id
        let identity = create_identity_from_device(&app_state, &device_id, &realm_name).await?;

        // 6. Store Identity in extensions for future use
        parts.extensions.insert(identity.clone());

        Ok(RequiredIdentity(identity))
    }
}
