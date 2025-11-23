# Food Analysis Specification

## Overview

This specification describes the food analysis feature for the FerrisKey project. The system allows users to analyze food items (from images or text descriptions) using LLM-powered analysis with configurable prompts. The analysis is designed to help users (especially those with dietary restrictions like IBD/IBS) determine if food items are safe to consume.

## Architecture Overview

The food analysis feature follows FerrisKey's hexagonal architecture pattern:

```
API Layer (HTTP Handlers)
    ↓
Application Layer (Service Composition)
    ↓
Domain Layer (Entities, Ports, Services, Policies)
    ↓
Infrastructure Layer (Repositories, LLM Client Adapter)
```

## Core Concepts

### Food Analysis Request

A food analysis request consists of:
- **Input Type**: Image (multipart/form-data) or Text (JSON)
- **Prompt Selection**: Reference to a Prompt entity (managed via existing prompt system)
- **Realm Context**: Analysis is scoped to a realm

### Food Analysis Response

The analysis returns structured data about identified dishes:
- **Dish Name**: Identified food item
- **Safety Level**: SAFE, CAUTION, or UNSAFE
- **Reason**: Brief explanation (15 words max)
- **Health Concerns**: IBD/IBS specific concerns
- **Risk Ingredients**: List of problematic ingredients
- **Recommendations**: Actionable advice (20 characters max)

## Domain Layer

### Entities

#### FoodAnalysisRequest

```rust
pub struct FoodAnalysisRequest {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub prompt_id: Uuid,  // Reference to Prompt entity
    pub input_type: InputType,  // Image or Text
    pub input_content: String,  // Text description or image metadata
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

pub enum InputType {
    Image,
    Text,
}
```

#### FoodAnalysisResult

```rust
pub struct FoodAnalysisResult {
    pub id: Uuid,
    pub request_id: Uuid,
    pub dishes: Vec<DishAnalysis>,
    pub raw_response: String,  // Original LLM JSON response
    pub created_at: DateTime<Utc>,
}

pub struct DishAnalysis {
    pub dish_name: String,
    pub safety_level: SafetyLevel,
    pub reason: String,
    pub ibd_concerns: Vec<String>,
    pub ibs_concerns: Vec<String>,
    pub recommendations: String,
    pub ingredients: Vec<RiskIngredient>,
}

pub enum SafetyLevel {
    Safe,
    Caution,
    Unsafe,
}

pub struct RiskIngredient {
    pub ingredient_name: String,
    pub risk_reason: String,
}
```

### Ports (Traits)

#### FoodAnalysisRepository

```rust
#[cfg_attr(test, mockall::automock)]
pub trait FoodAnalysisRepository: Send + Sync {
    fn create_request(
        &self,
        request: FoodAnalysisRequest,
    ) -> impl Future<Output = Result<FoodAnalysisRequest, CoreError>> + Send;

    fn create_result(
        &self,
        result: FoodAnalysisResult,
    ) -> impl Future<Output = Result<FoodAnalysisResult, CoreError>> + Send;

    fn get_request_by_id(
        &self,
        request_id: Uuid,
    ) -> impl Future<Output = Result<Option<FoodAnalysisRequest>, CoreError>> + Send;

    fn get_result_by_request_id(
        &self,
        request_id: Uuid,
    ) -> impl Future<Output = Result<Option<FoodAnalysisResult>, CoreError>> + Send;

    fn get_requests_by_realm(
        &self,
        realm_id: Uuid,
        filter: GetFoodAnalysisFilter,
    ) -> impl Future<Output = Result<Vec<FoodAnalysisRequest>, CoreError>> + Send;
}
```

#### LLMClient (Infrastructure Port)

```rust
#[cfg_attr(test, mockall::automock)]
pub trait LLMClient: Send + Sync {
    fn generate_with_image(
        &self,
        prompt: String,
        image_data: Vec<u8>,
        response_schema: serde_json::Value,
    ) -> impl Future<Output = Result<String, CoreError>> + Send;

    fn generate_with_text(
        &self,
        prompt: String,
        response_schema: serde_json::Value,
    ) -> impl Future<Output = Result<String, CoreError>> + Send;
}
```

#### FoodAnalysisService

```rust
pub trait FoodAnalysisService: Send + Sync {
    async fn analyze_food(
        &self,
        identity: Identity,
        input: AnalyzeFoodInput,
    ) -> Result<FoodAnalysisResult, CoreError>;

    async fn get_analysis_history(
        &self,
        identity: Identity,
        input: GetFoodAnalysisHistoryInput,
    ) -> Result<Vec<FoodAnalysisRequest>, CoreError>;

    async fn get_analysis_result(
        &self,
        identity: Identity,
        input: GetFoodAnalysisResultInput,
    ) -> Result<FoodAnalysisResult, CoreError>;
}
```

