use sha2::{Digest, Sha256};
use std::future::Future;
use std::time::Duration;
use tracing::instrument;
use uuid::Uuid;

use crate::domain::{
    authentication::{ports::AuthSessionRepository, value_objects::Identity},
    client::ports::{ClientRepository, RedirectUriRepository},
    common::{
        entities::app_errors::CoreError, generate_random_string, policies::ensure_policy,
        services::Service,
    },
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    food_analysis::ports::{FoodAnalysisRepository, LLMClient},
    health::ports::HealthCheckRepository,
    jwt::ports::{KeyStoreRepository, RefreshTokenRepository},
    prompt::ports::PromptRepository,
    realm::ports::RealmRepository,
    role::ports::RoleRepository,
    seawatch::SecurityEventRepository,
    storage::{
        entities::{StoredObject, UploadNegotiation},
        policies::FilePolicy,
        ports::{ObjectStoragePort, StoredObjectRepository},
        value_objects::{
            CreateStoredObject, OffsetLimit, Paginated, StoredObjectFilter, UploadFileInput,
        },
    },
    trident::ports::RecoveryCodeRepository,
    user::ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
    webhook::ports::WebhookRepository,
};

use super::entities::PresignedUrl;

/// Service trait for file storage operations
#[cfg_attr(test, mockall::automock)]
pub trait FileService: Send + Sync {
    /// Initiate a file upload
    fn initiate_upload(
        &self,
        identity: Identity,
        input: UploadFileInput,
    ) -> impl Future<Output = Result<UploadNegotiation, CoreError>> + Send;

    /// Complete a file upload
    fn complete_upload(
        &self,
        identity: Identity,
        object_id: Uuid,
    ) -> impl Future<Output = Result<StoredObject, CoreError>> + Send;

    /// List files with filtering and pagination
    fn list_files(
        &self,
        identity: Identity,
        filter: StoredObjectFilter,
        pagination: OffsetLimit,
    ) -> impl Future<Output = Result<Paginated<StoredObject>, CoreError>> + Send;

