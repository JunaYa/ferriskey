use std::time::Duration;

use aws_sdk_s3::{
    Client,
    config::{BehaviorVersion, Credentials, Region},
    presigning::PresigningConfig,
    primitives::ByteStream,
};
use bytes::Bytes;
use tracing::instrument;

use crate::domain::{
    common::entities::app_errors::CoreError,
    storage::{entities::PresignedUrl, ports::ObjectStoragePort},
};

#[derive(Debug, Clone)]
pub struct ObjectStorageConfig {
    pub endpoint: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket_prefix: String,
    pub use_ssl: bool,
}

#[derive(Clone)]
pub struct MinioObjectStorage {
    client: Client,
}

impl MinioObjectStorage {
    pub async fn new(config: ObjectStorageConfig) -> Self {
        let credentials = Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "ferriskey",
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .credentials_provider(credentials)
            .endpoint_url(&config.endpoint)
            .force_path_style(true)
            .build();

        let client = Client::from_conf(s3_config);

        Self { client }
    }
}

impl ObjectStoragePort for MinioObjectStorage {
    #[instrument(skip(self, payload))]
    async fn put_object(
        &self,
        bucket: &str,
        object_key: &str,
        payload: Bytes,
        content_type: &str,
    ) -> Result<(), CoreError> {
        tracing::info!(
            bucket = %bucket,
            object_key = %object_key,
            size = payload.len(),
            "Uploading object to storage"
        );

        let byte_stream = ByteStream::from(payload);

        self.client
            .put_object()
            .bucket(bucket)
            .key(object_key)
            .body(byte_stream)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    bucket = %bucket,
                    object_key = %object_key,
                    "Failed to upload object"
                );
                CoreError::ObjectStorageError(format!("Failed to upload object: {}", e))
            })?;

        tracing::info!(
            bucket = %bucket,
            object_key = %object_key,
            "Object uploaded successfully"
        );

        Ok(())
    }

    #[instrument(skip(self))]
    async fn presign_put_url(
        &self,
        bucket: &str,
        object_key: &str,
        expires_in: Duration,
    ) -> Result<PresignedUrl, CoreError> {
        tracing::debug!(
            bucket = %bucket,
            object_key = %object_key,
            expires_in_secs = expires_in.as_secs(),
            "Generating presigned PUT URL"
        );

        let presigning_config = PresigningConfig::expires_in(expires_in)
            .map_err(|e| CoreError::ObjectStorageError(format!("Invalid expiration: {}", e)))?;

        let presigned_request = self
            .client
            .put_object()
            .bucket(bucket)
            .key(object_key)
            .presigned(presigning_config)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    bucket = %bucket,
                    object_key = %object_key,
                    "Failed to generate presigned PUT URL"
                );
                CoreError::ObjectStorageError(format!("Failed to generate presigned URL: {}", e))
            })?;

        Ok(PresignedUrl {
            url: presigned_request.uri().to_string(),
            expires_in_seconds: expires_in.as_secs(),
        })
    }

    #[instrument(skip(self))]
    async fn presign_get_url(
        &self,
        bucket: &str,
        object_key: &str,
        expires_in: Duration,
    ) -> Result<PresignedUrl, CoreError> {
        tracing::debug!(
            bucket = %bucket,
            object_key = %object_key,
            expires_in_secs = expires_in.as_secs(),
            "Generating presigned GET URL"
        );

        let presigning_config = PresigningConfig::expires_in(expires_in)
            .map_err(|e| CoreError::ObjectStorageError(format!("Invalid expiration: {}", e)))?;

        let presigned_request = self
            .client
            .get_object()
            .bucket(bucket)
            .key(object_key)
            .presigned(presigning_config)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    bucket = %bucket,
                    object_key = %object_key,
                    "Failed to generate presigned GET URL"
                );
                CoreError::ObjectStorageError(format!("Failed to generate presigned URL: {}", e))
            })?;

        Ok(PresignedUrl {
            url: presigned_request.uri().to_string(),
            expires_in_seconds: expires_in.as_secs(),
        })
    }

    #[instrument(skip(self))]
    async fn delete_object(&self, bucket: &str, object_key: &str) -> Result<(), CoreError> {
        tracing::info!(
            bucket = %bucket,
            object_key = %object_key,
            "Deleting object from storage"
        );

        self.client
            .delete_object()
            .bucket(bucket)
            .key(object_key)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    bucket = %bucket,
                    object_key = %object_key,
                    "Failed to delete object"
                );
                CoreError::ObjectStorageError(format!("Failed to delete object: {}", e))
            })?;

        tracing::info!(
            bucket = %bucket,
            object_key = %object_key,
            "Object deleted successfully"
        );

        Ok(())
    }
}
