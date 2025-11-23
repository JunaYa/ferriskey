use crate::domain::{
    authentication::{ports::AuthSessionRepository, value_objects::Identity},
    client::ports::{ClientRepository, RedirectUriRepository},
    common::{entities::app_errors::CoreError, policies::ensure_policy, services::Service},
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    food_analysis::ports::{FoodAnalysisRepository, LLMClient},
    health::ports::HealthCheckRepository,
    jwt::ports::{KeyStoreRepository, RefreshTokenRepository},
    prompt::{
        entities::prompt::Prompt,
        ports::{PromptPolicy, PromptRepository, PromptService},
        value_objects::{
            CreatePromptInput, DeletePromptInput, GetPromptInput, GetPromptsFilter,
            UpdatePromptInput,
        },
    },
    realm::ports::RealmRepository,
    role::ports::RoleRepository,
    seawatch::SecurityEventRepository,
    trident::ports::RecoveryCodeRepository,
    user::ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
    webhook::ports::WebhookRepository,
};

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC, SE, PR, FA, LLM> PromptService
    for Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC, SE, PR, FA, LLM>
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
{
    async fn get_prompts(
        &self,
        identity: Identity,
        input: GetPromptsFilter,
    ) -> Result<Vec<Prompt>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_prompt(identity, realm).await,
            "insufficient permissions",
        )?;

        let prompts = self
            .prompt_repository
            .fetch_prompts_by_realm(realm_id, input.clone())
            .await?;

        Ok(prompts)
    }

    async fn get_prompt(
        &self,
        identity: Identity,
        input: GetPromptInput,
    ) -> Result<Option<Prompt>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_prompt(identity, realm).await,
            "insufficient permissions",
        )?;

        let prompt = self
            .prompt_repository
            .get_prompt_by_id(input.prompt_id, realm_id)
            .await?;

        Ok(prompt)
    }

    async fn create_prompt(
        &self,
        identity: Identity,
        input: CreatePromptInput,
    ) -> Result<Prompt, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_create_prompt(identity.clone(), realm).await,
            "insufficient permissions",
        )?;

        let user_id = identity.id();

        let prompt = Prompt::new(
            realm_id,
            input.name,
            input.description,
            input.template,
            input.version,
            user_id,
        );

        let created_prompt = self.prompt_repository.create_prompt(prompt).await?;

        Ok(created_prompt)
    }

    async fn update_prompt(
        &self,
        identity: Identity,
        input: UpdatePromptInput,
    ) -> Result<Prompt, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_update_prompt(identity.clone(), realm).await,
            "insufficient permissions",
        )?;

        let user_id = identity.id();

        let mut prompt = self
            .prompt_repository
            .get_prompt_by_id(input.prompt_id, realm_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        prompt.update(
            input.name,
            input.description,
            input.template,
            input.version,
            input.is_active,
            user_id,
        );

        let updated_prompt = self.prompt_repository.update_prompt(prompt).await?;

        Ok(updated_prompt)
    }

    async fn delete_prompt(
        &self,
        identity: Identity,
        input: DeletePromptInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_delete_prompt(identity.clone(), realm).await,
            "insufficient permissions",
        )?;

        let user_id = identity.id();

        self.prompt_repository
            .delete_prompt(input.prompt_id, realm_id, user_id)
            .await?;

        Ok(())
    }
}
