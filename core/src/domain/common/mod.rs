use chrono::{DateTime, Utc};
use rand::{Rng, distributions::Alphanumeric};
use uuid::{NoContext, Timestamp, Uuid};

pub mod entities;
pub mod policies;
pub mod ports;
pub mod services;

pub struct AppConfig {
    pub database_url: String,
}

#[derive(Clone, Debug)]
pub struct FerriskeyConfig {
    pub database: DatabaseConfig,
    pub llm: LLMConfig,
    pub object_storage: ObjectStorageConfig,
}

#[derive(Clone, Debug)]
pub struct ObjectStorageConfig {
    pub endpoint: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket_prefix: String,
    pub use_ssl: bool,
}

#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct LLMConfig {
    pub gemini_api_key: String,
    pub gemini_model: String,
}

pub fn generate_timestamp() -> (DateTime<Utc>, Timestamp) {
    let now = Utc::now();
    let seconds = now.timestamp().try_into().unwrap_or(0);
    let timestamp = Timestamp::from_unix(NoContext, seconds, 0);

    (now, timestamp)
}

pub fn generate_uuid_v7() -> Uuid {
    let (_, timestamp) = generate_timestamp();
    Uuid::new_v7(timestamp)
}

pub fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
