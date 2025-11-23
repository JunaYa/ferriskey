use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FoodAnalysisRequest {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub prompt_id: Uuid,
    pub input_type: InputType,
    pub input_content: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    Image,
    Text,
}

impl InputType {
    pub fn as_str(&self) -> &str {
        match self {
            InputType::Image => "image",
            InputType::Text => "text",
        }
    }
}

impl From<&str> for InputType {
    fn from(s: &str) -> Self {
        match s {
            "image" => InputType::Image,
            "text" => InputType::Text,
            _ => InputType::Text,
        }
    }
}

impl FoodAnalysisRequest {
    pub fn new(
        realm_id: Uuid,
        prompt_id: Uuid,
        input_type: InputType,
        input_content: String,
        created_by: Uuid,
    ) -> Self {
        let (_, timestamp) = generate_timestamp();
        let now = Utc::now();

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id,
            prompt_id,
            input_type,
            input_content,
            created_by,
            created_at: now,
        }
    }
}
