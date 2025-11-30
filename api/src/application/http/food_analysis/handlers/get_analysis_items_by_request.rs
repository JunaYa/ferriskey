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
        entities::FoodAnalysisItem, ports::FoodAnalysisItemRepository,
        value_objects::GetFoodAnalysisItemFilter,
    },
    realm::ports::{GetRealmInput, RealmService},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetAnalysisItemsByRequestResponse {
    pub items: Vec<FoodAnalysisItem>,
    pub offset: i64,
    pub limit: i64,
    pub count: usize,
}

#[utoipa::path(
    get,
    path = "/requests/{request_id}/items",
    tag = "food-analysis",
    summary = "Get food analysis items by request",
    description = "Get list of food analysis items for a specific request",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("request_id" = Uuid, Path, description = "Request ID"),
    ),
    responses(
        (status = 200, body = GetAnalysisItemsByRequestResponse)
    )
)]
pub async fn get_analysis_items_by_request(
    Path((realm_name, request_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Extension(device_context): Extension<DeviceContext>,
    QueryParamsExtractor(query_params): QueryParamsExtractor,
) -> Result<Response<GetAnalysisItemsByRequestResponse>, ApiError> {
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
    let mut filter = GetFoodAnalysisItemFilter {
        offset: Some(query_params.pagination.offset as u32),
        limit: Some(query_params.pagination.limit as u32),
        include_reaction_info: query_params
            .filter
            .conditions
            .iter()
            .find(|c| c.field == "include_reaction_info")
            .and_then(|c| c.value.parse::<bool>().ok())
            .unwrap_or(true),
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
    } else {
        // Default sort: dish_index
        filter.sort = Some("dish_index".to_string());
    }

    // Apply filters
    for cond in &query_params.filter.conditions {
        match cond.field.as_str() {
            "risk_band" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq {
                    filter.risk_band = Some(cond.value.clone());
                } else if cond.operator
                    == crate::application::http::query_params::FilterOperator::In
                {
                    filter.risk_band_in = Some(
                        cond.value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect(),
                    );
                }
            }
            "safety_level" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq {
                    filter.safety_level = Some(cond.value.clone());
                }
            }
            "risk_score" => match cond.operator {
                crate::application::http::query_params::FilterOperator::Gte => {
                    if let Ok(score) = cond.value.parse::<i32>() {
                        filter.risk_score_gte = Some(score);
                    }
                }
                crate::application::http::query_params::FilterOperator::Lte => {
                    if let Ok(score) = cond.value.parse::<i32>() {
                        filter.risk_score_lte = Some(score);
                    }
                }
                _ => {}
            },
            "dish_name" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Ilike {
                    filter.dish_name_ilike = Some(cond.value.clone());
                }
            }
            _ => {}
        }
    }

    // Get items
    let items = state
        .item_repository
        .get_by_request_id(request_id, realm.id, device_context.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get food analysis items: {}", e);
            ApiError::InternalServerError(format!("Failed to get food analysis items: {}", e))
        })?;

    Ok(Response::OK(GetAnalysisItemsByRequestResponse {
        items: items.clone(),
        offset: query_params.pagination.offset,
        limit: query_params.pagination.limit,
        count: items.len(),
    }))
}
