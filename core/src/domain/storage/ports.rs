use std::future::Future;
use std::time::Duration;

use bytes::Bytes;
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;

use super::entities::{PresignedUrl, StoredObject};
use super::value_objects::{CreateStoredObject, OffsetLimit, Paginated, StoredObjectFilter};

/// Port for object storage operations (MinIO/S3)
#[cfg_attr(test, mockall::automock)]
pub trait ObjectStoragePort: Send + Sync {
    /// Generate a bucket name for a given realm name
    fn bucket_name(&self, realm_name: &str) -> String;

    /// Upload an object directly to storage
    fn put_object(
        &self,
        bucket: &str,
        object_key: &str,
        payload: Bytes,
        content_type: &str,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Generate a presigned PUT URL for client-side uploads
    fn presign_put_url(
        &self,
        bucket: &str,
        object_key: &str,
        expires_in: Duration,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;

    /// Generate a presigned GET URL for downloads
    fn presign_get_url(
        &self,
        bucket: &str,
        object_key: &str,
        expires_in: Duration,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;

    /// Delete an object from storage
    fn delete_object(
        &self,
        bucket: &str,
        object_key: &str,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

/// Repository for stored object metadata
#[cfg_attr(test, mockall::automock)]
pub trait StoredObjectRepository: Send + Sync {
    /// Create a new stored object record
    fn create(
        &self,
        input: CreateStoredObject,
    ) -> impl Future<Output = Result<StoredObject, CoreError>> + Send;

    /// List stored objects with filtering and pagination
    fn list(
        &self,
        filter: StoredObjectFilter,
        pagination: OffsetLimit,
    ) -> impl Future<Output = Result<Paginated<StoredObject>, CoreError>> + Send;

    /// Get a stored object by ID
    fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<StoredObject, CoreError>> + Send;

    /// Delete a stored object record
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;
}
