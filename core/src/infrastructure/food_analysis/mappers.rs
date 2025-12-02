use crate::{
    domain::food_analysis::entities::{
        FoodAnalysisItem, FoodAnalysisRequest, FoodAnalysisResult, FoodAnalysisTrigger,
    },
    entity::{
        food_analysis_items, food_analysis_requests, food_analysis_results, food_analysis_triggers,
    },
};

impl From<&food_analysis_requests::Model> for FoodAnalysisRequest {
    fn from(model: &food_analysis_requests::Model) -> Self {
        Self {
            id: model.id,
            realm_id: model.realm_id,
            prompt_id: model.prompt_id,
            device_id: model.device_id.clone(),
            user_id: model.user_id,
            input_type: model.input_type.as_str().into(),
            input_content: model.input_content.clone().unwrap_or_default(),
            created_by: model.created_by,
            created_at: model.created_at.to_utc(),
            updated_at: model.updated_at.to_utc(),
            updated_by: model.updated_by,
        }
    }
}

impl From<food_analysis_requests::Model> for FoodAnalysisRequest {
    fn from(model: food_analysis_requests::Model) -> Self {
        Self::from(&model)
    }
}

impl From<&food_analysis_results::Model> for FoodAnalysisResult {
    fn from(model: &food_analysis_results::Model) -> Self {
        let dishes: Vec<crate::domain::food_analysis::entities::DishAnalysis> =
            serde_json::from_value(model.dishes.clone()).unwrap_or_default();

        Self {
            id: model.id,
            request_id: model.request_id,
            dishes,
            raw_response: model.raw_response.clone(),
            created_at: model.created_at.to_utc(),
            updated_at: model.updated_at.to_utc(),
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl From<food_analysis_results::Model> for FoodAnalysisResult {
    fn from(model: food_analysis_results::Model) -> Self {
        Self::from(&model)
    }
}

impl From<&food_analysis_items::Model> for FoodAnalysisItem {
    fn from(model: &food_analysis_items::Model) -> Self {
        Self {
            id: model.id,
            realm_id: model.realm_id,
            request_id: model.request_id,
            result_id: model.result_id,
            dish_index: model.dish_index,
            input_index: model.input_index,
            dish_name: model.dish_name.clone(),
            safety_level: model.safety_level.clone(),
            risk_score: model.risk_score,
            risk_band: model.risk_band.clone(),
            summary_reason: model.summary_reason.clone(),
            ibd_concerns: model.ibd_concerns.clone(),
            ibs_concerns: model.ibs_concerns.clone(),
            recommendations: model.recommendations.clone(),
            image_object_key: model.image_object_key.clone(),
            reaction_info: None,
            created_at: model.created_at.to_utc(),
            updated_at: model.updated_at.to_utc(),
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl From<food_analysis_items::Model> for FoodAnalysisItem {
    fn from(model: food_analysis_items::Model) -> Self {
        Self::from(&model)
    }
}

impl From<&food_analysis_triggers::Model> for FoodAnalysisTrigger {
    fn from(model: &food_analysis_triggers::Model) -> Self {
        Self {
            id: model.id,
            realm_id: model.realm_id,
            item_id: model.item_id,
            ingredient_name: model.ingredient_name.clone(),
            trigger_category: model.trigger_category.clone(),
            risk_level: model.risk_level.clone(),
            risk_reason: model.risk_reason.clone(),
            created_at: model.created_at.to_utc(),
            updated_at: model.updated_at.to_utc(),
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl From<food_analysis_triggers::Model> for FoodAnalysisTrigger {
    fn from(model: food_analysis_triggers::Model) -> Self {
        Self::from(&model)
    }
}
