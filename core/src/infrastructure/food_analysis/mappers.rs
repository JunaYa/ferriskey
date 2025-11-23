use crate::{
    domain::food_analysis::entities::{FoodAnalysisRequest, FoodAnalysisResult, InputType},
    entity::{food_analysis_requests, food_analysis_results},
};

impl From<&food_analysis_requests::Model> for FoodAnalysisRequest {
    fn from(model: &food_analysis_requests::Model) -> Self {
        Self {
            id: model.id,
            realm_id: model.realm_id,
            prompt_id: model.prompt_id,
            input_type: InputType::from(model.input_type.as_str()),
            input_content: model.input_content.clone().unwrap_or_default(),
            created_by: model.created_by,
            created_at: model.created_at.to_utc(),
        }
    }
}

impl From<food_analysis_requests::Model> for FoodAnalysisRequest {
    fn from(model: food_analysis_requests::Model) -> Self {
        Self::from(&model)
    }
}

impl From<&food_analysis_results::Model> for FoodAnalysisResult {
    fn from(model: &food_analysis_results::Model) -> Self {
        let dishes = serde_json::from_value(model.dishes.clone()).unwrap_or_default();

        Self {
            id: model.id,
            request_id: model.request_id,
            dishes,
            raw_response: model.raw_response.clone(),
            created_at: model.created_at.to_utc(),
        }
    }
}

impl From<food_analysis_results::Model> for FoodAnalysisResult {
    fn from(model: food_analysis_results::Model) -> Self {
        Self::from(&model)
    }
}
