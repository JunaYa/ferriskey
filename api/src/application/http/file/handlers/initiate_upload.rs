use axum::{
    Extension, Json,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    storage::services::FileService,
    storage::{entities::UploadNegotiation, value_objects::UploadFileInput},
};

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

use crate::application::http::file::validators::InitiateUploadRequest;

#[utoipa::path(
    post,
    path = "/uploads",
    tag = "file",
    summary = "Initiate a file upload",
    request_body = InitiateUploadRequest,
    responses(
        (status = 200, description = "Upload initiated successfully", body = UploadNegotiation),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 413, description = "File too large")
    )
)]
pub async fn initiate_upload(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(req): Json<InitiateUploadRequest>,
) -> Result<Response<UploadNegotiation>, ApiError> {
    let input = UploadFileInput {
        realm_name,
        filename: req.filename,
        mime_type: req.mime_type,
        size_bytes: req.size_bytes,
        checksum_sha256: req.checksum_sha256,
        metadata: req.metadata.unwrap_or_default(),
        use_presigned: req.use_presigned.unwrap_or(false),
    };

    let negotiation = state.service.initiate_upload(identity, input).await?;

    Ok(Response::OK(negotiation))
}
