use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    device_profile::{entities::DeviceProfile, ports::DeviceProfileRepository},
    realm::entities::Realm,
    user::{ports::UserRepository, value_objects::CreateUserRequest},
};

pub trait DeviceProfileService: Send + Sync {
    fn get_or_create_device_profile(
        &self,
        realm: Realm,
        device_id: &str,
        identity: &Identity,
    ) -> impl std::future::Future<Output = Result<DeviceProfile, CoreError>> + Send;
}

/// Helper function to get or create device profile
pub async fn get_or_create_device_profile<DP, U>(
    device_profile_repository: &DP,
    user_repository: &U,
    realm: Realm,
    device_id: &str,
    identity: &Identity,
) -> Result<DeviceProfile, CoreError>
where
    DP: DeviceProfileRepository,
    U: UserRepository,
{
    // 1. Try to find existing device profile
    if let Some(profile) = device_profile_repository
        .get_by_realm_and_device(realm.id, device_id)
        .await?
    {
        return Ok(profile);
    }

    // 2. Device doesn't exist, create anonymous user
    let username = generate_anonymous_username(device_id);
    let email = generate_anonymous_email(device_id);
    let firstname = generate_anonymous_name(device_id, "firstname");
    let lastname = generate_anonymous_name(device_id, "lastname");

    let user = user_repository
        .create_user(CreateUserRequest {
            realm_id: realm.id,
            username: username.clone(),
            email,
            firstname,
            lastname,
            email_verified: false,
            enabled: true,
            client_id: None,
        })
        .await?;

    // 3. Create device profile
    let device_profile = DeviceProfile::new(
        realm.id,
        device_id.to_string(),
        user.id,
        Some(identity.id()),
    );

    let profile = device_profile_repository.create(device_profile).await?;

    Ok(profile)
}

pub fn hash_device_id(device_id: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(device_id.as_bytes());
    let hash = hasher.finalize();
    hex::encode(&hash[..16]) // Use first 16 bytes (32 hex chars)
}

pub fn generate_anonymous_username(device_id: &str) -> String {
    hash_device_id(device_id)
}

pub fn generate_anonymous_email(device_id: &str) -> String {
    let hash = hash_device_id(device_id);
    format!("device_{}@anonymous.local", hash)
}

pub fn generate_anonymous_name(device_id: &str, suffix: &str) -> String {
    let hash = hash_device_id(device_id);
    format!("device_{}_{}", hash, suffix)
}
