use axum::{
    Extension,
    extract::{Path, State},
};

use crate::application::{
    device_middleware::DeviceContext,
    http::{
        food_analysis::validators::AnalyzeFoodTextRequest,
        server::{
            api_entities::{
                api_error::{ApiError, ValidateJson},
                response::Response,
            },
            app_state::AppState,
        },
    },
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    food_analysis::{
        entities::{FoodAnalysisResult, InputType},
        ports::FoodAnalysisService,
        value_objects::AnalyzeFoodInput,
    },
    user::ports::UserRepository,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AnalyzeFoodResponse {
    pub data: FoodAnalysisResult,
}

#[utoipa::path(
    post,
    path = "/text",
    tag = "food-analysis",
    summary = "Analyze food from text description",
    description = "Analyzes food items from text description using LLM",
    responses(
        (status = 200, body = AnalyzeFoodResponse)
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body = AnalyzeFoodTextRequest
)]
pub async fn analyze_food_text(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(device_context): Extension<DeviceContext>,
    ValidateJson(payload): ValidateJson<AnalyzeFoodTextRequest>,
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
        .analyze_food(
            identity,
            AnalyzeFoodInput {
                realm_name,
                prompt_id: payload.prompt_id,
                input_type: InputType::Text,
                text_input: Some(payload.text_input),
                image_data: None,
                device_id: device_context.device_id.clone(),
                user_id: device_context.user_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(AnalyzeFoodResponse { data: result }))
}
