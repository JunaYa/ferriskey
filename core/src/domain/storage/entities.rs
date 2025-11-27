use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_timestamp;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct StoredObject {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub bucket: String,
    pub object_key: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    pub metadata: serde_json::Value,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl StoredObject {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        realm_id: Uuid,
        bucket: String,
        object_key: String,
        original_name: String,
        mime_type: String,
        size_bytes: i64,
        checksum_sha256: String,
        metadata: serde_json::Value,
        uploaded_by: Uuid,
    ) -> Self {
        let (_, timestamp) = generate_timestamp();
        let now = Utc::now();

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id,
            bucket,
            object_key,
            original_name,
            mime_type,
            size_bytes,
            checksum_sha256,
            metadata,
            uploaded_by,
            created_at: now,
            updated_at: now,
            created_by: Some(uploaded_by),
            updated_by: Some(uploaded_by),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PresignedUrl {
    pub url: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum UploadNegotiation {
    #[serde(rename = "direct")]
    Direct { object_id: Uuid, upload_url: String },
    #[serde(rename = "presigned")]
    Presigned {
        object_id: Uuid,
        presigned_url: PresignedUrl,
    },
}
