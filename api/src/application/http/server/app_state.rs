use std::sync::Arc;

use ferriskey_core::{
    application::FerrisKeyService,
    infrastructure::{
        device_profile::PostgresDeviceProfileRepository,
        food_analysis::repositories::PostgresFoodAnalysisItemRepository,
        user::repository::PostgresUserRepository,
    },
};

use crate::args::Args;

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,
    pub service: FerrisKeyService,
    pub device_profile_repository: Arc<PostgresDeviceProfileRepository>,
    pub user_repository: Arc<PostgresUserRepository>,
    pub item_repository: Arc<PostgresFoodAnalysisItemRepository>,
}

impl AppState {
    pub fn new(
        args: Arc<Args>,
        service: FerrisKeyService,
        device_profile_repository: PostgresDeviceProfileRepository,
        user_repository: PostgresUserRepository,
        item_repository: PostgresFoodAnalysisItemRepository,
    ) -> Self {
        Self {
            args,
            service,
            device_profile_repository: Arc::new(device_profile_repository),
            user_repository: Arc::new(user_repository),
            item_repository: Arc::new(item_repository),
        }
    }
}
