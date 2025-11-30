use std::future::Future;
use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    food_analysis::{
        entities::{
            FoodAnalysisItem, FoodAnalysisRequest, FoodAnalysisResult, FoodAnalysisTrigger,
        },
        value_objects::{
            AnalyzeFoodInput, GetFoodAnalysisFilter, GetFoodAnalysisHistoryInput,
            GetFoodAnalysisItemFilter, GetFoodAnalysisItemInput,
            GetFoodAnalysisItemsByRequestInput, GetFoodAnalysisItemsInput,
            GetFoodAnalysisRequestInput, GetFoodAnalysisResultInput, GetFoodAnalysisTriggerFilter,
            GetTriggerCategoryFilter, TriggerCategoryStats,
        },
    },
    realm::entities::Realm,
};

/// Repository trait for food analysis items
#[cfg_attr(test, mockall::automock)]
pub trait FoodAnalysisItemRepository: Send + Sync {
    fn create_item(
        &self,
        item: FoodAnalysisItem,
    ) -> impl Future<Output = Result<FoodAnalysisItem, CoreError>> + Send;

    fn create_items_batch(
        &self,
        items: Vec<FoodAnalysisItem>,
    ) -> impl Future<Output = Result<Vec<FoodAnalysisItem>, CoreError>> + Send;

    fn get_by_id(
        &self,
        item_id: Uuid,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<FoodAnalysisItem>, CoreError>> + Send;

    fn get_by_request_id(
        &self,
        request_id: Uuid,
        realm_id: Uuid,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<FoodAnalysisItem>, CoreError>> + Send;

    fn get_by_realm(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        filter: GetFoodAnalysisItemFilter,
    ) -> impl Future<Output = Result<Vec<FoodAnalysisItem>, CoreError>> + Send;
}

/// Repository trait for food analysis triggers
#[cfg_attr(test, mockall::automock)]
pub trait FoodAnalysisTriggerRepository: Send + Sync {
    fn create_trigger(
        &self,
        trigger: FoodAnalysisTrigger,
    ) -> impl Future<Output = Result<FoodAnalysisTrigger, CoreError>> + Send;

    fn create_triggers_batch(
        &self,
        triggers: Vec<FoodAnalysisTrigger>,
    ) -> impl Future<Output = Result<Vec<FoodAnalysisTrigger>, CoreError>> + Send;

    fn get_by_item_id(
        &self,
        item_id: Uuid,
        realm_id: Uuid,
        filter: GetFoodAnalysisTriggerFilter,
    ) -> impl Future<Output = Result<Vec<FoodAnalysisTrigger>, CoreError>> + Send;

    fn get_categories_stats(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        filter: GetTriggerCategoryFilter,
    ) -> impl Future<Output = Result<Vec<TriggerCategoryStats>, CoreError>> + Send;
}

/// Repository trait for food analysis data access
#[cfg_attr(test, mockall::automock)]
pub trait FoodAnalysisRepository: Send + Sync {
    fn create_request(
        &self,
        request: FoodAnalysisRequest,
    ) -> impl Future<Output = Result<FoodAnalysisRequest, CoreError>> + Send;

    fn create_result(
        &self,
        result: FoodAnalysisResult,
    ) -> impl Future<Output = Result<FoodAnalysisResult, CoreError>> + Send;

    fn get_request_by_id(
        &self,
        request_id: Uuid,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<FoodAnalysisRequest>, CoreError>> + Send;

    fn get_result_by_request_id(
        &self,
        request_id: Uuid,
    ) -> impl Future<Output = Result<Option<FoodAnalysisResult>, CoreError>> + Send;

    fn get_requests_by_realm(
        &self,
        realm_id: Uuid,
        filter: GetFoodAnalysisFilter,
    ) -> impl Future<Output = Result<Vec<FoodAnalysisRequest>, CoreError>> + Send;

    fn create_items_batch(
        &self,
        items: Vec<FoodAnalysisItem>,
    ) -> impl Future<Output = Result<Vec<FoodAnalysisItem>, CoreError>> + Send;

    fn create_triggers_batch(
        &self,
        triggers: Vec<FoodAnalysisTrigger>,
    ) -> impl Future<Output = Result<Vec<FoodAnalysisTrigger>, CoreError>> + Send;
}

/// LLM Client trait for calling AI models
#[cfg_attr(test, mockall::automock)]
pub trait LLMClient: Send + Sync {
    fn generate_with_image(
        &self,
        prompt: String,
        image_data: Vec<u8>,
        response_schema: serde_json::Value,
    ) -> impl Future<Output = Result<String, CoreError>> + Send;

    fn generate_with_text(
        &self,
        prompt: String,
        response_schema: serde_json::Value,
    ) -> impl Future<Output = Result<String, CoreError>> + Send;
}

/// Service trait for food analysis business logic
#[cfg_attr(test, mockall::automock)]
pub trait FoodAnalysisService: Send + Sync {
    fn analyze_food(
        &self,
        identity: Identity,
        input: AnalyzeFoodInput,
    ) -> impl Future<Output = Result<FoodAnalysisResult, CoreError>> + Send;

    fn get_analysis_history(
        &self,
        identity: Identity,
        input: GetFoodAnalysisHistoryInput,
    ) -> impl Future<Output = Result<Vec<FoodAnalysisRequest>, CoreError>> + Send;

    fn get_analysis_result(
        &self,
        identity: Identity,
        input: GetFoodAnalysisResultInput,
    ) -> impl Future<Output = Result<FoodAnalysisResult, CoreError>> + Send;

    fn get_analysis_request(
        &self,
        identity: Identity,
        input: GetFoodAnalysisRequestInput,
    ) -> impl Future<Output = Result<FoodAnalysisRequest, CoreError>> + Send;

    fn get_analysis_items_by_request(
        &self,
        identity: Identity,
        input: GetFoodAnalysisItemsByRequestInput,
    ) -> impl Future<Output = Result<Vec<FoodAnalysisItem>, CoreError>> + Send;

    fn get_analysis_item(
        &self,
        identity: Identity,
        input: GetFoodAnalysisItemInput,
    ) -> impl Future<Output = Result<FoodAnalysisItem, CoreError>> + Send;

    fn get_analysis_items(
        &self,
        identity: Identity,
        input: GetFoodAnalysisItemsInput,
    ) -> impl Future<Output = Result<Vec<FoodAnalysisItem>, CoreError>> + Send;
}

/// Policy trait for food analysis authorization
pub trait FoodAnalysisPolicy: Send + Sync {
    fn can_analyze_food(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_view_analysis(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
