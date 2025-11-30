use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, Order, QueryFilter, QueryOrder,
    QuerySelect,
    prelude::Expr,
    sea_query::{IntoCondition, extension::postgres::PgExpr},
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::entities::app_errors::CoreError,
        food_analysis::{
            entities::FoodAnalysisItem, ports::FoodAnalysisItemRepository,
            value_objects::GetFoodAnalysisItemFilter,
        },
    },
    entity::{
        food_analysis_items::{ActiveModel, Column, Entity},
        food_analysis_requests,
    },
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

    async fn get_by_id(
        &self,
        item_id: Uuid,
        realm_id: Uuid,
    ) -> Result<Option<FoodAnalysisItem>, CoreError> {
        let item = Entity::find()
            .filter(Column::Id.eq(item_id))
            .filter(Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get food analysis item: {}", e);
                CoreError::InternalServerError
            })?
            .map(FoodAnalysisItem::from);

        Ok(item)
    }

    async fn get_by_request_id(
        &self,
        request_id: Uuid,
        realm_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<FoodAnalysisItem>, CoreError> {
        // Join with food_analysis_requests to verify user_id
        let items = Entity::find()
            .inner_join(food_analysis_requests::Entity)
            .filter(Column::RequestId.eq(request_id))
            .filter(Column::RealmId.eq(realm_id))
            .filter(food_analysis_requests::Column::UserId.eq(user_id))
            .order_by(Column::DishIndex, Order::Asc)
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get food analysis items by request_id: {}", e);
                CoreError::InternalServerError
            })?
            .iter()
            .map(FoodAnalysisItem::from)
            .collect();

        Ok(items)
    }

    async fn get_by_realm(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        filter: GetFoodAnalysisItemFilter,
    ) -> Result<Vec<FoodAnalysisItem>, CoreError> {
        // Join with food_analysis_requests to filter by user_id
        let mut query = Entity::find()
            .inner_join(food_analysis_requests::Entity)
            .filter(Column::RealmId.eq(realm_id))
            .filter(food_analysis_requests::Column::UserId.eq(user_id));

        // Apply filters
        let mut condition = Condition::all();

        if let Some(request_id) = filter.request_id {
            condition = condition.add(Column::RequestId.eq(request_id));
        }

        if let Some(ref risk_band) = filter.risk_band {
            condition = condition.add(Column::RiskBand.eq(risk_band.clone()));
        }

        if let Some(ref risk_bands) = filter.risk_band_in
            && !risk_bands.is_empty()
        {
            condition = condition.add(Column::RiskBand.is_in(risk_bands.clone()));
        }

        if let Some(ref safety_level) = filter.safety_level {
            condition = condition.add(Column::SafetyLevel.eq(safety_level.clone()));
        }

        if let Some(risk_score_gte) = filter.risk_score_gte {
            condition = condition.add(Column::RiskScore.gte(risk_score_gte));
        }

        if let Some(risk_score_lte) = filter.risk_score_lte {
            condition = condition.add(Column::RiskScore.lte(risk_score_lte));
        }

        if let Some(ref dish_name) = filter.dish_name_ilike {
            condition = condition.add(
                Expr::col(Column::DishName)
                    .ilike(format!("%{}%", dish_name))
                    .into_condition(),
            );
        }

        if let Some(created_at_gte) = filter.created_at_gte {
            condition = condition.add(Column::CreatedAt.gte(created_at_gte.fixed_offset()));
        }

        if let Some(created_at_lte) = filter.created_at_lte {
            condition = condition.add(Column::CreatedAt.lte(created_at_lte.fixed_offset()));
        }

        query = query.filter(condition);

        // Apply sorting
        if let Some(ref sort_str) = filter.sort {
            // Parse sort string like "-risk_score" or "dish_index,-risk_score"
            for sort_part in sort_str.split(',') {
                let sort_part = sort_part.trim();
                if let Some(field) = sort_part.strip_prefix('-') {
                    match field {
                        "dish_index" => {
                            query = query.order_by(Column::DishIndex, Order::Desc);
                        }
                        "risk_score" => {
                            query = query.order_by(Column::RiskScore, Order::Desc);
                        }
                        "dish_name" => {
                            query = query.order_by(Column::DishName, Order::Desc);
                        }
                        "created_at" => {
                            query = query.order_by(Column::CreatedAt, Order::Desc);
                        }
                        "risk_band" => {
                            query = query.order_by(Column::RiskBand, Order::Desc);
                        }
                        _ => {
                            // Unknown field, ignore
                        }
                    }
                } else {
                    match sort_part {
                        "dish_index" => {
                            query = query.order_by(Column::DishIndex, Order::Asc);
                        }
                        "risk_score" => {
                            query = query.order_by(Column::RiskScore, Order::Asc);
                        }
                        "dish_name" => {
                            query = query.order_by(Column::DishName, Order::Asc);
                        }
                        "created_at" => {
                            query = query.order_by(Column::CreatedAt, Order::Asc);
                        }
                        "risk_band" => {
                            query = query.order_by(Column::RiskBand, Order::Asc);
                        }
                        _ => {
                            // Unknown field, ignore
                        }
                    }
                }
            }
        } else {
            // Default sort: -created_at
            query = query.order_by_desc(Column::CreatedAt);
        }

        // Apply pagination
        if let Some(limit) = filter.limit {
            query = query.limit(limit as u64);
        }

        if let Some(offset) = filter.offset {
            query = query.offset(offset as u64);
        }

        let items = query
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get food analysis items: {}", e);
                CoreError::InternalServerError
            })?
            .iter()
            .map(FoodAnalysisItem::from)
            .collect();

        Ok(items)
    }
}
