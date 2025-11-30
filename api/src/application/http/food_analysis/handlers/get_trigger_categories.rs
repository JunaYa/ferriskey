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
        ports::FoodAnalysisTriggerRepository,
        value_objects::{GetTriggerCategoryFilter, TriggerCategoryStats},
    },
    realm::ports::{GetRealmInput, RealmService},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetTriggerCategoriesResponse {
    pub items: Vec<TriggerCategoryStats>,
}

#[utoipa::path(
    get,
    path = "/triggers/categories",
    tag = "food-analysis",
    summary = "Get trigger category statistics",
    description = "Get statistics of trigger categories with filtering, sorting, and pagination",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = GetTriggerCategoriesResponse)
    )
)]
pub async fn get_trigger_categories(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Extension(device_context): Extension<DeviceContext>,
    QueryParamsExtractor(query_params): QueryParamsExtractor,
) -> Result<Response<GetTriggerCategoriesResponse>, ApiError> {
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
    let mut filter = GetTriggerCategoryFilter {
        offset: Some(query_params.pagination.offset as u32),
        limit: Some(query_params.pagination.limit as u32),
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
        // Default sort: -count
        filter.sort = Some("-count".to_string());
    }

    // Apply filters
    for cond in &query_params.filter.conditions {
        if cond.field.as_str() == "trigger_category" {
            if cond.operator == crate::application::http::query_params::FilterOperator::Eq {
                filter.trigger_category = Some(cond.value.clone());
            } else if cond.operator == crate::application::http::query_params::FilterOperator::In {
                filter.trigger_category_in = Some(
                    cond.value
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect(),
                );
            } else if cond.operator == crate::application::http::query_params::FilterOperator::Ilike
            {
                filter.trigger_category_ilike = Some(cond.value.clone());
            }
        }
    }

    // Get category stats
    let stats = state
        .trigger_repository
        .get_categories_stats(realm.id, device_context.user_id, filter)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get trigger category stats: {}", e);
            ApiError::InternalServerError(format!("Failed to get trigger category stats: {}", e))
        })?;

    Ok(Response::OK(GetTriggerCategoriesResponse { items: stats }))
}
