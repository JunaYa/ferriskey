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
            entities::{FoodAnalysisRequest, FoodAnalysisResult},
            ports::FoodAnalysisRepository,
            value_objects::GetFoodAnalysisFilter,
        },
    },
    entity::{
        food_analysis_requests::{
            ActiveModel as RequestActiveModel, Column as RequestColumn, Entity as RequestEntity,
        },
        food_analysis_results::{
            ActiveModel as ResultActiveModel, Column as ResultColumn, Entity as ResultEntity,
        },
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
        let mut query = RequestEntity::find()
            .filter(RequestColumn::RealmId.eq(realm_id))
            .order_by_desc(RequestColumn::CreatedAt);

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
}
