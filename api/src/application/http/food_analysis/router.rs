use super::handlers::{
    analyze_food_image::{__path_analyze_food_image, analyze_food_image},
    analyze_food_text::{__path_analyze_food_text, analyze_food_text},
    get_analysis_history::{__path_get_analysis_history, get_analysis_history},
    get_analysis_item::{__path_get_analysis_item, get_analysis_item},
    get_analysis_items::{__path_get_analysis_items, get_analysis_items},
    get_analysis_items_by_request::{
        __path_get_analysis_items_by_request, get_analysis_items_by_request,
    },
    get_analysis_request::{__path_get_analysis_request, get_analysis_request},
    get_analysis_requests::{__path_get_analysis_requests, get_analysis_requests},
    get_analysis_result::{__path_get_analysis_result, get_analysis_result},
    get_analysis_triggers::{__path_get_analysis_triggers, get_analysis_triggers},
    get_trigger_categories::{__path_get_trigger_categories, get_trigger_categories},
};
use crate::application::{
    auth::auth, device_middleware::device_middleware, http::server::app_state::AppState,
};
use axum::{
    Router, middleware,
    routing::{get, post},
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    analyze_food_text,
    analyze_food_image,
    get_analysis_history,
    get_analysis_requests,
    get_analysis_request,
    get_analysis_result,
    get_analysis_items_by_request,
    get_analysis_item,
    get_analysis_items,
    get_analysis_triggers,
    get_trigger_categories
))]
pub struct FoodAnalysisApiDoc;

pub fn food_analysis_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-analysis/text",
                state.args.server.root_path
            ),
            post(analyze_food_text),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-analysis/image",
                state.args.server.root_path
            ),
            post(analyze_food_image),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-analysis",
                state.args.server.root_path
            ),
            get(get_analysis_history),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-analysis/requests",
                state.args.server.root_path
            ),
            get(get_analysis_requests),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-analysis/requests/{{request_id}}",
                state.args.server.root_path
            ),
            get(get_analysis_request),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-analysis/{{request_id}}/result",
                state.args.server.root_path
            ),
            get(get_analysis_result),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-analysis/requests/{{request_id}}/items",
                state.args.server.root_path
            ),
            get(get_analysis_items_by_request),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-analysis/items/{{item_id}}",
                state.args.server.root_path
            ),
            get(get_analysis_item),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-analysis/items",
                state.args.server.root_path
            ),
            get(get_analysis_items),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-analysis/items/{{item_id}}/triggers",
                state.args.server.root_path
            ),
            get(get_analysis_triggers),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-analysis/triggers/categories",
                state.args.server.root_path
            ),
            get(get_trigger_categories),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            device_middleware,
        ))
}
