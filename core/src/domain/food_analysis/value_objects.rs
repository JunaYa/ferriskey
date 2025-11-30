use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::food_analysis::entities::SafetyLevel;

#[derive(Debug, Clone, Default)]
pub struct GetFoodAnalysisFilter {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub prompt_id: Option<Uuid>,
    pub input_type: Option<String>, // 'image' | 'text'
    pub user_id: Option<Uuid>,
    pub created_at_gte: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at_lte: Option<chrono::DateTime<chrono::Utc>>,
    pub sort: Option<String>, // e.g., "-created_at" or "created_at,prompt_id"
}

#[derive(Debug, Clone, Default)]
pub struct GetFoodAnalysisItemFilter {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub request_id: Option<Uuid>,
    pub risk_band: Option<String>, // 'SAFE' | 'MODERATE' | 'HIGH'
    pub risk_band_in: Option<Vec<String>>, // Multiple risk bands
    pub safety_level: Option<String>, // 'SAFE' | 'CAUTION' | 'UNSAFE'
    pub risk_score_gte: Option<i32>,
    pub risk_score_lte: Option<i32>,
    pub dish_name_ilike: Option<String>,
    pub created_at_gte: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at_lte: Option<chrono::DateTime<chrono::Utc>>,
    pub sort: Option<String>, // e.g., "dish_index" or "-risk_score,dish_name"
    pub include_reaction_info: bool, // Whether to include reaction_info in results
}

#[derive(Debug, Clone)]
pub struct GetFoodAnalysisHistoryInput {
    pub realm_name: String,
    pub filter: GetFoodAnalysisFilter,
}

#[derive(Debug, Clone)]
pub struct GetFoodAnalysisResultInput {
    pub realm_name: String,
    pub request_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct GetFoodAnalysisRequestInput {
    pub realm_name: String,
    pub request_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct GetFoodAnalysisItemsByRequestInput {
    pub realm_name: String,
    pub request_id: Uuid,
    pub user_id: Uuid,
    pub filter: GetFoodAnalysisItemFilter,
}

#[derive(Debug, Clone)]
pub struct GetFoodAnalysisItemInput {
    pub realm_name: String,
    pub item_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct GetFoodAnalysisItemsInput {
    pub realm_name: String,
    pub user_id: Uuid,
    pub filter: GetFoodAnalysisItemFilter,
}

#[derive(Debug, Clone, Default)]
pub struct GetFoodAnalysisTriggerFilter {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub trigger_category: Option<String>,
    pub risk_level: Option<String>, // 'HIGH' | 'MEDIUM' | 'LOW'
    pub risk_level_in: Option<Vec<String>>, // Multiple risk levels
    pub ingredient_name_ilike: Option<String>,
    pub sort: Option<String>, // e.g., "risk_level,-created_at"
}

#[derive(Debug, Clone, Default)]
pub struct GetTriggerCategoryFilter {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub trigger_category: Option<String>,
    pub trigger_category_in: Option<Vec<String>>,
    pub trigger_category_ilike: Option<String>,
    pub sort: Option<String>, // e.g., "-count" or "trigger_category"
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct TriggerCategoryStats {
    pub trigger_category: String,
    pub count: i64,
    pub high_risk_count: i64,
    pub medium_risk_count: i64,
    pub low_risk_count: i64,
}

#[derive(Debug, Clone)]
pub struct AnalyzeFoodInput {
    pub realm_name: String,
    pub prompt_id: Uuid,
    pub input_type: crate::domain::food_analysis::entities::InputType,
    pub text_input: Option<String>,
    pub image_data: Option<Vec<u8>>,
    pub device_id: String,
    pub user_id: Uuid,
}

/// Map safety_level to risk_score and risk_band
///
/// Mapping rules:
/// - SAFE → risk_score 0-33, risk_band SAFE
/// - CAUTION → risk_score 34-66, risk_band MODERATE
/// - UNSAFE → risk_score 67-100, risk_band HIGH
///
/// If risk_score is provided directly in the dish, use it; otherwise calculate from safety_level
pub fn map_safety_to_risk(
    safety_level: &SafetyLevel,
    provided_risk_score: Option<i32>,
) -> (i32, String) {
    // If risk_score is provided directly, use it (backward compatibility)
    if let Some(score) = provided_risk_score {
        let score = score.clamp(0, 100);
        let band = match score {
            0..=33 => "SAFE".to_string(),
            34..=66 => "MODERATE".to_string(),
            67..=100 => "HIGH".to_string(),
            _ => "SAFE".to_string(), // Fallback
        };
        return (score, band);
    }

    // Otherwise, map from safety_level
    match safety_level {
        SafetyLevel::Safe => {
            // Use middle of range: 16 (0-33)
            (16, "SAFE".to_string())
        }
        SafetyLevel::Caution => {
            // Use middle of range: 50 (34-66)
            (50, "MODERATE".to_string())
        }
        SafetyLevel::Unsafe => {
            // Use middle of range: 83 (67-100)
            (83, "HIGH".to_string())
        }
    }
}

/// Convert SafetyLevel enum to string
pub fn safety_level_to_string(safety_level: &SafetyLevel) -> String {
    match safety_level {
        SafetyLevel::Safe => "SAFE".to_string(),
        SafetyLevel::Caution => "CAUTION".to_string(),
        SafetyLevel::Unsafe => "UNSAFE".to_string(),
    }
}

/// Convert RiskIngredient to trigger category
/// For now, we'll use a simple mapping. In the future, this could be enhanced
/// to extract category from LLM response or use a more sophisticated classification
pub fn ingredient_to_trigger_category(ingredient_name: &str) -> String {
    // Simple heuristic-based categorization
    // In production, this could be enhanced with a lookup table or ML model
    let name_lower = ingredient_name.to_lowercase();

    if name_lower.contains("dairy")
        || name_lower.contains("milk")
        || name_lower.contains("cheese")
        || name_lower.contains("cream")
        || name_lower.contains("butter")
    {
        "Dairy".to_string()
    } else if name_lower.contains("gluten")
        || name_lower.contains("wheat")
        || name_lower.contains("flour")
    {
        "Gluten".to_string()
    } else if name_lower.contains("fodmap")
        || name_lower.contains("onion")
        || name_lower.contains("garlic")
        || name_lower.contains("bean")
    {
        "FODMAP".to_string()
    } else if name_lower.contains("caffeine")
        || name_lower.contains("coffee")
        || name_lower.contains("tea")
    {
        "Caffeine".to_string()
    } else if name_lower.contains("spicy")
        || name_lower.contains("pepper")
        || name_lower.contains("chili")
    {
        "Spicy".to_string()
    } else {
        "Other".to_string()
    }
}

/// Convert RiskIngredient risk_reason to risk_level
/// Simple heuristic: if reason contains "high" or "severe", use HIGH; otherwise MEDIUM or LOW
pub fn risk_reason_to_level(risk_reason: &str) -> String {
    let reason_lower = risk_reason.to_lowercase();

    if reason_lower.contains("high")
        || reason_lower.contains("severe")
        || reason_lower.contains("significant")
    {
        "HIGH".to_string()
    } else if reason_lower.contains("moderate")
        || reason_lower.contains("medium")
        || reason_lower.contains("some")
    {
        "MEDIUM".to_string()
    } else {
        "LOW".to_string()
    }
}
