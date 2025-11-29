use std::future::Future;
use uuid::Uuid;

use crate::domain::{
    common::entities::app_errors::CoreError, device_profile::entities::DeviceProfile,
};

#[cfg_attr(test, mockall::automock)]
pub trait DeviceProfileRepository: Send + Sync {
    fn get_by_realm_and_device(
        &self,
        realm_id: Uuid,
        device_id: &str,
    ) -> impl Future<Output = Result<Option<DeviceProfile>, CoreError>> + Send;

    fn create(
        &self,
        profile: DeviceProfile,
    ) -> impl Future<Output = Result<DeviceProfile, CoreError>> + Send;

    fn get_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<DeviceProfile>, CoreError>> + Send;
}
