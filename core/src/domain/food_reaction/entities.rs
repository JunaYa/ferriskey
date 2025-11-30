use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FoodReaction {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub device_id: String,
    pub user_id: Uuid,
    pub analysis_item_id: Option<Uuid>,
    pub eaten_at: DateTime<Utc>,
    pub feeling: String,       // 'GREAT' | 'OKAY' | 'MILD_ISSUES' | 'BAD'
    pub symptom_onset: String, // 'LT_1H' | 'H1_3H' | 'H3_6H' | 'NEXT_DAY'
    pub notes: Option<String>,
    pub symptoms: Vec<String>, // Symptom codes
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
}

#[derive(Debug, Clone)]
pub struct FoodReactionConfig {
    pub realm_id: Uuid,
    pub device_id: String,
    pub user_id: Uuid,
    pub analysis_item_id: Option<Uuid>,
    pub eaten_at: DateTime<Utc>,
    pub feeling: String,
    pub symptom_onset: String,
    pub notes: Option<String>,
    pub symptoms: Vec<String>,
    pub created_by: Uuid,
}

impl FoodReaction {
    pub fn new(config: FoodReactionConfig) -> Self {
        let (now, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id: config.realm_id,
            device_id: config.device_id,
            user_id: config.user_id,
            analysis_item_id: config.analysis_item_id,
            eaten_at: config.eaten_at,
            feeling: config.feeling,
            symptom_onset: config.symptom_onset,
            notes: config.notes,
            symptoms: config.symptoms,
            created_at: now,
            updated_at: now,
            created_by: config.created_by,
            updated_by: config.created_by,
        }
    }

    pub fn update(
        &mut self,
        feeling: Option<String>,
        symptom_onset: Option<String>,
        symptoms: Option<Vec<String>>,
        notes: Option<String>,
        updated_by: Uuid,
    ) {
        let (now, _) = generate_timestamp();

        if let Some(f) = feeling {
            self.feeling = f;
        }
        if let Some(so) = symptom_onset {
            self.symptom_onset = so;
        }
        if let Some(s) = symptoms {
            self.symptoms = s;
        }
        if let Some(n) = notes {
            self.notes = Some(n);
        }
        self.updated_at = now;
        self.updated_by = updated_by;
    }
}
