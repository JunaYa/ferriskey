use axum::{
    Extension,
    extract::{Path, State},
};

use crate::application::{
    auth::RequiredIdentity,
    device_middleware::DeviceContext,
    http::{
        query_extractor::QueryParamsExtractor,
        server::{
            api_entities::{api_error::ApiError, response::Response},
            app_state::AppState,
        },
    },
};
use ferriskey_core::domain::{
    food_stats::{
        ports::FoodStatsRepository,
        value_objects::{GetTimelineStatsFilter, TimelineStatsResponse},
    },
    realm::ports::{GetRealmInput, RealmService},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetTimelineResponse {
    pub items: Vec<ferriskey_core::domain::food_stats::value_objects::TimelineStats>,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
}

impl From<TimelineStatsResponse> for GetTimelineResponse {
    fn from(response: TimelineStatsResponse) -> Self {
        Self {
            items: response.items,
            start_date: response.start_date,
            end_date: response.end_date,
        }
    }
}

#[utoipa::path(
    get,
    path = "/timeline",
    tag = "food-stats",
    summary = "Get timeline statistics",
    description = "Get time series statistics with date range filtering",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = GetTimelineResponse),
        (status = 400, description = "Bad request - missing required parameters")
    )
)]
pub async fn get_timeline(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    RequiredIdentity(identity): RequiredIdentity,
    Extension(device_context): Extension<DeviceContext>,
    QueryParamsExtractor(query_params): QueryParamsExtractor,
) -> Result<Response<GetTimelineResponse>, ApiError> {
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

    // Extract required parameters
    let mut start_date: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut end_date: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut granularity = "day".to_string();
    let mut feeling_in: Option<Vec<String>> = None;

    for cond in &query_params.filter.conditions {
        match cond.field.as_str() {
            "start_date" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq
                    && let Ok(dt) = cond.value.parse::<chrono::DateTime<chrono::Utc>>()
                {
                    start_date = Some(dt);
                }
            }
            "end_date" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq
                    && let Ok(dt) = cond.value.parse::<chrono::DateTime<chrono::Utc>>()
                {
                    end_date = Some(dt);
                }
            }
            "granularity" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq {
                    granularity = cond.value.clone();
                }
            }
            "feeling" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::In {
                    feeling_in = Some(
                        cond.value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect(),
                    );
                }
            }
            _ => {}
        }
    }

    // Validate required parameters
    let start_date =
        start_date.ok_or_else(|| ApiError::BadRequest("start_date is required".to_string()))?;
    let end_date =
        end_date.ok_or_else(|| ApiError::BadRequest("end_date is required".to_string()))?;

    // Build filter
    let mut filter = GetTimelineStatsFilter {
        start_date,
        end_date,
        granularity,
        feeling_in,
        sort: Some("date".to_string()),
    };

    // Apply sort
    if !query_params.sort.is_empty() {
        let sort_str = query_params
            .sort
            .sorts
            .iter()
            .map(|s| {
                if s.direction == crate::application::http::query_params::SortDirection::Desc {
                    format!("-{}", s.field)
                } else {
                    s.field.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        filter.sort = Some(sort_str);
    }

    // Get timeline stats
    let stats = state
        .stats_repository
        .get_timeline_stats(realm.id, device_context.user_id, filter)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get timeline stats: {}", e);
            ApiError::InternalServerError(format!("Failed to get timeline stats: {}", e))
        })?;

    Ok(Response::OK(GetTimelineResponse::from(stats)))
}
