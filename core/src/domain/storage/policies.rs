use std::future::Future;

use crate::domain::{
    authentication::value_objects::Identity, common::entities::app_errors::CoreError,
    realm::entities::Realm,
};

/// Policy trait for file storage operations
#[cfg_attr(test, mockall::automock)]
pub trait FilePolicy: Send + Sync {
    /// Check if the identity can upload files to the realm
    fn can_upload(
        &self,
        identity: &Identity,
        realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    /// Check if the identity can view/download files in the realm
    fn can_view(
        &self,
        identity: &Identity,
        realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    /// Check if the identity can delete files in the realm
    fn can_delete(
        &self,
        identity: &Identity,
        realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
