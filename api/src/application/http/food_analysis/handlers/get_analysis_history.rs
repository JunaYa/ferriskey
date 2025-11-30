use axum::{
    Extension,
    extract::{Path, Query, State},
};

use crate::application::{
    device_middleware::DeviceContext,
    http::{
        food_analysis::validators::GetAnalysisHistoryParams,
        server::{
            api_entities::{api_error::ApiError, response::Response},
            app_state::AppState,
        },
    },
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    food_analysis::{
        entities::FoodAnalysisRequest, ports::FoodAnalysisService,
        value_objects::GetFoodAnalysisHistoryInput,
    },
    user::ports::UserRepository,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetAnalysisHistoryResponse {
    pub data: Vec<FoodAnalysisRequest>,
}

#[utoipa::path(
    get,
    path = "",
    tag = "food-analysis",
    summary = "Get food analysis history",
    description = "Get history of food analysis requests for a realm",
    responses(
        (status = 200, body = GetAnalysisHistoryResponse)
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        GetAnalysisHistoryParams
    ),
)]
pub async fn get_analysis_history(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    Query(params): Query<GetAnalysisHistoryParams>,
    Extension(device_context): Extension<DeviceContext>,
) -> Result<Response<GetAnalysisHistoryResponse>, ApiError> {
    use ferriskey_core::domain::food_analysis::value_objects::GetFoodAnalysisFilter;

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

    let filter = GetFoodAnalysisFilter {
        offset: params.offset,
        limit: params.limit,
        user_id: Some(device_context.user_id), // Filter by device user
        ..Default::default()
    };

    let requests = state
        .service
        .get_analysis_history(identity, GetFoodAnalysisHistoryInput { realm_name, filter })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetAnalysisHistoryResponse { data: requests }))
}
