use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    storage::{entities::PresignedUrl, services::FileService},
};
use uuid::Uuid;

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

#[utoipa::path(
    get,
    path = "/{file_id}/download",
    tag = "file",
    summary = "Get a presigned download URL for a file",
    responses(
        (status = 200, description = "Download URL generated successfully", body = PresignedUrl),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "File not found")
    )
)]
pub async fn get_download_url(
    Path((_realm_name, file_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<PresignedUrl>, ApiError> {
    let presigned_url = state.service.get_download_url(identity, file_id).await?;

    Ok(Response::OK(presigned_url))
}