#### FoodAnalysisPolicy

```rust
pub trait FoodAnalysisPolicy: Send + Sync {
    fn can_analyze_food(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_view_analysis(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
```

### Services

#### FoodAnalysisService Implementation

```rust
impl<...> FoodAnalysisService for Service<...> {
    async fn analyze_food(
        &self,
        identity: Identity,
        input: AnalyzeFoodInput,
    ) -> Result<FoodAnalysisResult, CoreError> {
        // 1. Validate realm
        let realm = self.realm_repository
            .get_by_name(input.realm_name.clone())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        // 2. Check permissions
        ensure_policy(
            self.policy.can_analyze_food(identity, realm).await,
            "insufficient permissions",
        )?;

        // 3. Get prompt
        let prompt = self.prompt_repository
            .get_by_id(input.prompt_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        // Validate prompt belongs to realm
        if prompt.realm_id != realm.id {
            return Err(CoreError::InvalidRealm);
        }

        if !prompt.is_active || prompt.is_deleted {
            return Err(CoreError::InvalidInput("Prompt is not active".to_string()));
        }

        // 4. Build prompt template
        let input_content = match input.input_type {
            InputType::Image => "图片中的食物或菜单".to_string(),
            InputType::Text => input.text_input.unwrap_or_default(),
        };

        let full_prompt = prompt.template.replace("{input_content}", &input_content);

        // 5. Get response schema
        let response_schema = get_food_analysis_schema();

        // 6. Call LLM
        let raw_response = match input.input_type {
            InputType::Image => {
                self.llm_client
                    .generate_with_image(full_prompt, input.image_data, response_schema)
                    .await?
            }
            InputType::Text => {
                self.llm_client
                    .generate_with_text(full_prompt, response_schema)
                    .await?
            }
        };

        // 7. Parse and validate response
        let analysis_result = parse_food_analysis_response(&raw_response)?;

        // 8. Create request record
        let request = FoodAnalysisRequest::new(
            realm.id,
            prompt.id,
            input.input_type,
            input_content,
            identity.user_id,
        );
        let request = self.food_analysis_repository.create_request(request).await?;

        // 9. Create result record
        let result = FoodAnalysisResult {
            id: Uuid::new_v7(generate_timestamp().1),
            request_id: request.id,
            dishes: analysis_result.dishes,
            raw_response,
            created_at: Utc::now(),
        };
        let result = self.food_analysis_repository.create_result(result).await?;

        Ok(result)
    }
}
```

## Infrastructure Layer

### LLM Client Adapter

#### GeminiLLMClient

```rust
pub struct GeminiLLMClient {
    api_key: String,
    model_name: String,
    client: reqwest::Client,
}

impl LLMClient for GeminiLLMClient {
    async fn generate_with_image(
        &self,
        prompt: String,
        image_data: Vec<u8>,
        response_schema: serde_json::Value,
    ) -> Result<String, CoreError> {
        // Use Google Gemini API
        // Convert image_data to base64
        // Make API call with structured output
        // Return JSON string
    }

    async fn generate_with_text(
        &self,
        prompt: String,
        response_schema: serde_json::Value,
    ) -> Result<String, CoreError> {
        // Use Google Gemini API
        // Make API call with structured output
        // Return JSON string
    }
}
```

**Configuration**:
- API key from environment variable: `GEMINI_API_KEY`
- Model: `gemini-2.0-flash-exp` (configurable)
- Support for structured JSON output via `response_schema`

### Repository Implementation

#### PostgresFoodAnalysisRepository

```rust
pub struct PostgresFoodAnalysisRepository {
    pub db: DatabaseConnection,
}

impl FoodAnalysisRepository for PostgresFoodAnalysisRepository {
    async fn create_request(
        &self,
        request: FoodAnalysisRequest,
    ) -> Result<FoodAnalysisRequest, CoreError> {
        // Convert to entity, insert, convert back
    }

    // ... other methods
}
```

### Database Schema

#### food_analysis_requests table

```sql
CREATE TABLE food_analysis_requests (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL REFERENCES realms(id),
    prompt_id UUID NOT NULL REFERENCES prompts(id),
    input_type VARCHAR(10) NOT NULL,  -- 'image' or 'text'
    input_content TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    INDEX idx_realm_created (realm_id, created_at DESC)
);
```

#### food_analysis_results table

```sql
CREATE TABLE food_analysis_results (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL UNIQUE REFERENCES food_analysis_requests(id),
    dishes JSONB NOT NULL,  -- Array of dish analysis objects
    raw_response TEXT NOT NULL,  -- Original LLM JSON response
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    INDEX idx_request (request_id)
);
```

