use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FoodAnalysisTrigger {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub item_id: Uuid,
    pub ingredient_name: String,
    pub trigger_category: String,
    pub risk_level: String, // 'HIGH' | 'MEDIUM' | 'LOW'
    pub risk_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

impl FoodAnalysisTrigger {
    pub fn new(
        realm_id: Uuid,
        item_id: Uuid,
        ingredient_name: String,
        trigger_category: String,
        risk_level: String,
        risk_reason: Option<String>,
        created_by: Uuid,
    ) -> Self {
        let (now, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id,
            item_id,
            ingredient_name,
            trigger_category,
            risk_level,
            risk_reason,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }
}
