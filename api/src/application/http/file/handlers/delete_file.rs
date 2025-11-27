use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity, storage::services::FileService,
};
use uuid::Uuid;

use crate::application::http::server::{api_entities::api_error::ApiError, app_state::AppState};

#[utoipa::path(
    delete,
    path = "/{file_id}",
    tag = "file",
    summary = "Delete a file",
    responses(
        (status = 204, description = "File deleted successfully"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "File not found")
    )
)]
pub async fn delete_file(
    Path((_realm_name, file_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<axum::http::StatusCode, ApiError> {
    state.service.delete_file(identity, file_id).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
