use axum::{
    Extension,
    extract::{Multipart, Path, State},
    http::HeaderMap,
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
    food_analysis::{
        entities::InputType, ports::FoodAnalysisService, value_objects::AnalyzeFoodInput,
    },
};

const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB

#[utoipa::path(
    post,
    path = "/image",
    tag = "food-analysis",
    summary = "Analyze food from image",
    description = "Analyzes food items from image using LLM vision",
    responses(
        (status = 200, body = AnalyzeFoodResponse)
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
)]
pub async fn analyze_food_image(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Response<AnalyzeFoodResponse>, ApiError> {
    let mut prompt_id: Option<Uuid> = None;
    let mut image_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to read multipart field: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "prompt_id" => {
                let value = field.text().await.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read prompt_id: {}", e))
                })?;
                prompt_id =
                    Some(Uuid::parse_str(&value).map_err(|_| {
                        ApiError::BadRequest("Invalid prompt_id format".to_string())
                    })?);
            }
            "image" => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("Failed to read image: {}", e)))?;

                if data.len() > MAX_IMAGE_SIZE {
                    return Err(ApiError::BadRequest(format!(
                        "Image too large. Max size is {} bytes",
                        MAX_IMAGE_SIZE
                    )));
                }

                image_data = Some(data.to_vec());
            }
            _ => {}
        }
    }

    let prompt_id =
        prompt_id.ok_or_else(|| ApiError::BadRequest("Missing prompt_id field".to_string()))?;

    let image_data =
        image_data.ok_or_else(|| ApiError::BadRequest("Missing image field".to_string()))?;

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
                prompt_id,
                input_type: InputType::Image,
                text_input: None,
                image_data: Some(image_data),
                device_id,
                user_id: identity.id(),
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(AnalyzeFoodResponse { data: result }))
}
