use chrono::DateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct OverviewStats {
    pub accuracy_level: i32,  // 0-100
    pub target_accuracy: i32, // 目标准确度，默认 85
    pub meals_to_target: i32, // 为达到目标还需记录的餐次数估算
    pub tracked_reactions: i64,
    pub triggered_foods: i64,
    pub triggers: Vec<TriggerStats>,
    pub safe_foods: Vec<SafeFoodStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct TriggerStats {
    pub trigger_category: String,
    pub emoji: Option<String>,
    pub issue_count: i64,
    pub total_exposures: i64,
    pub risk_percent: i32, // 0-100
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct SafeFoodStats {
    pub trigger_category: String,
    pub emoji: Option<String>,
    pub safe_exposures: i64,
}

#[derive(Debug, Clone, Default)]
pub struct GetSymptomStatsFilter {
    pub start_date: Option<DateTime<chrono::Utc>>,
    pub end_date: Option<DateTime<chrono::Utc>>,
    pub symptom_code: Option<String>,
    pub symptom_code_in: Option<Vec<String>>,
    pub sort: Option<String>, // e.g., "-count" or "symptom_code"
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct SymptomStats {
    pub symptom_code: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct SymptomStatsResponse {
    pub items: Vec<SymptomStats>,
    pub total_reactions: i64,
}

#[derive(Debug, Clone, Default)]
pub struct GetTimelineStatsFilter {
    pub start_date: DateTime<chrono::Utc>, // Required
    pub end_date: DateTime<chrono::Utc>,   // Required
    pub granularity: String,               // 'day' | 'week' | 'month', default 'day'
    pub feeling_in: Option<Vec<String>>,
    pub sort: Option<String>, // e.g., "date" or "-total_reactions"
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct TimelineStats {
    pub date: String, // ISO date string (YYYY-MM-DD)
    pub total_reactions: i64,
    pub positive_reactions: i64,
    pub negative_reactions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct TimelineStatsResponse {
    pub items: Vec<TimelineStats>,
    pub start_date: DateTime<chrono::Utc>,
    pub end_date: DateTime<chrono::Utc>,
}
