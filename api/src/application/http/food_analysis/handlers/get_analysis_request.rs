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
    authentication::value_objects::Identity,
    food_analysis::{
        entities::FoodAnalysisRequest, ports::FoodAnalysisService,
        value_objects::GetFoodAnalysisRequestInput,
    },
    user::ports::UserRepository,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetAnalysisRequestResponse {
    pub data: FoodAnalysisRequest,
}

#[utoipa::path(
    get,
    path = "/requests/{request_id}",
    tag = "food-analysis",
    summary = "Get a food analysis request",
    description = "Get a single food analysis request by ID",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("request_id" = Uuid, Path, description = "Request ID"),
    ),
    responses(
        (status = 200, body = GetAnalysisRequestResponse),
        (status = 404, description = "Request not found")
    )
)]
pub async fn get_analysis_request(
    Path((realm_name, request_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(device_context): Extension<DeviceContext>,
) -> Result<Response<GetAnalysisRequestResponse>, ApiError> {
    // Create Identity from DeviceContext (for device authentication mode)
    let user = state
        .user_repository
        .get_by_id(device_context.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get user: {}", e);
            ApiError::InternalServerError(format!("Failed to get user: {}", e))
        })?;

    let identity = Identity::User(user);

    let request = state
        .service
        .get_analysis_request(
            identity,
            GetFoodAnalysisRequestInput {
                realm_name,
                request_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetAnalysisRequestResponse { data: request }))
}
