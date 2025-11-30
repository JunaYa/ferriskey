use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::entities::app_errors::CoreError,
        food_analysis::{
            entities::{
                FoodAnalysisItem, FoodAnalysisRequest, FoodAnalysisResult, FoodAnalysisTrigger,
            },
            ports::FoodAnalysisRepository,
            value_objects::GetFoodAnalysisFilter,
        },
    },
    entity::{
        food_analysis_items::{ActiveModel as ItemActiveModel, Entity as ItemEntity},
        food_analysis_requests::{
            ActiveModel as RequestActiveModel, Column as RequestColumn, Entity as RequestEntity,
        },
        food_analysis_results::{
            ActiveModel as ResultActiveModel, Column as ResultColumn, Entity as ResultEntity,
        },
        food_analysis_triggers::{ActiveModel as TriggerActiveModel, Entity as TriggerEntity},
    },
};

#[derive(Debug, Clone)]
pub struct PostgresFoodAnalysisRepository {
    pub db: DatabaseConnection,
}

impl PostgresFoodAnalysisRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl FoodAnalysisRepository for PostgresFoodAnalysisRepository {
    async fn create_request(
        &self,
        request: FoodAnalysisRequest,
    ) -> Result<FoodAnalysisRequest, CoreError> {
        let created = RequestEntity::insert(RequestActiveModel {
            id: Set(request.id),
            realm_id: Set(request.realm_id),
            prompt_id: Set(request.prompt_id),
            input_type: Set(request.input_type.as_str().to_string()),
            input_content: Set(Some(request.input_content)),
            created_by: Set(request.created_by),
            created_at: Set(request.created_at.fixed_offset()),
            device_id: Set(request.device_id),
            user_id: Set(request.user_id),
            updated_at: Set(request.created_at.fixed_offset()),
            updated_by: Set(request.created_by),
        })
        .exec_with_returning(&self.db)
        .await
        .map(FoodAnalysisRequest::from)
        .map_err(|e| {
            error!("Failed to create food analysis request: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(created)
    }

    async fn create_result(
        &self,
        result: FoodAnalysisResult,
    ) -> Result<FoodAnalysisResult, CoreError> {
        let dishes_json = serde_json::to_value(&result.dishes).map_err(|e| {
            error!("Failed to serialize dishes: {}", e);
            CoreError::InternalServerError
        })?;

        let created = ResultEntity::insert(ResultActiveModel {
            id: Set(result.id),
            request_id: Set(result.request_id),
            dishes: Set(dishes_json),
            raw_response: Set(result.raw_response),
            created_at: Set(result.created_at.fixed_offset()),
            updated_at: Set(result.created_at.fixed_offset()),
            updated_by: Set(result.created_by),
            created_by: Set(result.created_by),
        })
        .exec_with_returning(&self.db)
        .await
        .map(FoodAnalysisResult::from)
        .map_err(|e| {
            error!("Failed to create food analysis result: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(created)
    }

    async fn get_request_by_id(
        &self,
        request_id: Uuid,
        realm_id: Uuid,
    ) -> Result<Option<FoodAnalysisRequest>, CoreError> {
        let request = RequestEntity::find()
            .filter(RequestColumn::Id.eq(request_id))
            .filter(RequestColumn::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get food analysis request: {}", e);
                CoreError::InternalServerError
            })?
            .map(FoodAnalysisRequest::from);

        Ok(request)
    }

    async fn get_result_by_request_id(
        &self,
        request_id: Uuid,
    ) -> Result<Option<FoodAnalysisResult>, CoreError> {
        let result = ResultEntity::find()
            .filter(ResultColumn::RequestId.eq(request_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get food analysis result: {}", e);
                CoreError::InternalServerError
            })?
            .map(FoodAnalysisResult::from);

        Ok(result)
    }

    async fn get_requests_by_realm(
        &self,
        realm_id: Uuid,
        filter: GetFoodAnalysisFilter,
    ) -> Result<Vec<FoodAnalysisRequest>, CoreError> {
        use sea_orm::{Condition, Order};

        let mut query = RequestEntity::find().filter(RequestColumn::RealmId.eq(realm_id));

        // Apply filters
        let mut condition = Condition::all();

        if let Some(prompt_id) = filter.prompt_id {
            condition = condition.add(RequestColumn::PromptId.eq(prompt_id));
        }

        if let Some(ref input_type) = filter.input_type {
            condition = condition.add(RequestColumn::InputType.eq(input_type.clone()));
        }

        if let Some(user_id) = filter.user_id {
            condition = condition.add(RequestColumn::UserId.eq(user_id));
        }

        if let Some(created_at_gte) = filter.created_at_gte {
            condition = condition.add(RequestColumn::CreatedAt.gte(created_at_gte.fixed_offset()));
        }

        if let Some(created_at_lte) = filter.created_at_lte {
            condition = condition.add(RequestColumn::CreatedAt.lte(created_at_lte.fixed_offset()));
        }

        query = query.filter(condition);

        // Apply sorting
        if let Some(ref sort_str) = filter.sort {
            // Parse sort string like "-created_at" or "created_at,prompt_id"
            for sort_part in sort_str.split(',') {
                let sort_part = sort_part.trim();
                if let Some(field) = sort_part.strip_prefix('-') {
                    match field {
                        "created_at" => {
                            query = query.order_by(RequestColumn::CreatedAt, Order::Desc);
                        }
                        "updated_at" => {
                            query = query.order_by(RequestColumn::UpdatedAt, Order::Desc);
                        }
                        "prompt_id" => {
                            query = query.order_by(RequestColumn::PromptId, Order::Desc);
                        }
                        _ => {
                            // Unknown field, ignore
                        }
                    }
                } else {
                    match sort_part {
                        "created_at" => {
                            query = query.order_by(RequestColumn::CreatedAt, Order::Asc);
                        }
                        "updated_at" => {
                            query = query.order_by(RequestColumn::UpdatedAt, Order::Asc);
                        }
                        "prompt_id" => {
                            query = query.order_by(RequestColumn::PromptId, Order::Asc);
                        }
                        _ => {
                            // Unknown field, ignore
                        }
                    }
                }
            }
        } else {
            // Default sort: -created_at
            query = query.order_by_desc(RequestColumn::CreatedAt);
        }

        // Apply pagination
        if let Some(limit) = filter.limit {
            query = query.limit(limit as u64);
        }

        if let Some(offset) = filter.offset {
            query = query.offset(offset as u64);
        }

        let requests = query
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to fetch food analysis requests: {}", e);
                CoreError::InternalServerError
            })?
            .iter()
            .map(FoodAnalysisRequest::from)
            .collect();

        Ok(requests)
    }

    async fn create_items_batch(
        &self,
        items: Vec<FoodAnalysisItem>,
    ) -> Result<Vec<FoodAnalysisItem>, CoreError> {
        let mut created_items = Vec::new();

        for item in items {
            let active_model = ItemActiveModel {
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

            let created = ItemEntity::insert(active_model)
                .exec_with_returning(&self.db)
                .await
                .map_err(|e| {
                    error!("Failed to create food analysis item: {}", e);
                    CoreError::InternalServerError
                })?;

            created_items.push(FoodAnalysisItem::from(created));
        }

        Ok(created_items)
    }

    async fn create_triggers_batch(
        &self,
        triggers: Vec<FoodAnalysisTrigger>,
    ) -> Result<Vec<FoodAnalysisTrigger>, CoreError> {
        let mut created_triggers = Vec::new();

        for trigger in triggers {
            let active_model = TriggerActiveModel {
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

            let created = TriggerEntity::insert(active_model)
                .exec_with_returning(&self.db)
                .await
                .map_err(|e| {
                    error!("Failed to create food analysis trigger: {}", e);
                    CoreError::InternalServerError
                })?;

            created_triggers.push(FoodAnalysisTrigger::from(created));
        }

        Ok(created_triggers)
    }
}
