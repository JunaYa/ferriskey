use axum::{
    Extension,
    extract::{Path, State},
};

use crate::application::{
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
    authentication::value_objects::Identity,
    food_analysis::{
        entities::FoodAnalysisRequest,
        ports::FoodAnalysisService,
        value_objects::{GetFoodAnalysisFilter, GetFoodAnalysisHistoryInput},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetAnalysisRequestsResponse {
    pub items: Vec<FoodAnalysisRequest>,
    pub offset: i64,
    pub limit: i64,
    pub count: usize,
}

#[utoipa::path(
    get,
    path = "/requests",
    tag = "food-analysis",
    summary = "Get food analysis requests",
    description = "Get list of food analysis requests with filtering, sorting, and pagination",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = GetAnalysisRequestsResponse)
    )
)]
pub async fn get_analysis_requests(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Extension(device_context): Extension<DeviceContext>,
    QueryParamsExtractor(query_params): QueryParamsExtractor,
) -> Result<Response<GetAnalysisRequestsResponse>, ApiError> {
    // Build filter from query params
    let mut filter = GetFoodAnalysisFilter {
        offset: Some(query_params.pagination.offset as u32),
        limit: Some(query_params.pagination.limit as u32),
        ..Default::default()
    };

    // Sort
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
    } else {
        // Default sort: -created_at
        filter.sort = Some("-created_at".to_string());
    }

    // Apply filters from query params
    for cond in &query_params.filter.conditions {
        match cond.field.as_str() {
            "prompt_id" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq
                    && let Ok(uuid) = Uuid::parse_str(&cond.value)
                {
                    filter.prompt_id = Some(uuid);
                }
            }
            "input_type" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq {
                    filter.input_type = Some(cond.value.clone());
                }
            }
            "user_id" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq
                    && let Ok(uuid) = Uuid::parse_str(&cond.value)
                {
                    filter.user_id = Some(uuid);
                }
            }
            "created_at" => match cond.operator {
                crate::application::http::query_params::FilterOperator::Gte => {
                    if let Ok(dt) = cond.value.parse::<chrono::DateTime<chrono::Utc>>() {
                        filter.created_at_gte = Some(dt);
                    }
                }
                crate::application::http::query_params::FilterOperator::Lte => {
                    if let Ok(dt) = cond.value.parse::<chrono::DateTime<chrono::Utc>>() {
                        filter.created_at_lte = Some(dt);
                    }
                }
                _ => {}
            },
            _ => {
                // Unknown filter field, ignore
            }
        }
    }

    // If no user_id filter is specified, use device_context.user_id (automatic filtering by device)
    let user_id = filter.user_id.unwrap_or(device_context.user_id);
    filter.user_id = Some(user_id);

    let requests = state
        .service
        .get_analysis_history(identity, GetFoodAnalysisHistoryInput { realm_name, filter })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetAnalysisRequestsResponse {
        items: requests.clone(),
        offset: query_params.pagination.offset,
        limit: query_params.pagination.limit,
        count: requests.len(),
    }))
}
