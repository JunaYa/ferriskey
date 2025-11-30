use std::future::Future;
use uuid::Uuid;

use crate::domain::{
    common::entities::app_errors::CoreError,
    food_stats::value_objects::{
        GetSymptomStatsFilter, GetTimelineStatsFilter, OverviewStats, SymptomStatsResponse,
        TimelineStatsResponse,
    },
};

/// Repository trait for food statistics
#[cfg_attr(test, mockall::automock)]
pub trait FoodStatsRepository: Send + Sync {
    fn get_overview_stats(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
    ) -> impl Future<Output = Result<OverviewStats, CoreError>> + Send;

    fn get_symptom_stats(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        filter: GetSymptomStatsFilter,
    ) -> impl Future<Output = Result<SymptomStatsResponse, CoreError>> + Send;

    fn get_timeline_stats(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        filter: GetTimelineStatsFilter,
    ) -> impl Future<Output = Result<TimelineStatsResponse, CoreError>> + Send;
}
