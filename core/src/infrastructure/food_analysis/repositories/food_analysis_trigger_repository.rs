use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};
use tracing::error;

use crate::{
    domain::{
        common::entities::app_errors::CoreError,
        food_analysis::{entities::FoodAnalysisTrigger, ports::FoodAnalysisTriggerRepository},
    },
    entity::food_analysis_triggers::{ActiveModel, Entity},
};

#[derive(Debug, Clone)]
pub struct PostgresFoodAnalysisTriggerRepository {
    pub db: DatabaseConnection,
}

impl PostgresFoodAnalysisTriggerRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl FoodAnalysisTriggerRepository for PostgresFoodAnalysisTriggerRepository {
    async fn create_trigger(
        &self,
        trigger: FoodAnalysisTrigger,
    ) -> Result<FoodAnalysisTrigger, CoreError> {
        let active_model = ActiveModel {
            id: Set(trigger.id),
            realm_id: Set(trigger.realm_id),
            item_id: Set(trigger.item_id),
            ingredient_name: Set(trigger.ingredient_name.clone()),
            trigger_category: Set(trigger.trigger_category.clone()),
            risk_level: Set(trigger.risk_level.clone()),
            risk_reason: Set(trigger.risk_reason.clone()),
            created_at: Set(trigger.created_at.fixed_offset()),
            updated_at: Set(trigger.updated_at.fixed_offset()),
            created_by: Set(trigger.created_by),
            updated_by: Set(trigger.updated_by),
        };

        let created = Entity::insert(active_model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to create food analysis trigger: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(FoodAnalysisTrigger::from(created))
    }

    async fn create_triggers_batch(
        &self,
        triggers: Vec<FoodAnalysisTrigger>,
    ) -> Result<Vec<FoodAnalysisTrigger>, CoreError> {
        let mut created_triggers = Vec::new();

        for trigger in triggers {
            let created = self.create_trigger(trigger).await?;
            created_triggers.push(created);
        }

        Ok(created_triggers)
    }
}
