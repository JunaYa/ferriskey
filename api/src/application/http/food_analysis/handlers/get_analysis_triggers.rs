use axum::{
    Extension,
    extract::{Path, State},
};

use crate::application::http::{
    query_extractor::QueryParamsExtractor,
    server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    food_analysis::{
        entities::FoodAnalysisTrigger, ports::FoodAnalysisTriggerRepository,
        value_objects::GetFoodAnalysisTriggerFilter,
    },
    realm::ports::{GetRealmInput, RealmService},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetAnalysisTriggersResponse {
    pub items: Vec<FoodAnalysisTrigger>,
}

#[utoipa::path(
    get,
    path = "/items/{item_id}/triggers",
    tag = "food-analysis",
    summary = "Get food analysis triggers by item",
    description = "Get list of food analysis triggers for a specific item with filtering, sorting, and pagination",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("item_id" = Uuid, Path, description = "Item ID"),
    ),
    responses(
        (status = 200, body = GetAnalysisTriggersResponse)
    )
)]
pub async fn get_analysis_triggers(
    Path((realm_name, item_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    QueryParamsExtractor(query_params): QueryParamsExtractor,
) -> Result<Response<GetAnalysisTriggersResponse>, ApiError> {
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
    let mut filter = GetFoodAnalysisTriggerFilter {
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
        // Default sort: risk_level,-created_at
        filter.sort = Some("risk_level,-created_at".to_string());
    }

    // Apply filters
    for cond in &query_params.filter.conditions {
        match cond.field.as_str() {
            "trigger_category" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq {
                    filter.trigger_category = Some(cond.value.clone());
                }
            }
            "risk_level" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq {
                    filter.risk_level = Some(cond.value.clone());
                } else if cond.operator
                    == crate::application::http::query_params::FilterOperator::In
                {
                    filter.risk_level_in = Some(
                        cond.value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect(),
                    );
                }
            }
            "ingredient_name" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Ilike {
                    filter.ingredient_name_ilike = Some(cond.value.clone());
                }
            }
            _ => {}
        }
    }

    // Get triggers
    let triggers = state
        .trigger_repository
        .get_by_item_id(item_id, realm.id, filter)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get food analysis triggers: {}", e);
            ApiError::InternalServerError(format!("Failed to get food analysis triggers: {}", e))
        })?;

    Ok(Response::OK(GetAnalysisTriggersResponse {
        items: triggers,
    }))
}
