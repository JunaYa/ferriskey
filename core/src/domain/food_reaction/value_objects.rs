use chrono::DateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFoodReactionInput {
    pub analysis_item_id: Option<Uuid>,
    pub eaten_at: DateTime<chrono::Utc>,
    pub feeling: String,       // 'GREAT' | 'OKAY' | 'MILD_ISSUES' | 'BAD'
    pub symptom_onset: String, // 'LT_1H' | 'H1_3H' | 'H3_6H' | 'NEXT_DAY'
    pub symptoms: Vec<String>, // Symptom codes
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateFoodReactionInput {
    pub feeling: Option<String>,
    pub symptom_onset: Option<String>,
    pub symptoms: Option<Vec<String>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GetFoodReactionFilter {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub feeling: Option<String>,
    pub feeling_in: Option<Vec<String>>,
    pub analysis_item_id: Option<Uuid>,
    pub symptom_onset: Option<String>,
    pub eaten_at_gte: Option<DateTime<chrono::Utc>>,
    pub eaten_at_lte: Option<DateTime<chrono::Utc>>,
    pub created_at_gte: Option<DateTime<chrono::Utc>>,
    pub created_at_lte: Option<DateTime<chrono::Utc>>,
    pub has_symptoms: Option<bool>,
    pub sort: Option<String>, // e.g., "-eaten_at" or "eaten_at,feeling"
}
