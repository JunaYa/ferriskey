use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, Policy},
    },
    food_analysis::ports::FoodAnalysisPolicy,
    realm::entities::Realm,
    role::entities::permission::Permissions,
    user::ports::{UserRepository, UserRoleRepository},
};

impl<U, C, UR> FoodAnalysisPolicy for FerriskeyPolicy<U, C, UR>
where
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    async fn can_analyze_food(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(&identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[
                Permissions::ManageRealm,
                Permissions::ManageWebhooks,
                Permissions::ViewWebhooks,
            ],
        );

        Ok(has_permission)
    }

    async fn can_view_analysis(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> Result<bool, CoreError> {
        // Same permissions as analyze for now
        self.can_analyze_food(identity, target_realm).await
    }
}
