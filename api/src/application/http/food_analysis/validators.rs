use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct AnalyzeFoodTextRequest {
    pub prompt_id: Option<Uuid>,
    #[validate(length(
        min = 1,
        max = 5000,
        message = "text_input must be between 1 and 5000 characters"
    ))]
    pub text_input: String,
}

#[derive(Debug, Serialize, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct GetAnalysisHistoryParams {
    #[schema(example = 0)]
    pub offset: Option<u32>,
    #[schema(example = 20)]
    pub limit: Option<u32>,
}
