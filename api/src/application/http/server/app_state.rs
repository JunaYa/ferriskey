use std::sync::Arc;

use ferriskey_core::{
    application::FerrisKeyService,
    infrastructure::{
        device_profile::PostgresDeviceProfileRepository,
        food_analysis::repositories::{
            PostgresFoodAnalysisItemRepository, PostgresFoodAnalysisTriggerRepository,
        },
        food_reaction::PostgresFoodReactionRepository,
        food_stats::PostgresFoodStatsRepository,
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
    pub trigger_repository: Arc<PostgresFoodAnalysisTriggerRepository>,
    pub reaction_repository: Arc<PostgresFoodReactionRepository>,
    pub stats_repository: Arc<PostgresFoodStatsRepository>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        args: Arc<Args>,
        service: FerrisKeyService,
        device_profile_repository: PostgresDeviceProfileRepository,
        user_repository: PostgresUserRepository,
        item_repository: PostgresFoodAnalysisItemRepository,
        trigger_repository: PostgresFoodAnalysisTriggerRepository,
        reaction_repository: PostgresFoodReactionRepository,
        stats_repository: PostgresFoodStatsRepository,
    ) -> Self {
        Self {
            args,
            service,
            device_profile_repository: Arc::new(device_profile_repository),
            user_repository: Arc::new(user_repository),
            item_repository: Arc::new(item_repository),
            trigger_repository: Arc::new(trigger_repository),
            reaction_repository: Arc::new(reaction_repository),
            stats_repository: Arc::new(stats_repository),
        }
    }
}
