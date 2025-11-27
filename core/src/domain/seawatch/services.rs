#[allow(unused_imports)]
use crate::domain::storage::ports::{ObjectStoragePort, StoredObjectRepository};

use crate::domain::{
    authentication::{ports::AuthSessionRepository, value_objects::Identity},
    client::ports::{ClientRepository, RedirectUriRepository},
    common::{entities::app_errors::CoreError, policies::ensure_policy, services::Service},
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    food_analysis::ports::{FoodAnalysisRepository, LLMClient},
    health::ports::HealthCheckRepository,
    jwt::ports::{KeyStoreRepository, RefreshTokenRepository},
    prompt::ports::PromptRepository,
    realm::ports::RealmRepository,
    role::ports::RoleRepository,
    seawatch::{
        SecurityEvent, SecurityEventFilter, SecurityEventPolicy, SecurityEventRepository,
        ports::SecurityEventService, value_objects::FetchEventsInput,
    },
    trident::ports::RecoveryCodeRepository,
    user::ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
    webhook::ports::WebhookRepository,
};

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC, SE, PR, FA, LLM, OS, SO>
    SecurityEventService
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
{
    async fn fetch_events(
        &self,
        identity: Identity,
        input: FetchEventsInput,
    ) -> Result<Vec<SecurityEvent>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_events(identity, realm).await,
            "insufficient permissions",
        )?;

        let security_events = self
            .security_event_repository
            .get_events(
                realm_id,
                SecurityEventFilter {
                    ..Default::default()
                },
            )
            .await?;

        Ok(security_events)
    }
}
