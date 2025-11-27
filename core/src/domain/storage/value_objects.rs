use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateStoredObject {
    pub realm_id: Uuid,
    pub bucket: String,
    pub object_key: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    pub metadata: serde_json::Value,
    pub uploaded_by: Uuid,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct StoredObjectFilter {
    pub realm_id: Option<Uuid>,
    pub mime_type: Option<String>,
    pub uploaded_by: Option<Uuid>,
    pub created_before: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadFileInput {
    pub realm_name: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    pub metadata: serde_json::Value,
    pub use_presigned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OffsetLimit {
    pub offset: i64,
    pub limit: i64,
}

impl Default for OffsetLimit {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 20,
        }
    }
}

impl OffsetLimit {
    pub fn new(offset: i64, limit: i64) -> Self {
        Self { offset, limit }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.offset < 0 {
            return Err("offset must be >= 0".to_string());
        }
        if self.limit <= 0 || self.limit > 100 {
            return Err("limit must be between 1 and 100".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub offset: i64,
    pub limit: i64,
    pub count: i64,
}
