use std::sync::Arc;

use ferriskey_core::application::FerrisKeyService;
use ferriskey_core::infrastructure::device_profile::PostgresDeviceProfileRepository;

use crate::args::Args;

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,
    pub service: FerrisKeyService,
    pub device_profile_repository: Arc<PostgresDeviceProfileRepository>,
}

impl AppState {
    pub fn new(
        args: Arc<Args>,
        service: FerrisKeyService,
        device_profile_repository: PostgresDeviceProfileRepository,
    ) -> Self {
        Self {
            args,
            service,
            device_profile_repository: Arc::new(device_profile_repository),
        }
    }
}
