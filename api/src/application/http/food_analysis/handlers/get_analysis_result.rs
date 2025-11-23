use axum::{
    Extension,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::application::http::{
    food_analysis::handlers::analyze_food_text::AnalyzeFoodResponse,
    server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    food_analysis::{ports::FoodAnalysisService, value_objects::GetFoodAnalysisResultInput},
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
    Extension(identity): Extension<Identity>,
) -> Result<Response<AnalyzeFoodResponse>, ApiError> {
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
