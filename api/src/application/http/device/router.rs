use super::handlers::{__path_get_device, get_device};
use crate::application::{auth::auth, http::server::app_state::AppState};
use axum::{Router, middleware, routing::get};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(get_device))]
pub struct DeviceApiDoc;

pub fn device_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/devices/{{device_id}}",
                state.args.server.root_path
            ),
            get(get_device),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
