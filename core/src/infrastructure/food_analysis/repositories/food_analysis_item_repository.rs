use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};
use tracing::error;

use crate::{
    domain::{
        common::entities::app_errors::CoreError,
        food_analysis::{entities::FoodAnalysisItem, ports::FoodAnalysisItemRepository},
    },
    entity::food_analysis_items::{ActiveModel, Entity},
};

#[derive(Debug, Clone)]
pub struct PostgresFoodAnalysisItemRepository {
    pub db: DatabaseConnection,
}

impl PostgresFoodAnalysisItemRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl FoodAnalysisItemRepository for PostgresFoodAnalysisItemRepository {
    async fn create_item(&self, item: FoodAnalysisItem) -> Result<FoodAnalysisItem, CoreError> {
        let active_model = ActiveModel {
            id: Set(item.id),
            realm_id: Set(item.realm_id),
            request_id: Set(item.request_id),
            result_id: Set(item.result_id),
            dish_index: Set(item.dish_index),
            input_index: Set(item.input_index),
            dish_name: Set(item.dish_name.clone()),
            safety_level: Set(item.safety_level.clone()),
            risk_score: Set(item.risk_score),
            risk_band: Set(item.risk_band.clone()),
            summary_reason: Set(item.summary_reason.clone()),
            ibd_concerns: Set(item.ibd_concerns.clone()),
            ibs_concerns: Set(item.ibs_concerns.clone()),
            recommendations: Set(item.recommendations.clone()),
            image_object_key: Set(item.image_object_key.clone()),
            created_at: Set(item.created_at.fixed_offset()),
            updated_at: Set(item.updated_at.fixed_offset()),
            created_by: Set(item.created_by),
            updated_by: Set(item.updated_by),
        };

        let created = Entity::insert(active_model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to create food analysis item: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(FoodAnalysisItem::from(created))
    }

    async fn create_items_batch(
        &self,
        items: Vec<FoodAnalysisItem>,
    ) -> Result<Vec<FoodAnalysisItem>, CoreError> {
        let mut created_items = Vec::new();

        for item in items {
            let created = self.create_item(item).await?;
            created_items.push(created);
        }

        Ok(created_items)
    }
}
