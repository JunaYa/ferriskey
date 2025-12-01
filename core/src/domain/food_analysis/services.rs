#[allow(unused_imports)]
use crate::domain::storage::ports::{ObjectStoragePort, StoredObjectRepository};

use crate::domain::{
    authentication::{ports::AuthSessionRepository, value_objects::Identity},
    client::ports::{ClientRepository, RedirectUriRepository},
    common::{entities::app_errors::CoreError, policies::ensure_policy, services::Service},
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    food_analysis::{
        entities::{
            DishAnalysis, FoodAnalysisItem, FoodAnalysisRequest, FoodAnalysisResult, InputType,
        },
        helpers::create_items_and_triggers_from_dishes,
        ports::{FoodAnalysisPolicy, FoodAnalysisRepository, FoodAnalysisService, LLMClient},
        schema::get_food_analysis_schema,
        value_objects::{
            AnalyzeFoodInput, GetFoodAnalysisHistoryInput, GetFoodAnalysisItemInput,
            GetFoodAnalysisItemsByRequestInput, GetFoodAnalysisItemsInput,
            GetFoodAnalysisRequestInput, GetFoodAnalysisResultInput,
        },
    },
    health::ports::HealthCheckRepository,
    jwt::ports::{KeyStoreRepository, RefreshTokenRepository},
    prompt::ports::PromptRepository,
    realm::ports::RealmRepository,
    role::ports::RoleRepository,
    seawatch::SecurityEventRepository,
    trident::ports::RecoveryCodeRepository,
    user::ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
    webhook::ports::WebhookRepository,
};

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC, SE, PR, FA, LLM, OS, SO>
    FoodAnalysisService
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
{
    async fn analyze_food(
        &self,
        identity: Identity,
        input: AnalyzeFoodInput,
    ) -> Result<FoodAnalysisResult, CoreError> {
        // 1. Validate realm
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        // 2. Check permissions
        ensure_policy(
            self.policy
                .can_analyze_food(identity.clone(), realm.clone())
                .await,
            "insufficient permissions to analyze food",
        )?;

        // 3. Get and validate prompt
        let prompt = if let Some(prompt_id) = input.prompt_id {
            // If prompt_id is provided, get the specific prompt
            let prompt = self
                .prompt_repository
                .get_prompt_by_id(prompt_id, realm.id)
                .await?
                .ok_or(CoreError::NotFound)?;

            if prompt.realm_id != realm.id {
                return Err(CoreError::InvalidRealm);
            }

            if !prompt.is_active || prompt.is_deleted {
                return Err(CoreError::Invalid);
            }

            prompt
        } else {
            // If prompt_id is None, get the first active prompt
            use crate::domain::prompt::value_objects::GetPromptsFilter;
            let filter = GetPromptsFilter {
                realm_name: input.realm_name.clone(),
                name: None,
                description: None,
                include_deleted: false,
                limit: None,
                offset: None,
            };

            let prompts = self
                .prompt_repository
                .fetch_prompts_by_realm(realm.id, filter)
                .await?;

            // Filter active prompts and sort by updated_at descending
            let mut active_prompts: Vec<_> = prompts
                .into_iter()
                .filter(|p| p.is_active && !p.is_deleted && p.realm_id == realm.id)
                .collect();

            // Sort by updated_at descending
            active_prompts.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

            active_prompts
                .into_iter()
                .next()
                .ok_or(CoreError::NotFound)?
        };

        // 4. Build prompt template
        let input_content = match input.input_type {
            InputType::Image => "图片中的食物或菜单".to_string(),
            InputType::Text => input.text_input.clone().unwrap_or_default(),
        };

        let full_prompt = prompt.template.replace("{input_content}", &input_content);

        // 5. Get response schema
        let response_schema = get_food_analysis_schema();

        // 6. Call LLM
        let raw_response = match input.input_type {
            InputType::Image => {
                let image_data = input.image_data.ok_or(CoreError::Invalid)?;
                self.llm_client
                    .generate_with_image(full_prompt, image_data, response_schema)
                    .await?
            }
            InputType::Text => {
                self.llm_client
                    .generate_with_text(full_prompt, response_schema)
                    .await?
            }
        };

        // 7. Parse and validate response
        let parsed: serde_json::Value = serde_json::from_str(&raw_response).map_err(|e| {
            tracing::error!("Failed to parse LLM response: {}", e);
            CoreError::ExternalServiceError(format!("Failed to parse LLM response: {}", e))
        })?;

        let dishes: Vec<DishAnalysis> = serde_json::from_value(
            parsed
                .get("dishes")
                .ok_or_else(|| {
                    CoreError::ExternalServiceError("No dishes field in response".to_string())
                })?
                .clone(),
        )
        .map_err(|e| {
            tracing::error!("Invalid dishes format: {}", e);
            CoreError::ExternalServiceError(format!("Invalid dishes format: {}", e))
        })?;

        // 8. Create request record
        let request = FoodAnalysisRequest::new(
            realm.id,
            prompt.id,
            input.input_type,
            input_content,
            identity.id(),
            input.device_id,
            input.user_id,
        );
        let request = self
            .food_analysis_repository
            .create_request(request)
            .await?;

        // 9. Create result record
        let result = FoodAnalysisResult::new(
            request.id,
            dishes.clone(),
            raw_response,
            identity.id(),
            identity.id(),
        );
        let result = self.food_analysis_repository.create_result(result).await?;

        // 10. Create items and triggers from dishes
        let (items, triggers) = create_items_and_triggers_from_dishes(
            realm.id,
            request.id,
            result.id,
            &dishes,
            identity.id(),
        );

        // Create items in batch
        self.food_analysis_repository
            .create_items_batch(items)
            .await?;

        // Create triggers in batch
        self.food_analysis_repository
            .create_triggers_batch(triggers)
            .await?;

        Ok(result)
    }

    async fn get_analysis_history(
        &self,
        identity: Identity,
        input: GetFoodAnalysisHistoryInput,
    ) -> Result<Vec<FoodAnalysisRequest>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_analysis(identity, realm.clone()).await,
            "insufficient permissions to view analysis history",
        )?;

        let filter = input.filter;

        let requests = self
            .food_analysis_repository
            .get_requests_by_realm(realm.id, filter)
            .await?;

        Ok(requests)
    }

    async fn get_analysis_request(
        &self,
        identity: Identity,
        input: GetFoodAnalysisRequestInput,
    ) -> Result<FoodAnalysisRequest, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_analysis(identity, realm.clone()).await,
            "insufficient permissions to view analysis",
        )?;

        // Get request and verify it belongs to realm
        let request = self
            .food_analysis_repository
            .get_request_by_id(input.request_id, realm.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        Ok(request)
    }

    async fn get_analysis_result(
        &self,
        identity: Identity,
        input: GetFoodAnalysisResultInput,
    ) -> Result<FoodAnalysisResult, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_analysis(identity, realm.clone()).await,
            "insufficient permissions to view analysis",
        )?;

        // Verify request belongs to realm
        let _request = self
            .food_analysis_repository
            .get_request_by_id(input.request_id, realm.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let result = self
            .food_analysis_repository
            .get_result_by_request_id(input.request_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        Ok(result)
    }

    async fn get_analysis_items_by_request(
        &self,
        _identity: Identity,
        _input: GetFoodAnalysisItemsByRequestInput,
    ) -> Result<Vec<FoodAnalysisItem>, CoreError> {
        // This will be implemented in handlers using item_repository from AppState
        Err(CoreError::InternalServerError)
    }

    async fn get_analysis_item(
        &self,
        _identity: Identity,
        _input: GetFoodAnalysisItemInput,
    ) -> Result<FoodAnalysisItem, CoreError> {
        // This will be implemented in handlers using item_repository from AppState
        Err(CoreError::InternalServerError)
    }

    async fn get_analysis_items(
        &self,
        _identity: Identity,
        _input: GetFoodAnalysisItemsInput,
    ) -> Result<Vec<FoodAnalysisItem>, CoreError> {
        // This will be implemented in handlers using item_repository from AppState
        Err(CoreError::InternalServerError)
    }
}
