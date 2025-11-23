use base64::{Engine as _, engine::general_purpose};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::domain::{common::entities::app_errors::CoreError, food_analysis::ports::LLMClient};

#[derive(Debug, Clone)]
pub struct GeminiLLMClient {
    api_key: String,
    model_name: String,
    client: Client,
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Part {
    Text { text: String },
    InlineData { inline_data: InlineData },
}

#[derive(Debug, Serialize)]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    response_mime_type: String,
    response_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ContentResponse,
}

#[derive(Debug, Deserialize)]
struct ContentResponse {
    parts: Vec<PartResponse>,
}

#[derive(Debug, Deserialize)]
struct PartResponse {
    text: String,
}

impl GeminiLLMClient {
    pub fn new(api_key: String, model_name: String) -> Self {
        Self {
            api_key,
            model_name,
            client: Client::new(),
        }
    }

    async fn call_gemini_api(&self, request: GeminiRequest) -> Result<String, CoreError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model_name, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Gemini API request failed: {}", e);
                CoreError::ExternalServiceError(format!("LLM API error: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("Gemini API error: {} - {}", status, error_text);
            return Err(CoreError::ExternalServiceError(format!(
                "LLM API returned error: {} - {}",
                status, error_text
            )));
        }

        let gemini_response: GeminiResponse = response.json().await.map_err(|e| {
            tracing::error!("Failed to parse Gemini response: {}", e);
            CoreError::ExternalServiceError(format!("Failed to parse LLM response: {}", e))
        })?;

        gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| CoreError::ExternalServiceError("No response from LLM".to_string()))
    }
}

impl LLMClient for GeminiLLMClient {
    async fn generate_with_image(
        &self,
        prompt: String,
        image_data: Vec<u8>,
        response_schema: serde_json::Value,
    ) -> Result<String, CoreError> {
        let base64_image = general_purpose::STANDARD.encode(&image_data);

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![
                    Part::Text { text: prompt },
                    Part::InlineData {
                        inline_data: InlineData {
                            mime_type: "image/jpeg".to_string(),
                            data: base64_image,
                        },
                    },
                ],
            }],
            generation_config: Some(GenerationConfig {
                response_mime_type: "application/json".to_string(),
                response_schema,
            }),
        };

        self.call_gemini_api(request).await
    }

    async fn generate_with_text(
        &self,
        prompt: String,
        response_schema: serde_json::Value,
    ) -> Result<String, CoreError> {
        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part::Text { text: prompt }],
            }],
            generation_config: Some(GenerationConfig {
                response_mime_type: "application/json".to_string(),
                response_schema,
            }),
        };

        self.call_gemini_api(request).await
    }
}