## API Layer

### Endpoints

#### Analyze Food

```http
POST /realms/{realm_name}/food-analysis
Content-Type: multipart/form-data or application/json
Authorization: Bearer <token>

# For image input:
{
    "prompt_id": "uuid",
    "image": <file>
}

# For text input:
{
    "prompt_id": "uuid",
    "text_input": "宫保鸡丁、麻婆豆腐、糖醋里脊"
}
```

**Response**:
```json
{
    "data": {
        "id": "uuid",
        "request_id": "uuid",
        "dishes": [
            {
                "dish_name": "宫保鸡丁",
                "safety_level": "CAUTION",
                "reason": "Contains spicy peppers and high fat",
                "ibd_concerns": ["Spicy ingredients may irritate", "High fat content"],
                "ibs_concerns": ["FODMAP triggers possible"],
                "recommendations": "少量尝试，避免辣椒",
                "ingredients": [
                    {
                        "ingredient_name": "辣椒",
                        "risk_reason": "可能刺激肠道"
                    }
                ]
            }
        ],
        "created_at": "2024-01-01T00:00:00Z"
    }
}
```

#### Get Analysis History

```http
GET /realms/{realm_name}/food-analysis?offset=0&limit=20
Authorization: Bearer <token>
```

**Response**:
```json
{
    "data": [
        {
            "id": "uuid",
            "prompt_id": "uuid",
            "prompt_name": "IBD/IBS 专用版本",
            "input_type": "text",
            "input_content": "宫保鸡丁",
            "created_at": "2024-01-01T00:00:00Z"
        }
    ]
}
```

#### Get Analysis Result

```http
GET /realms/{realm_name}/food-analysis/{request_id}/result
Authorization: Bearer <token>
```

**Response**: Same as Analyze Food response

### Request/Response DTOs

```rust
pub struct AnalyzeFoodRequest {
    pub prompt_id: Uuid,
    pub text_input: Option<String>,
    // Image handled separately in multipart
}

pub struct FoodAnalysisResponse {
    pub data: FoodAnalysisResult,
}

pub struct FoodAnalysisHistoryResponse {
    pub data: Vec<FoodAnalysisRequestItem>,
}

pub struct FoodAnalysisRequestItem {
    pub id: Uuid,
    pub prompt_id: Uuid,
    pub prompt_name: String,
    pub input_type: String,
    pub input_content: String,
    pub created_at: DateTime<Utc>,
}
```

## Frontend

### API Hooks

```typescript
// front/src/api/food-analysis.api.ts

export const useAnalyzeFood = () => {
  const queryClient = useQueryClient()
  return useMutation({
    ...window.tanstackApi.mutation(
      'post',
      '/realms/{realm_name}/food-analysis',
      async (res) => res.json()
    ).mutationOptions,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: ['food-analysis']
      })
    },
  })
}

export const useGetFoodAnalysisHistory = ({ realm, offset, limit }: GetHistoryParams) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/food-analysis', {
      path: { realm_name: realm! },
      query: { offset, limit },
    }).queryOptions,
    enabled: !!realm,
  })
}

export const useGetFoodAnalysisResult = ({ realm, requestId }: GetResultParams) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/food-analysis/{request_id}/result', {
      path: {
        realm_name: realm!,
        request_id: requestId!,
      },
    }).queryOptions,
    enabled: !!requestId && !!realm,
  })
}
```

### UI Components

- **Food Analysis Page**: Main page for analyzing food
  - Image upload component
  - Text input component
  - Prompt selector (dropdown of available prompts)
  - Results display component

- **Analysis History Page**: View past analyses
  - List of previous analyses
  - Filter by prompt, date range
  - Click to view detailed results

## JSON Schema for LLM Response

The LLM client uses structured output with the following JSON schema:

```json
{
  "type": "object",
  "properties": {
    "dishes": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "dish_name": { "type": "string" },
          "safety_level": { "type": "string", "enum": ["SAFE", "CAUTION", "UNSAFE"] },
          "reason": { "type": "string" },
          "ibd_concerns": { "type": "array", "items": { "type": "string" } },
          "ibs_concerns": { "type": "array", "items": { "type": "string" } },
          "recommendations": { "type": "string" },
          "ingredients": {
            "type": "array",
            "items": {
              "type": "object",
              "properties": {
                "ingredient_name": { "type": "string" },
                "risk_reason": { "type": "string" }
              }
            }
          }
        },
        "required": ["dish_name", "safety_level", "reason", "ibd_concerns", "ibs_concerns", "recommendations", "ingredients"]
      }
    }
  },
  "required": ["dishes"]
}
```

