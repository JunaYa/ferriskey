use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct GetPromptsFilter {
    pub realm_name: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub include_deleted: bool,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub struct GetPromptInput {
    pub realm_name: String,
    pub prompt_id: Uuid,
}

pub struct CreatePromptInput {
    pub realm_name: String,
    pub name: String,
    pub description: String,
    pub template: String,
    pub version: String,
}

pub struct UpdatePromptInput {
    pub realm_name: String,
    pub prompt_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub template: Option<String>,
    pub version: Option<String>,
    pub is_active: Option<bool>,
}

pub struct DeletePromptInput {
    pub realm_name: String,
    pub prompt_id: Uuid,
}
