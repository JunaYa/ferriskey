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
    food_reaction::{
        entities::FoodReaction, ports::FoodReactionRepository, value_objects::GetFoodReactionFilter,
    },
    realm::ports::RealmRepository,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetReactionsResponse {
    pub items: Vec<FoodReaction>,
    pub offset: i64,
    pub limit: i64,
    pub count: usize,
}

#[utoipa::path(
    get,
    path = "/food-reactions",
    tag = "food-reaction",
    summary = "Get food reactions",
    description = "Get list of food reactions with filtering, sorting, and pagination",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = GetReactionsResponse)
    )
)]
pub async fn get_reactions(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(device_context): Extension<DeviceContext>,
    QueryParamsExtractor(query_params): QueryParamsExtractor,
) -> Result<Response<GetReactionsResponse>, ApiError> {
    // Get realm
    let realm = state
        .realm_repository
        .get_by_name(realm_name.clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to get realm: {}", e);
            ApiError::InternalServerError(format!("Failed to get realm: {}", e))
        })?
        .ok_or_else(|| {
            tracing::error!("Realm not found: {}", realm_name);
            ApiError::NotFound(format!("Realm '{}' not found", realm_name))
        })?;

    // Build filter from query params
    let mut filter = GetFoodReactionFilter {
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
        // Default sort: -eaten_at
        filter.sort = Some("-eaten_at".to_string());
    }

    // Apply filters
    for cond in &query_params.filter.conditions {
        match cond.field.as_str() {
            "feeling" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq {
                    filter.feeling = Some(cond.value.clone());
                } else if cond.operator
                    == crate::application::http::query_params::FilterOperator::In
                {
                    filter.feeling_in = Some(
                        cond.value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect(),
                    );
                }
            }
            "analysis_item_id" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq
                    && let Ok(uuid) = Uuid::parse_str(&cond.value)
                {
                    filter.analysis_item_id = Some(uuid);
                }
            }
            "symptom_onset" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq {
                    filter.symptom_onset = Some(cond.value.clone());
                }
            }
            "eaten_at" => match cond.operator {
                crate::application::http::query_params::FilterOperator::Gte => {
                    if let Ok(dt) = cond.value.parse::<chrono::DateTime<chrono::Utc>>() {
                        filter.eaten_at_gte = Some(dt);
                    }
                }
                crate::application::http::query_params::FilterOperator::Lte => {
                    if let Ok(dt) = cond.value.parse::<chrono::DateTime<chrono::Utc>>() {
                        filter.eaten_at_lte = Some(dt);
                    }
                }
                _ => {}
            },
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
            "has_symptoms" => {
                if cond.operator == crate::application::http::query_params::FilterOperator::Eq
                    && let Ok(has) = cond.value.parse::<bool>()
                {
                    filter.has_symptoms = Some(has);
                }
            }
            _ => {}
        }
    }

    // Get reactions
    let reactions = state
        .reaction_repository
        .get_by_realm(realm.id, device_context.user_id, filter)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get food reactions: {}", e);
            ApiError::InternalServerError(format!("Failed to get food reactions: {}", e))
        })?;

    Ok(Response::OK(GetReactionsResponse {
        items: reactions.clone(),
        offset: query_params.pagination.offset,
        limit: query_params.pagination.limit,
        count: reactions.len(),
    }))
}
