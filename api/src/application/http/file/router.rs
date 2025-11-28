use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, post},
};
use utoipa::OpenApi;

use crate::application::{auth::auth, http::server::app_state::AppState};

use super::handlers::{
    complete_upload::{__path_complete_upload, complete_upload},
    delete_file::{__path_delete_file, delete_file},
    get_download_url::{__path_get_download_url, get_download_url},
    initiate_upload::{__path_initiate_upload, initiate_upload},
    list_files::{__path_list_files, list_files},
    upload_file::{__path_upload_file, upload_file},
};

#[derive(OpenApi)]
#[openapi(paths(
    initiate_upload,
    complete_upload,
    list_files,
    get_download_url,
    delete_file,
    upload_file
))]
pub struct FileApiDoc;

pub fn file_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/files/uploads",
                state.args.server.root_path
            ),
            post(initiate_upload),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/files/upload",
                state.args.server.root_path
            ),
            post(upload_file),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/files/{{file_id}}/complete",
                state.args.server.root_path
            ),
            post(complete_upload),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/files",
                state.args.server.root_path
            ),
            get(list_files),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/files/{{file_id}}/download",
                state.args.server.root_path
            ),
            get(get_download_url),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/files/{{file_id}}",
                state.args.server.root_path
            ),
            delete(delete_file),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
        // Limit request body size to 50 MB for file uploads
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
}
