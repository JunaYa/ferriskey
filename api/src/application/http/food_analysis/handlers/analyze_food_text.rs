use axum::{
    Extension,
    extract::{Path, State},
    http::HeaderMap,
};

use crate::application::http::{
    food_analysis::validators::AnalyzeFoodTextRequest,
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    food_analysis::{
        entities::{FoodAnalysisResult, InputType},
        ports::FoodAnalysisService,
        value_objects::AnalyzeFoodInput,
    },
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
    Extension(identity): Extension<Identity>,
    headers: HeaderMap,
    ValidateJson(payload): ValidateJson<AnalyzeFoodTextRequest>,
) -> Result<Response<AnalyzeFoodResponse>, ApiError> {
    let device_id = headers
        .get("x-device-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| identity.id().to_string());

    let result = state
        .service
        .analyze_food(
            identity.clone(),
            AnalyzeFoodInput {
                realm_name,
                prompt_id: payload.prompt_id,
                input_type: InputType::Text,
                text_input: Some(payload.text_input),
                image_data: None,
                device_id,
                user_id: identity.id(),
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(AnalyzeFoodResponse { data: result }))
}