    /// Get a download URL for a file
    fn get_download_url(
        &self,
        identity: Identity,
        object_id: Uuid,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;

    /// Delete a file
    fn delete_file(
        &self,
        identity: Identity,
        object_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Upload a file directly (multipart form)
    fn upload_file_direct(
        &self,
        identity: Identity,
        realm_name: String,
        filename: String,
        mime_type: String,
        file_data: bytes::Bytes,
        metadata: serde_json::Value,
    ) -> impl Future<Output = Result<StoredObject, CoreError>> + Send;
}

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC, SE, PR, FA, LLM, OS, SO> FileService
    for Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC, SE, PR, FA, LLM, OS, SO>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    RU: RedirectUriRepository,
    RO: RoleRepository,
    KS: KeyStoreRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    HC: HealthCheckRepository,
    W: WebhookRepository,
    RT: RefreshTokenRepository,
    RC: RecoveryCodeRepository,
    SE: SecurityEventRepository,
    PR: PromptRepository,
    FA: FoodAnalysisRepository,
    LLM: LLMClient,
    OS: ObjectStoragePort,
    SO: StoredObjectRepository,
    OS: ObjectStoragePort,
    SO: StoredObjectRepository,
    OS: ObjectStoragePort,
    SO: StoredObjectRepository,
{
    #[instrument(skip(self), fields(realm_name = %input.realm_name))]
    async fn initiate_upload(
        &self,
        identity: Identity,
        input: UploadFileInput,
    ) -> Result<UploadNegotiation, CoreError> {
        // Validate file size
        if input.size_bytes > 52_428_800 {
            // 50 MB
            return Err(CoreError::FileTooLarge);
        }

        // Validate pagination
        if input.size_bytes < 0 {
            return Err(CoreError::Invalid);
        }

        // Get realm
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        // Check permissions
        ensure_policy(
            self.policy.can_upload(&identity, &realm).await,
            "insufficient permissions to upload files",
        )?;

        // Generate object key
        let object_key = format!(
            "{}/{}/{}",
            realm.id,
            generate_random_string(16),
            input.filename
        );

        let bucket = format!("ferriskey-{}", realm.name);

        // Create metadata record
        let create_input = CreateStoredObject {
            realm_id: realm.id,
            bucket: bucket.clone(),
            object_key: object_key.clone(),
            original_name: input.filename.clone(),
            mime_type: input.mime_type.clone(),
            size_bytes: input.size_bytes,
            checksum_sha256: input.checksum_sha256,
            metadata: input.metadata,
            uploaded_by: identity.id(),
        };

        let stored_object = self.stored_object_repository.create(create_input).await?;

        tracing::info!(
            object_id = %stored_object.id,
            bucket = %bucket,
            object_key = %object_key,
            "File upload initiated"
        );

        if input.use_presigned {
            // Generate presigned URL for client-side upload
            let presigned_url = self
                .object_storage
                .presign_put_url(&bucket, &object_key, Duration::from_secs(300))
                .await?;

            Ok(UploadNegotiation::Presigned {
                object_id: stored_object.id,
                presigned_url,
            })
        } else {
            // Return direct upload URL (will be handled by API layer)
            Ok(UploadNegotiation::Direct {
                object_id: stored_object.id,
                upload_url: format!("/files/{}/upload", stored_object.id),
            })
        }
    }

    #[instrument(skip(self))]
    async fn complete_upload(
        &self,
        identity: Identity,
        object_id: Uuid,
    ) -> Result<StoredObject, CoreError> {
        let stored_object = self.stored_object_repository.get_by_id(object_id).await?;

        // Get user from identity and verify realm access
        let user = self.user_repository.get_by_id(identity.id()).await?;

        let realm = user.realm.ok_or(CoreError::InvalidRealm)?;

        // Verify the file belongs to the user's realm or they have cross-realm access
        if stored_object.realm_id != realm.id && realm.name != "master" {
            return Err(CoreError::Forbidden(
                "File not in accessible realm".to_string(),
            ));
        }

        // Check permissions
        ensure_policy(
            self.policy.can_upload(&identity, &realm).await,
            "insufficient permissions to complete upload",
        )?;

        // Verify the user is the one who initiated the upload
        if stored_object.uploaded_by != identity.id() {
            return Err(CoreError::Forbidden(
                "cannot complete upload for another user".to_string(),
            ));
        }

        tracing::info!(
            object_id = %object_id,
            "File upload completed"
        );

        Ok(stored_object)
    }

    #[instrument(skip(self))]
    async fn list_files(
        &self,
        identity: Identity,
        filter: StoredObjectFilter,
        pagination: OffsetLimit,
    ) -> Result<Paginated<StoredObject>, CoreError> {
        // Validate pagination
        pagination
            .validate()
            .map_err(|_| CoreError::InvalidPagination)?;

        // Get realm from filter
        let realm_id = filter.realm_id.ok_or(CoreError::InvalidRealm)?;

        // Get user from identity and verify realm access
        let user = self.user_repository.get_by_id(identity.id()).await?;

        let realm = user.realm.ok_or(CoreError::InvalidRealm)?;

        // Verify filtering on accessible realm
        if realm_id != realm.id && realm.name != "master" {
            return Err(CoreError::Forbidden(
                "Cannot list files in inaccessible realm".to_string(),
            ));
        }

        // Check permissions
        ensure_policy(
            self.policy.can_view(&identity, &realm).await,
            "insufficient permissions to list files",
        )?;

        // List files
        let result = self
            .stored_object_repository
            .list(filter, pagination)
            .await?;

        tracing::debug!(
            count = result.items.len(),
            total = result.count,
            "Listed files"
        );

        Ok(result)
    }

    #[instrument(skip(self))]
    async fn get_download_url(
        &self,
        identity: Identity,
        object_id: Uuid,
    ) -> Result<PresignedUrl, CoreError> {
        let stored_object = self.stored_object_repository.get_by_id(object_id).await?;

        // Get user to check realm access
        let user = self.user_repository.get_by_id(identity.id()).await?;

        let realm = user.realm.ok_or(CoreError::InvalidRealm)?;

        // Verify the file belongs to the user's realm or they have cross-realm access
        if stored_object.realm_id != realm.id && realm.name != "master" {
            return Err(CoreError::Forbidden(
                "File not in accessible realm".to_string(),
            ));
        }

        // Check permissions
        ensure_policy(
            self.policy.can_view(&identity, &realm).await,
            "insufficient permissions to download file",
        )?;

        // Generate presigned URL
        let presigned_url = self
            .object_storage
            .presign_get_url(
                &stored_object.bucket,
                &stored_object.object_key,
                Duration::from_secs(300),
            )
            .await?;

        tracing::info!(
            object_id = %object_id,
            "Generated download URL"
        );

        Ok(presigned_url)
    }

    #[instrument(skip(self))]
    async fn delete_file(&self, identity: Identity, object_id: Uuid) -> Result<(), CoreError> {
        let stored_object = self.stored_object_repository.get_by_id(object_id).await?;

        // Get user to check realm access
        let user = self.user_repository.get_by_id(identity.id()).await?;

        let realm = user.realm.ok_or(CoreError::InvalidRealm)?;

        // Verify the file belongs to the user's realm or they have cross-realm access
        if stored_object.realm_id != realm.id && realm.name != "master" {
            return Err(CoreError::Forbidden(
                "File not in accessible realm".to_string(),
            ));
        }

        // Check permissions
        ensure_policy(
            self.policy.can_delete(&identity, &realm).await,
            "insufficient permissions to delete file",
        )?;

        // Delete from storage
        self.object_storage
            .delete_object(&stored_object.bucket, &stored_object.object_key)
            .await?;

        // Delete from database
        self.stored_object_repository.delete(object_id).await?;

        tracing::info!(
            object_id = %object_id,
            "File deleted"
        );

        Ok(())
    }

    #[instrument(skip(self, file_data), fields(realm_name = %realm_name, filename = %filename))]
    async fn upload_file_direct(
        &self,
        identity: Identity,
        realm_name: String,
        filename: String,
        mime_type: String,
        file_data: bytes::Bytes,
        metadata: serde_json::Value,
    ) -> Result<StoredObject, CoreError> {
        let size_bytes = file_data.len() as i64;

        // Validate file size
        if size_bytes > 52_428_800 {
            // 50 MB
            return Err(CoreError::FileTooLarge);
        }

        // Calculate SHA256 checksum
        let mut hasher = Sha256::new();
        hasher.update(&file_data);
        let checksum_sha256 = format!("{:x}", hasher.finalize());

        // Get realm
        let realm = self
            .realm_repository
            .get_by_name(realm_name.clone())
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        // Check permissions
        ensure_policy(
            self.policy.can_upload(&identity, &realm).await,
            "insufficient permissions to upload files",
        )?;

        // Generate object key
        let object_key = format!("{}/{}/{}", realm.id, generate_random_string(16), filename);

        let bucket = format!("ferriskey-{}", realm.name);

        // Create metadata record
        let create_input = CreateStoredObject {
            realm_id: realm.id,
            bucket: bucket.clone(),
            object_key: object_key.clone(),
            original_name: filename.clone(),
            mime_type: mime_type.clone(),
            size_bytes,
            checksum_sha256: checksum_sha256.clone(),
            metadata,
            uploaded_by: identity.id(),
        };

        let stored_object = self.stored_object_repository.create(create_input).await?;

        tracing::info!(
            object_id = %stored_object.id,
            bucket = %bucket,
            object_key = %object_key,
            size = size_bytes,
            "Uploading file directly to storage"
        );

        // Upload to MinIO
        self.object_storage
            .put_object(&bucket, &object_key, file_data, &mime_type)
            .await?;

        tracing::info!(
            object_id = %stored_object.id,
            "File uploaded successfully"
        );

        Ok(stored_object)
    }
}
