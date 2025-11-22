use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreatePromptValidator {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,

    #[validate(length(min = 1, message = "description is required"))]
    pub description: String,

    #[validate(length(min = 1, message = "template is required"))]
    pub template: String,

    #[validate(length(min = 1, message = "version is required"))]
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdatePromptValidator {
    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub template: Option<String>,

    #[serde(default)]
    pub version: Option<String>,

    #[serde(default)]
    pub is_active: Option<bool>,
}
