use super::handlers::{
    create_reaction::{__path_create_reaction, create_reaction},
    delete_reaction::{__path_delete_reaction, delete_reaction},
    get_reaction::{__path_get_reaction, get_reaction},
    get_reactions::{__path_get_reactions, get_reactions},
    update_reaction::{__path_update_reaction, update_reaction},
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
    create_reaction,
    get_reactions,
    get_reaction,
    update_reaction,
    delete_reaction
))]
pub struct FoodReactionApiDoc;

pub fn food_reaction_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-reactions",
                state.args.server.root_path
            ),
            post(create_reaction).get(get_reactions),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/food-reactions/{{reaction_id}}",
                state.args.server.root_path
            ),
            get(get_reaction)
                .put(update_reaction)
                .delete(delete_reaction),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            device_middleware,
        ))
}
