use super::handlers::{
    get_overview::{__path_get_overview, get_overview},
    get_symptoms::{__path_get_symptoms, get_symptoms},
    get_timeline::{__path_get_timeline, get_timeline},
};
use crate::application::{
    auth::auth, device_middleware::device_middleware, http::server::app_state::AppState,
};
use axum::{Router, middleware, routing::get};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(get_overview, get_symptoms, get_timeline))]
pub struct FoodStatsApiDoc;

pub fn food_stats_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-stats/overview",
                state.args.server.root_path
            ),
            get(get_overview),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-stats/symptoms",
                state.args.server.root_path
            ),
            get(get_symptoms),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-stats/timeline",
                state.args.server.root_path
            ),
            get(get_timeline),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            device_middleware,
        ))
}
