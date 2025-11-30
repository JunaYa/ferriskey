use axum::{
    Extension,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::application::{
    device_middleware::DeviceContext,
    http::{
        food_analysis::handlers::analyze_food_text::AnalyzeFoodResponse,
        server::{
            api_entities::{api_error::ApiError, response::Response},
            app_state::AppState,
        },
    },
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    food_analysis::{ports::FoodAnalysisService, value_objects::GetFoodAnalysisResultInput},
    user::ports::UserRepository,
};

#[utoipa::path(
    get,
    path = "/{request_id}/result",
    tag = "food-analysis",
    summary = "Get food analysis result",
    description = "Get the result of a specific food analysis request",
    responses(
        (status = 200, body = AnalyzeFoodResponse)
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("request_id" = Uuid, Path, description = "Request ID"),
    ),
)]
pub async fn get_analysis_result(
    Path((realm_name, request_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(device_context): Extension<DeviceContext>,
) -> Result<Response<AnalyzeFoodResponse>, ApiError> {
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

    let result = state
        .service
        .get_analysis_result(
            identity,
            GetFoodAnalysisResultInput {
                realm_name,
                request_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(AnalyzeFoodResponse { data: result }))
}