## Integration with Existing Prompt System

The food analysis feature leverages the existing prompt management system:

1. **Prompt Selection**: Users select a prompt from the realm's available prompts
2. **Template Processing**: The selected prompt's template is used, with `{input_content}` replaced by user input
3. **Versioning**: Prompt versioning is handled by the existing prompt system
4. **Realm Scoping**: Prompts are realm-scoped, ensuring proper isolation

## Security Considerations

1. **Authentication**: All endpoints require valid JWT token
2. **Authorization**: Users must have appropriate permissions in the realm
3. **Realm Isolation**: Analysis requests are scoped to realms
4. **Input Validation**:
   - Image size limits (e.g., max 10MB)
   - Text input length limits (e.g., max 5000 characters)
   - Prompt ownership validation
5. **API Key Security**: Gemini API key stored in environment variables, not in code

## Error Handling

- **Invalid Prompt**: Return 404 if prompt not found or not active
- **LLM API Failure**: Return 500 with error message
- **Invalid Input**: Return 400 with validation errors
- **Permission Denied**: Return 403
- **Image Processing Error**: Return 400 with specific error

## Performance Considerations

1. **LLM API Calls**: Can be slow (2-10 seconds), use async/await
2. **Image Upload**: Support streaming for large images
3. **Caching**: Consider caching results for identical inputs (optional)
4. **Rate Limiting**: Implement rate limiting per user/realm

## Configuration

### Environment Variables

- `GEMINI_API_KEY`: Google Gemini API key (required)
- `GEMINI_MODEL`: Model name (default: `gemini-2.0-flash-exp`)
- `MAX_IMAGE_SIZE_MB`: Maximum image size (default: 10)
- `MAX_TEXT_INPUT_LENGTH`: Maximum text input length (default: 5000)

## Migration Path

1. **Phase 1**: Core domain and infrastructure
   - Create entities, ports, services
   - Implement LLM client adapter
   - Database migrations

2. **Phase 2**: API layer
   - HTTP handlers
   - Request/response DTOs
   - Router setup

3. **Phase 3**: Frontend
   - API hooks
   - UI components
   - Integration with prompt management UI

4. **Phase 4**: Testing and refinement
   - Unit tests
   - Integration tests
   - E2E tests

## Example Prompt Template

The prompt template stored in the Prompt entity should follow this format:

```
你是一位专业的营养师和消化系统疾病专家，专门为 IBD（炎症性肠病）和 IBS（肠易激综合征）患者提供饮食建议。

## 任务说明

请按照以下步骤分析输入内容：

### 第一步：识别所有菜品名称
从文本输入或菜单照片中识别出所有的菜品名称。

### 第二步：评估每个菜品对 IBD/IBS 患者的安全性
为每个识别出的菜品进行评估，给出安全等级和详细原因。

**重要要求**：
- `reason`（评估原因）必须限制在 **15 个单词以内**，简洁明了地说明主要风险
- `recommendations`（建议）必须限制在 **20 个字以内**，提供简短实用的建议

## 输入内容
{input_content}

## 输出要求

请严格按照以下 JSON Schema 格式返回结果，确保返回的是有效的 JSON 格式：

{
  "dishes": [
    {
      "dish_name": "菜品名称",
      "safety_level": "SAFE|CAUTION|UNSAFE",
      "reason": "详细的安全评估原因（限制在 15 个单词以内）",
      "ibd_concerns": ["可能的 IBD 相关担忧点1", "担忧点2"],
      "ibs_concerns": ["可能的 IBS 相关担忧点1", "担忧点2"],
      "recommendations": "针对该菜品的具体建议（限制在 20 个字以内）",
      "ingredients": [
        {
          "ingredient_name": "风险成分名称",
          "risk_reason": "该成分的触发风险原因（一行说明）"
        }
      ]
    }
  ]
}

## 安全等级说明

- **SAFE**: 对 IBD/IBS 患者相对安全，可以适量食用
- **CAUTION**: 需要谨慎，可能引起症状，建议少量尝试或避免某些成分
- **UNSAFE**: 不建议食用，很可能引起 IBD/IBS 症状加重

请开始分析。
```

## Future Enhancements

1. **Batch Analysis**: Analyze multiple images/texts in one request
2. **Analysis Comparison**: Compare results from different prompts
3. **User Preferences**: Store user dietary restrictions/preferences
4. **Personalized Recommendations**: Use user history for better recommendations
5. **Export Functionality**: Export analysis results to PDF/CSV
6. **Webhook Integration**: Trigger webhooks on analysis completion
7. **Analytics**: Track analysis usage and patterns
