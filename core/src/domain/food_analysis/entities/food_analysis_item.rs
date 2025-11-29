use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FoodAnalysisItem {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub request_id: Uuid,
    pub result_id: Uuid,
    pub dish_index: i32,
    pub input_index: Option<i32>,
    pub dish_name: String,
    pub safety_level: String, // 'SAFE' | 'CAUTION' | 'UNSAFE'
    pub risk_score: i32,      // 0-100
    pub risk_band: String,    // 'SAFE' | 'MODERATE' | 'HIGH'
    pub summary_reason: String,
    pub ibd_concerns: Vec<String>,
    pub ibs_concerns: Vec<String>,
    pub recommendations: String,
    pub image_object_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

#[derive(Debug, Clone)]
pub struct FoodAnalysisItemConfig {
    pub realm_id: Uuid,
    pub request_id: Uuid,
    pub result_id: Uuid,
    pub dish_index: i32,
    pub input_index: Option<i32>,
    pub dish_name: String,
    pub safety_level: String,
    pub risk_score: i32,
    pub risk_band: String,
    pub summary_reason: String,
    pub ibd_concerns: Vec<String>,
    pub ibs_concerns: Vec<String>,
    pub recommendations: String,
    pub image_object_key: Option<String>,
    pub created_by: Uuid,
}

impl FoodAnalysisItem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        realm_id: Uuid,
        request_id: Uuid,
        result_id: Uuid,
        dish_index: i32,
        input_index: Option<i32>,
        dish_name: String,
        safety_level: String,
        risk_score: i32,
        risk_band: String,
        summary_reason: String,
        ibd_concerns: Vec<String>,
        ibs_concerns: Vec<String>,
        recommendations: String,
        image_object_key: Option<String>,
        created_by: Uuid,
    ) -> Self {
        let (now, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id,
            request_id,
            result_id,
            dish_index,
            input_index,
            dish_name,
            safety_level,
            risk_score,
            risk_band,
            summary_reason,
            ibd_concerns,
            ibs_concerns,
            recommendations,
            image_object_key,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }
}
