use crate::{
    domain::food_reaction::entities::FoodReaction,
    entity::{food_reaction_symptoms, food_reactions},
};

impl From<&food_reactions::Model> for FoodReaction {
    fn from(model: &food_reactions::Model) -> Self {
        // Note: symptoms will be loaded separately
        Self {
            id: model.id,
            realm_id: model.realm_id,
            device_id: model.device_id.clone(),
            user_id: model.user_id,
            analysis_item_id: model.analysis_item_id,
            eaten_at: model.eaten_at.to_utc(),
            feeling: model.feeling.clone(),
            symptom_onset: model.symptom_onset.clone(),
            notes: model.notes.clone(),
            symptoms: Vec::new(), // Will be populated separately
            created_at: model.created_at.to_utc(),
            updated_at: model.updated_at.to_utc(),
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl From<food_reactions::Model> for FoodReaction {
    fn from(model: food_reactions::Model) -> Self {
        Self::from(&model)
    }
}

pub fn map_symptoms(symptoms: Vec<food_reaction_symptoms::Model>) -> Vec<String> {
    symptoms.into_iter().map(|s| s.symptom_code).collect()
}
