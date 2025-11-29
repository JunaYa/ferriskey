use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_timestamp;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct FoodAnalysisResult {
    pub id: Uuid,
    pub request_id: Uuid,
    pub dishes: Vec<DishAnalysis>,
    pub raw_response: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Uuid,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct DishAnalysis {
    pub dish_name: String,
    pub safety_level: SafetyLevel,
    pub reason: String,
    pub ibd_concerns: Vec<String>,
    pub ibs_concerns: Vec<String>,
    pub recommendations: String,
    pub ingredients: Vec<RiskIngredient>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum SafetyLevel {
    Safe,
    Caution,
    Unsafe,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RiskIngredient {
    pub ingredient_name: String,
    pub risk_reason: String,
}

impl FoodAnalysisResult {
    pub fn new(
        request_id: Uuid,
        dishes: Vec<DishAnalysis>,
        raw_response: String,
        created_by: Uuid,
        updated_by: Uuid,
    ) -> Self {
        let (_, timestamp) = generate_timestamp();
        let now = Utc::now();

        Self {
            id: Uuid::new_v7(timestamp),
            request_id,
            dishes,
            raw_response,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by,
        }
    }
}
