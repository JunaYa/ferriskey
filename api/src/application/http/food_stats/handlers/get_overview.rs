use axum::{
    Extension,
    extract::{Path, State},
};

use crate::application::{
    device_middleware::DeviceContext,
    http::server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    food_stats::{ports::FoodStatsRepository, value_objects::OverviewStats},
    realm::ports::{GetRealmInput, RealmService},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetOverviewResponse {
    pub accuracy_level: i32,
    pub target_accuracy: i32,
    pub meals_to_target: i32,
    pub tracked_reactions: i64,
    pub triggered_foods: i64,
    pub triggers: Vec<ferriskey_core::domain::food_stats::value_objects::TriggerStats>,
    pub safe_foods: Vec<ferriskey_core::domain::food_stats::value_objects::SafeFoodStats>,
}

impl From<OverviewStats> for GetOverviewResponse {
    fn from(stats: OverviewStats) -> Self {
        Self {
            accuracy_level: stats.accuracy_level,
            target_accuracy: stats.target_accuracy,
            meals_to_target: stats.meals_to_target,
            tracked_reactions: stats.tracked_reactions,
            triggered_foods: stats.triggered_foods,
            triggers: stats.triggers,
            safe_foods: stats.safe_foods,
        }
    }
}

#[utoipa::path(
    get,
    path = "/food-stats/overview",
    tag = "food-stats",
    summary = "Get food stats overview",
    description = "Get personal trigger statistics overview including accuracy level, tracked reactions, and trigger categories",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = GetOverviewResponse)
    )
)]
pub async fn get_overview(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Extension(device_context): Extension<DeviceContext>,
) -> Result<Response<GetOverviewResponse>, ApiError> {
    // Get realm
    let realm = state
        .service
        .get_realm_by_name(
            identity.clone(),
            GetRealmInput {
                realm_name: realm_name.clone(),
            },
        )
        .await
        .map_err(ApiError::from)?;

    // Get overview stats
    let stats = state
        .stats_repository
        .get_overview_stats(realm.id, device_context.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get overview stats: {}", e);
            ApiError::InternalServerError(format!("Failed to get overview stats: {}", e))
        })?;

    Ok(Response::OK(GetOverviewResponse::from(stats)))
}
