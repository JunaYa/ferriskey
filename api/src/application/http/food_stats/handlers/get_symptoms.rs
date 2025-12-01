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
        value_objects::{GetSymptomStatsFilter, SymptomStatsResponse},
    },
    realm::ports::{GetRealmInput, RealmService},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetSymptomsResponse {
    pub items: Vec<ferriskey_core::domain::food_stats::value_objects::SymptomStats>,
    pub total_reactions: i64,
}

impl From<SymptomStatsResponse> for GetSymptomsResponse {
    fn from(response: SymptomStatsResponse) -> Self {
        Self {
            items: response.items,
            total_reactions: response.total_reactions,
        }
    }
}

#[utoipa::path(
    get,
    path = "/symptoms",
    tag = "food-stats",
    summary = "Get symptom statistics",
    description = "Get symptom statistics with filtering and sorting",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = GetSymptomsResponse)
    )
)]
pub async fn get_symptoms(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    RequiredIdentity(identity): RequiredIdentity,
    Extension(device_context): Extension<DeviceContext>,
    QueryParamsExtractor(query_params): QueryParamsExtractor,
) -> Result<Response<GetSymptomsResponse>, ApiError> {
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

    // Build filter from query params
    let mut filter = GetSymptomStatsFilter {
        sort: Some("-count".to_string()),
        ..Default::default()
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

    // Apply filters
    for cond in &query_params.filter.conditions {
        match cond.field.as_str() {
            "start_date" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq
                    && let Ok(dt) = cond.value.parse::<chrono::DateTime<chrono::Utc>>()
                {
                    filter.start_date = Some(dt);
                }
            }
            "end_date" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq
                    && let Ok(dt) = cond.value.parse::<chrono::DateTime<chrono::Utc>>()
                {
                    filter.end_date = Some(dt);
                }
            }
            "symptom_code" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq {
                    filter.symptom_code = Some(cond.value.clone());
                } else if cond.operator
                    == crate::application::http::query_params::FilterOperator::In
                {
                    filter.symptom_code_in = Some(
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

    // Get symptom stats
    let stats = state
        .stats_repository
        .get_symptom_stats(realm.id, device_context.user_id, filter)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get symptom stats: {}", e);
            ApiError::InternalServerError(format!("Failed to get symptom stats: {}", e))
        })?;

    Ok(Response::OK(GetSymptomsResponse::from(stats)))
}
