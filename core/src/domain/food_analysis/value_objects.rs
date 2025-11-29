use uuid::Uuid;

use crate::domain::food_analysis::entities::InputType;

#[derive(Debug, Clone)]
pub struct AnalyzeFoodInput {
    pub realm_name: String,
    pub prompt_id: Uuid,
    pub input_type: InputType,
    pub text_input: Option<String>,
    pub image_data: Option<Vec<u8>>,
    pub device_id: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct GetFoodAnalysisHistoryInput {
    pub realm_name: String,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct GetFoodAnalysisResultInput {
    pub realm_name: String,
    pub request_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct GetFoodAnalysisFilter {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}
