## HTTP 接口设计

接口遵循 FerrisKey 现有风格和 RESTful 设计原则：
- 路径前缀：`/realms/{realm_name}`
- 使用 `Authorization: Bearer <token>` 获取 `Identity`，再结合 `X-Device-Id` 识别具体终端用户
- 所有列表类接口统一使用 **offset/limit 分页**，并在响应中返回 `items, offset, limit, count`
- 所有列表接口支持 **filter 查询**和 **排序**
- 遵循 RESTful 资源命名规范：使用名词复数，避免动词
- HTTP 状态码：200（成功）、201（创建成功）、400（请求错误）、401（未授权）、403（禁止）、404（未找到）、500（服务器错误）

---

## 0. 认证与设备标识

### 0.1 认证流程

所有接口都需要：
1. **Authorization Header**：`Authorization: Bearer <token>` - 用于获取 `Identity` 和 realm 权限验证
2. **设备标识 Header**：`X-Device-Id: <device_id>` - 用于识别唯一用户

### 0.2 设备标识处理流程

**服务端处理逻辑**（所有接口统一执行）：

1. **接收请求**：从 `X-Device-Id` Header 获取设备标识
2. **验证 Realm**：通过 `Authorization: Bearer <token>` 获取 `Identity`，验证用户对 `realm_name` 的访问权限
3. **查找设备配置**：
   ```sql
   SELECT * FROM device_profiles
   WHERE realm_id = ? AND device_id = ?
   ```
4. **设备不存在时自动创建**：
   - 如果设备未在 `device_profiles` 中注册，服务端**自动执行**（无需客户端调用注册接口）：
     - 创建匿名用户（`users` 表），用户名格式：`anonymous_device_{device_id_hash}`
     - 创建设备配置（`device_profiles` 表），绑定 `device_id` 与 `user_id`
     - 设置 `created_by` 为当前 `Identity.user_id()`（如果有）
   - 后续所有操作使用该 `user_id`
5. **设备已存在**：直接使用现有的 `user_id`
6. **数据隔离**：
   - 所有查询操作自动按 `user_id` 过滤，确保数据隔离
   - 创建操作（如 `food_analysis_requests`、`food_reactions`）自动使用该 `user_id`

**实现要点**：
- **透明性**：客户端无需关心用户创建流程，只需传递 `X-Device-Id`
- **一致性**：所有接口使用相同的设备标识处理逻辑
- **安全性**：通过 `realm_id` 和 `user_id` 双重隔离，确保数据安全

**Rust 实现示例**：
```rust
// 伪代码示例
async fn get_or_create_device_profile(
    realm_id: Uuid,
    device_id: &str,
    identity: &Identity,
) -> Result<DeviceProfile, CoreError> {
    // 1. 尝试查找现有设备配置
    if let Some(profile) = device_profiles_repository
        .get_by_realm_and_device(realm_id, device_id)
        .await?
    {
        return Ok(profile);
    }

    // 2. 设备不存在，创建匿名用户
    let anonymous_user = User::new_anonymous(device_id)?;
    let user = user_repository.create(anonymous_user).await?;

    // 3. 创建设备配置
    let device_profile = DeviceProfile::new(
        realm_id,
        device_id.to_string(),
        user.id,
        identity.id(), // created_by
    )?;
    let profile = device_profiles_repository.create(device_profile).await?;

    Ok(profile)
}

// 在中间件或服务层统一调用
async fn handle_request(
    realm_name: String,
    device_id: String,
    identity: Identity,
) -> Result<Response> {
    // 1. 获取 realm
    let realm = get_realm_by_name(&realm_name)?;

    // 2. 获取或创建设备配置（自动处理）
    let device_profile = get_or_create_device_profile(
        realm.id,
        &device_id,
        &identity,
    ).await?;

    // 3. 使用 device_profile.user_id 进行后续操作
    // 所有查询自动按 user_id 过滤
    // 所有创建操作自动使用 user_id
    // ...
}
```

### 0.3 Filter 查询规范

所有列表接口支持以下 filter 参数格式：

**格式**：`filter[field_name]=value` 或 `filter[field_name][operator]=value`

**支持的运算符**：
- `eq` - 等于（默认，可省略）
- `ne` - 不等于
- `gt` - 大于
- `gte` - 大于等于
- `lt` - 小于
- `lte` - 小于等于
- `in` - 在列表中（值用逗号分隔）
- `like` - 模糊匹配（字符串）
- `ilike` - 不区分大小写模糊匹配（PostgreSQL）

**示例**：
```
GET /realms/{realm_name}/food-analysis/items?filter[risk_band]=SAFE
GET /realms/{realm_name}/food-analysis/items?filter[risk_score][gte]=50&filter[risk_score][lte]=80
GET /realms/{realm_name}/food-analysis/items?filter[dish_name][ilike]=chicken
GET /realms/{realm_name}/food-reactions?filter[feeling][in]=MILD_ISSUES,BAD
GET /realms/{realm_name}/food-analysis/items?filter[created_at][gte]=2025-11-01T00:00:00Z
```

**多条件组合**：多个 filter 参数之间为 AND 关系

### 0.4 排序规范

所有列表接口支持排序参数：

**格式**：`sort=field_name` 或 `sort=-field_name`

- `sort=field_name` - 升序（ASC）
- `sort=-field_name` - 降序（DESC）
- 多个排序字段：`sort=field1,-field2`（逗号分隔）

**示例**：
```
GET /realms/{realm_name}/food-analysis/items?sort=-risk_score
GET /realms/{realm_name}/food-analysis/items?sort=created_at,-risk_score
GET /realms/{realm_name}/food-reactions?sort=-eaten_at
```

**默认排序**：
- 如果没有指定 `sort` 参数，使用默认排序（通常是 `created_at DESC`）
- 每个接口的默认排序在接口文档中说明

---

## 1. 设备管理接口（可选）

> **注意**：设备配置会在首次使用 `X-Device-Id` 时自动创建，通常不需要手动调用设备管理接口。

### 1.1 获取设备配置

```http
GET /realms/{realm_name}/devices/{device_id}
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

**响应** (200 OK)

```json
{
  "id": "uuid-of-device-profile",
  "realm_id": "uuid-of-realm",
  "device_id": "ios-uuid-or-android-id",
  "user_id": "uuid-of-user",
  "created_at": "2025-11-29T00:00:00Z",
  "updated_at": "2025-11-29T00:00:00Z"
}
```

**说明**：
- 如果设备不存在，服务端会自动创建匿名用户和设备配置，然后返回（200 OK）
- 如果设备已存在，直接返回现有配置（200 OK）

---

## 2. 食物分析接口

### 2.1 文本分析

```http
POST /realms/{realm_name}/food-analysis/text
Authorization: Bearer <token>
Content-Type: application/json
X-Device-Id: <device_id>
```

**请求体**

```json
{
  "prompt_id": "uuid-of-prompt",
  "text_input": "Grilled chicken, steamed vegetables, house salad"
}
```

**响应** (200 OK)

```json
{
  "request_id": "uuid-of-request",
  "result": {
    "id": "uuid-of-result",
    "request_id": "uuid-of-request",
    "dishes": [
      {
        "dish_name": "Grilled Chicken",
        "safety_level": "SAFE",
        "reason": "Low inflammatory risk",
        "ibd_concerns": [],
        "ibs_concerns": [],
        "recommendations": "Good choice",
        "ingredients": []
      }
    ],
    "created_at": "2025-11-29T00:00:00Z"
  },
  "items": [
    {
      "id": "uuid-of-item-1",
      "dish_name": "Grilled Chicken",
      "risk_score": 5,
      "risk_band": "SAFE",
      "safety_level": "SAFE",
      "summary_reason": "Low inflammatory risk"
    }
  ]
}
```

### 2.2 图片分析

```http
POST /realms/{realm_name}/food-analysis/image
Authorization: Bearer <token>
Content-Type: multipart/form-data
X-Device-Id: <device_id>
```

**请求体** (multipart/form-data)

- `prompt_id`: UUID (text field)
- `image`: File (binary)

**响应** (200 OK)

同 2.1 文本分析响应格式

### 2.3 获取分析请求列表

```http
GET /realms/{realm_name}/food-analysis/requests
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

**Query 参数**

**分页参数**：
- `offset` (int, optional, default 0, ≥ 0)
- `limit` (int, optional, default 20, > 0, max 100)

**Filter 参数**：
- `filter[prompt_id]` (uuid, optional) - 按 Prompt 筛选
- `filter[input_type]` (string, optional) - 按输入类型筛选：'image' | 'text'
- `filter[user_id]` (uuid, optional) - 按用户筛选（通常自动按 device_id 映射的 user_id 过滤）
- `filter[created_at][gte]` (datetime, optional) - 创建时间大于等于
- `filter[created_at][lte]` (datetime, optional) - 创建时间小于等于

**排序参数**：
- `sort` (string, optional, default '-created_at') - 排序字段，支持：
  - `created_at` / `-created_at` - 按创建时间
  - `updated_at` / `-updated_at` - 按更新时间
  - 多字段排序：`sort=-created_at,prompt_id`

**响应** (200 OK)

```json
{
  "items": [
    {
      "id": "uuid-of-request",
      "realm_id": "uuid-of-realm",
      "prompt_id": "uuid-of-prompt",
      "device_id": "ios-uuid",
      "user_id": "uuid-of-user",
      "input_type": "text",
      "input_content": "Grilled chicken, steamed vegetables",
      "created_at": "2025-11-29T00:00:00Z",
      "updated_at": "2025-11-29T00:00:00Z"
    }
  ],
  "offset": 0,
  "limit": 20,
  "count": 15
}
```

### 2.4 获取单个分析请求

```http
GET /realms/{realm_name}/food-analysis/requests/{request_id}
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

**响应** (200 OK)

```json
{
  "id": "uuid-of-request",
  "realm_id": "uuid-of-realm",
  "prompt_id": "uuid-of-prompt",
  "device_id": "ios-uuid",
  "user_id": "uuid-of-user",
  "input_type": "text",
  "input_content": "Grilled chicken, steamed vegetables",
  "created_at": "2025-11-29T00:00:00Z",
  "updated_at": "2025-11-29T00:00:00Z"
}
```

### 2.5 获取分析结果

```http
GET /realms/{realm_name}/food-analysis/requests/{request_id}/result
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

**响应** (200 OK)

```json
{
  "id": "uuid-of-result",
  "request_id": "uuid-of-request",
  "raw_response": "{...}",
  "dishes": [
    {
      "dish_name": "Grilled Chicken",
      "safety_level": "SAFE",
      "reason": "Low inflammatory risk",
      "ibd_concerns": [],
      "ibs_concerns": [],
      "recommendations": "Good choice",
      "ingredients": []
    }
  ],
  "created_at": "2025-11-29T00:00:00Z",
  "updated_at": "2025-11-29T00:00:00Z"
}
```

---

## 3. 食物分析项接口（菜单列表）

### 3.1 获取分析请求的所有菜品

```http
GET /realms/{realm_name}/food-analysis/requests/{request_id}/items
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

> **说明**：此接口返回的每个菜品都包含 `reaction_info` 字段，标识该菜品是否已有反应记录。
>
> **服务端实现建议**：
> 1. 通过 `X-Device-Id` 获取当前用户的 `user_id`
> 2. 使用 LEFT JOIN 或子查询优化性能，避免 N+1 查询问题
> 3. 统计反应记录数量和获取最新记录
>
> **SQL 查询示例**（使用 PostgreSQL）：
> ```sql
> SELECT
>   fai.*,
>   COALESCE(reaction_stats.reaction_count, 0) as reaction_count,
>   latest_reaction.id as latest_reaction_id,
>   latest_reaction.eaten_at as latest_reaction_eaten_at,
>   latest_reaction.feeling as latest_reaction_feeling,
>   latest_reaction.symptom_onset as latest_reaction_symptom_onset,
>   latest_reaction.created_at as latest_reaction_created_at,
>   latest_reaction.symptoms as latest_reaction_symptoms
> FROM food_analysis_items fai
> INNER JOIN food_analysis_requests far ON fai.request_id = far.id
> LEFT JOIN (
>   SELECT
>     analysis_item_id,
>     COUNT(*) as reaction_count
>   FROM food_reactions
>   WHERE realm_id = ? AND user_id = ?
>   GROUP BY analysis_item_id
> ) reaction_stats ON fai.id = reaction_stats.analysis_item_id
> LEFT JOIN LATERAL (
>   SELECT
>     fr.id,
>     fr.eaten_at,
>     fr.feeling,
>     fr.symptom_onset,
>     fr.created_at,
>     COALESCE(
>       ARRAY_AGG(frs.symptom_code ORDER BY frs.symptom_code) FILTER (WHERE frs.symptom_code IS NOT NULL),
>       ARRAY[]::TEXT[]
>     ) as symptoms
>   FROM food_reactions fr
>   LEFT JOIN food_reaction_symptoms frs ON fr.id = frs.reaction_id
>   WHERE fr.analysis_item_id = fai.id
>     AND fr.realm_id = ?
>     AND fr.user_id = ?
>   GROUP BY fr.id, fr.eaten_at, fr.feeling, fr.symptom_onset, fr.created_at
>   ORDER BY fr.created_at DESC
>   LIMIT 1
> ) latest_reaction ON true
> WHERE fai.realm_id = ?
>   AND fai.request_id = ?
>   AND far.user_id = ?  -- 确保只返回当前用户的数据
> ORDER BY fai.dish_index;
> ```
>
> **注意**：
> - 症状列表（`symptoms`）使用 `ARRAY_AGG` 聚合函数从 `food_reaction_symptoms` 表获取
> - 如果 `include_reaction_info=false`，可以省略所有反应记录相关的 JOIN，提升查询性能
> - 建议在应用层将查询结果转换为响应格式，包括构建 `reaction_info` 对象

**Query 参数**

**分页参数**：
- `offset` (int, optional, default 0, ≥ 0)
- `limit` (int, optional, default 20, > 0, max 100)

**Filter 参数**：
- `filter[risk_band]` (string, optional) - 按风险分组筛选：'SAFE' | 'MODERATE' | 'HIGH'
- `filter[risk_band][in]` (string, optional) - 多个风险分组，逗号分隔：'SAFE,MODERATE'
- `filter[safety_level]` (string, optional) - 按安全等级筛选：'SAFE' | 'CAUTION' | 'UNSAFE'
- `filter[risk_score][gte]` (int, optional) - 风险分数大于等于（0-100）
- `filter[risk_score][lte]` (int, optional) - 风险分数小于等于（0-100）
- `filter[dish_name][ilike]` (string, optional) - 按菜品名称模糊搜索（不区分大小写）

**其他参数**：
- `include_reaction_info` (boolean, optional, default true) - 是否包含反应记录信息。设置为 `false` 时，`reaction_info` 字段将不包含在响应中，可提升查询性能

**排序参数**：
- `sort` (string, optional, default 'dish_index') - 排序字段，支持：
  - `dish_index` / `-dish_index` - 按原始顺序
  - `risk_score` / `-risk_score` - 按风险分数
  - `dish_name` / `-dish_name` - 按菜品名称
  - `created_at` / `-created_at` - 按创建时间
  - 多字段排序：`sort=risk_band,-risk_score`

**响应** (200 OK)

```json
{
  "items": [
    {
      "id": "uuid-of-item",
      "dish_name": "Grilled Chicken",
      "risk_score": 5,
      "risk_band": "SAFE",
      "safety_level": "SAFE",
      "summary_reason": "Low inflammatory risk",
      "image_object_key": "images/grilled-chicken.jpg",
      "triggers": [
        {
          "id": "uuid-t1",
          "ingredient_name": "Plain chicken",
          "trigger_category": "Protein",
          "risk_level": "LOW"
        }
      ],
      "reaction_info": {
        "has_reaction": true,
        "reaction_count": 2,
        "latest_reaction": {
          "id": "uuid-of-reaction",
          "eaten_at": "2025-11-29T10:30:00Z",
          "feeling": "OKAY",
          "symptom_onset": "LT_1H",
          "symptoms": ["BLOATING"],
          "created_at": "2025-11-29T10:35:00Z"
        }
      },
      "created_at": "2025-11-29T00:00:00Z"
    },
    {
      "id": "uuid-of-item-2",
      "dish_name": "Fettuccine Alfredo",
      "risk_score": 88,
      "risk_band": "HIGH",
      "safety_level": "UNSAFE",
      "summary_reason": "Multiple inflammatory triggers",
      "image_object_key": "images/fettuccine.jpg",
      "triggers": [],
      "reaction_info": {
        "has_reaction": false,
        "reaction_count": 0,
        "latest_reaction": null
      },
      "created_at": "2025-11-29T00:00:00Z"
    }
  ],
  "offset": 0,
  "limit": 20,
  "count": 4
}
```

**响应字段说明**：

- `reaction_info` (object) - 反应记录信息
  - `has_reaction` (boolean) - 该菜品是否有反应记录（当前用户）
  - `reaction_count` (int) - 该菜品的反应记录总数（当前用户）
  - `latest_reaction` (object | null) - 最新的反应记录（如果存在），包含：
    - `id` (uuid) - 反应记录 ID
    - `eaten_at` (datetime) - 进食时间
    - `feeling` (string) - 感觉：'GREAT' | 'OKAY' | 'MILD_ISSUES' | 'BAD'
    - `symptom_onset` (string) - 症状出现时间：'LT_1H' | 'H1_3H' | 'H3_6H' | 'NEXT_DAY'
    - `symptoms` (array) - 症状列表
    - `created_at` (datetime) - 创建时间

> **说明**：
> - 前端根据 `risk_band` 做分组：Safe Options / Moderate Risk / High Risk，并展示 `risk_score%`
> - `reaction_info.has_reaction` 可用于 UI 显示是否已记录反应（如显示已记录图标）
> - `reaction_info.latest_reaction` 可用于显示最新的反应信息，帮助用户快速了解该菜品的历史反应

### 3.2 获取单个菜品详情

```http
GET /realms/{realm_name}/food-analysis/items/{item_id}
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

**响应** (200 OK)

```json
{
  "item": {
    "id": "uuid-of-item",
    "request_id": "uuid-of-request",
    "result_id": "uuid-of-result",
    "dish_index": 0,
    "dish_name": "Fettuccine Alfredo",
    "risk_score": 80,
    "risk_band": "HIGH",
    "safety_level": "UNSAFE",
    "summary_reason": "Multiple inflammatory triggers detected",
    "ibd_concerns": ["High fat may irritate IBD"],
    "ibs_concerns": ["FODMAP triggers from garlic"],
    "recommendations": "Avoid or choose lighter sauce",
    "image_object_key": "images/fettuccine.jpg",
    "reaction_info": {
      "has_reaction": true,
      "reaction_count": 3,
      "latest_reaction": {
        "id": "uuid-of-reaction",
        "eaten_at": "2025-11-29T12:00:00Z",
        "feeling": "BAD",
        "symptom_onset": "H1_3H",
        "symptoms": ["BLOATING", "PAIN", "GAS"],
        "created_at": "2025-11-29T12:05:00Z"
      }
    },
    "created_at": "2025-11-29T00:00:00Z",
    "updated_at": "2025-11-29T00:00:00Z"
  },
  "triggers": [
    {
      "id": "uuid-t1",
      "ingredient_name": "Heavy Cream",
      "trigger_category": "Dairy",
      "risk_level": "HIGH",
      "risk_reason": "High lactose content"
    },
    {
      "id": "uuid-t2",
      "ingredient_name": "Butter",
      "trigger_category": "Lactose",
      "risk_level": "HIGH",
      "risk_reason": "Dairy product"
    },
    {
      "id": "uuid-t3",
      "ingredient_name": "Garlic",
      "trigger_category": "FODMAP",
      "risk_level": "MEDIUM",
      "risk_reason": "High FODMAP content"
    }
  ]
}
```

**响应字段说明**：

- `item.reaction_info` (object) - 反应记录信息（同 3.1 接口）
  - `has_reaction` (boolean) - 该菜品是否有反应记录（当前用户）
  - `reaction_count` (int) - 该菜品的反应记录总数（当前用户）
  - `latest_reaction` (object | null) - 最新的反应记录（如果存在）

> **说明**：
> - 按钮「Avoid This Food」只需前端根据 `risk_band === "HIGH"` 决定显隐/样式，后端无需额外字段
> - `reaction_info` 字段帮助用户了解该菜品的历史反应情况，可用于 UI 展示和决策参考

### 3.3 获取菜品列表（跨请求查询）

```http
GET /realms/{realm_name}/food-analysis/items
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

**Query 参数**

**分页参数**：
- `offset` (int, optional, default 0, ≥ 0)
- `limit` (int, optional, default 20, > 0, max 100)

**Filter 参数**：
- `filter[risk_band]` (string, optional) - 按风险分组筛选：'SAFE' | 'MODERATE' | 'HIGH'
- `filter[risk_band][in]` (string, optional) - 多个风险分组，逗号分隔
- `filter[safety_level]` (string, optional) - 按安全等级筛选：'SAFE' | 'CAUTION' | 'UNSAFE'
- `filter[request_id]` (uuid, optional) - 按分析请求筛选
- `filter[risk_score][gte]` (int, optional) - 风险分数大于等于（0-100）
- `filter[risk_score][lte]` (int, optional) - 风险分数小于等于（0-100）
- `filter[dish_name][ilike]` (string, optional) - 按菜品名称模糊搜索（不区分大小写）
- `filter[created_at][gte]` (datetime, optional) - 创建时间大于等于
- `filter[created_at][lte]` (datetime, optional) - 创建时间小于等于

**其他参数**：
- `include_reaction_info` (boolean, optional, default true) - 是否包含反应记录信息。设置为 `false` 时，`reaction_info` 字段将不包含在响应中，可提升查询性能

**排序参数**：
- `sort` (string, optional, default '-created_at') - 排序字段，支持：
  - `risk_score` / `-risk_score` - 按风险分数
  - `dish_name` / `-dish_name` - 按菜品名称
  - `created_at` / `-created_at` - 按创建时间
  - `risk_band` / `-risk_band` - 按风险分组
  - 多字段排序：`sort=risk_band,-risk_score,created_at`

**响应** (200 OK)

```json
{
  "items": [
    {
      "id": "uuid-of-item",
      "request_id": "uuid-of-request",
      "dish_name": "Grilled Chicken",
      "risk_score": 5,
      "risk_band": "SAFE",
      "safety_level": "SAFE",
      "summary_reason": "Low inflammatory risk",
      "image_object_key": "images/grilled-chicken.jpg",
      "reaction_info": {
        "has_reaction": true,
        "reaction_count": 2,
        "latest_reaction": {
          "id": "uuid-of-reaction",
          "eaten_at": "2025-11-29T10:30:00Z",
          "feeling": "OKAY",
          "symptom_onset": "LT_1H",
          "symptoms": ["BLOATING"],
          "created_at": "2025-11-29T10:35:00Z"
        }
      },
      "created_at": "2025-11-29T00:00:00Z"
    }
  ],
  "offset": 0,
  "limit": 20,
  "count": 45
}
```

**响应字段说明**：

- `reaction_info` (object) - 反应记录信息（同 3.1 接口）
  - `has_reaction` (boolean) - 该菜品是否有反应记录（当前用户）
  - `reaction_count` (int) - 该菜品的反应记录总数（当前用户）
  - `latest_reaction` (object | null) - 最新的反应记录（如果存在）

---

## 4. 触发成分接口

### 4.1 获取菜品的触发成分列表

```http
GET /realms/{realm_name}/food-analysis/items/{item_id}/triggers
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

**Query 参数**

**分页参数**：
- `offset` (int, optional, default 0, ≥ 0)
- `limit` (int, optional, default 20, > 0, max 100)

**Filter 参数**：
- `filter[trigger_category]` (string, optional) - 按触发分类筛选
- `filter[risk_level]` (string, optional) - 按风险等级筛选：'HIGH' | 'MEDIUM' | 'LOW'
- `filter[risk_level][in]` (string, optional) - 多个风险等级，逗号分隔
- `filter[ingredient_name][ilike]` (string, optional) - 按成分名称模糊搜索

**排序参数**：
- `sort` (string, optional, default 'risk_level,-created_at') - 排序字段，支持：
  - `risk_level` / `-risk_level` - 按风险等级（HIGH > MEDIUM > LOW）
  - `ingredient_name` / `-ingredient_name` - 按成分名称
  - `trigger_category` / `-trigger_category` - 按分类
  - `created_at` / `-created_at` - 按创建时间

**响应** (200 OK)

```json
{
  "items": [
    {
      "id": "uuid-t1",
      "item_id": "uuid-of-item",
      "ingredient_name": "Heavy Cream",
      "trigger_category": "Dairy",
      "risk_level": "HIGH",
      "risk_reason": "High lactose content",
      "created_at": "2025-11-29T00:00:00Z"
    },
    {
      "id": "uuid-t2",
      "item_id": "uuid-of-item",
      "ingredient_name": "Butter",
      "trigger_category": "Lactose",
      "risk_level": "HIGH",
      "risk_reason": "Dairy product",
      "created_at": "2025-11-29T00:00:00Z"
    }
  ]
}
```

### 4.2 按分类统计触发成分

```http
GET /realms/{realm_name}/food-analysis/triggers/categories
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

**Query 参数**

**分页参数**：
- `offset` (int, optional, default 0, ≥ 0)
- `limit` (int, optional, default 20, > 0, max 100)

**Filter 参数**：
- `filter[trigger_category]` (string, optional) - 按分类筛选
- `filter[trigger_category][in]` (string, optional) - 多个分类，逗号分隔
- `filter[trigger_category][ilike]` (string, optional) - 按分类名称模糊搜索

**排序参数**：
- `sort` (string, optional, default '-count') - 排序字段，支持：
  - `trigger_category` / `-trigger_category` - 按分类名称
  - `count` / `-count` - 按数量
  - `high_risk_count` / `-high_risk_count` - 按高风险数量

**响应** (200 OK)

```json
{
  "items": [
    {
      "trigger_category": "Dairy",
      "count": 15,
      "high_risk_count": 8,
      "medium_risk_count": 5,
      "low_risk_count": 2
    },
    {
      "trigger_category": "FODMAP",
      "count": 12,
      "high_risk_count": 3,
      "medium_risk_count": 7,
      "low_risk_count": 2
    }
  ]
}
```

---

## 5. 反应记录接口

### 5.1 创建反应记录

```http
POST /realms/{realm_name}/food-reactions
Authorization: Bearer <token>
Content-Type: application/json
X-Device-Id: <device_id>
```

**请求体**

```json
{
  "analysis_item_id": "uuid-of-item",   // 可选，若用户手动记录则可为 null
  "eaten_at": "2025-11-29T10:30:00Z",
  "feeling": "OKAY",                    // GREAT | OKAY | MILD_ISSUES | BAD
  "symptom_onset": "LT_1H",            // LT_1H | H1_3H | H3_6H | NEXT_DAY
  "symptoms": ["BLOATING", "PAIN"],    // 零个或多个：BLOATING | PAIN | GAS | URGENCY | NAUSEA | CRAMPING | OTHER
  "notes": "Slight bloating but overall fine"
}
```

**服务端流程**：
1. 通过 `realm_name + device_id` 在 `device_profiles` 中找到（或创建）对应 `user_id`
2. 写入 `food_reactions` 与 `food_reaction_symptoms`

**响应** (201 Created)

```json
{
  "id": "uuid-of-reaction",
  "realm_id": "uuid-of-realm",
  "device_id": "ios-uuid",
  "user_id": "uuid-of-user",
  "analysis_item_id": "uuid-of-item",
  "eaten_at": "2025-11-29T10:30:00Z",
  "feeling": "OKAY",
  "symptom_onset": "LT_1H",
  "notes": "Slight bloating but overall fine",
  "symptoms": ["BLOATING", "PAIN"],
  "created_at": "2025-11-29T10:35:00Z",
  "updated_at": "2025-11-29T10:35:00Z"
}
```

### 5.2 获取反应记录列表

```http
GET /realms/{realm_name}/food-reactions
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

**Query 参数**

**分页参数**：
- `offset` (int, optional, default 0, ≥ 0)
- `limit` (int, optional, default 20, > 0, max 100)

**Filter 参数**：
- `filter[feeling]` (string, optional) - 按感觉筛选：'GREAT' | 'OKAY' | 'MILD_ISSUES' | 'BAD'
- `filter[feeling][in]` (string, optional) - 多个感觉，逗号分隔：'MILD_ISSUES,BAD'
- `filter[analysis_item_id]` (uuid, optional) - 按菜品筛选
- `filter[symptom_onset]` (string, optional) - 按症状出现时间筛选：'LT_1H' | 'H1_3H' | 'H3_6H' | 'NEXT_DAY'
- `filter[eaten_at][gte]` (datetime, optional) - 进食时间大于等于（ISO 8601）
- `filter[eaten_at][lte]` (datetime, optional) - 进食时间小于等于（ISO 8601）
- `filter[created_at][gte]` (datetime, optional) - 创建时间大于等于
- `filter[created_at][lte]` (datetime, optional) - 创建时间小于等于
- `filter[has_symptoms]` (boolean, optional) - 是否有症状（true/false）

**排序参数**：
- `sort` (string, optional, default '-eaten_at') - 排序字段，支持：
  - `eaten_at` / `-eaten_at` - 按进食时间
  - `created_at` / `-created_at` - 按创建时间
  - `feeling` / `-feeling` - 按感觉（BAD > MILD_ISSUES > OKAY > GREAT）
  - 多字段排序：`sort=-eaten_at,feeling`

**响应** (200 OK)

```json
{
  "items": [
    {
      "id": "uuid-of-reaction",
      "realm_id": "uuid-of-realm",
      "device_id": "ios-uuid",
      "user_id": "uuid-of-user",
      "analysis_item_id": "uuid-of-item",
      "eaten_at": "2025-11-29T10:30:00Z",
      "feeling": "OKAY",
      "symptom_onset": "LT_1H",
      "notes": "Slight bloating but overall fine",
      "symptoms": ["BLOATING"],
      "analysis_item": {
        "id": "uuid-of-item",
        "dish_name": "Caesar Salad",
        "risk_score": 5,
        "risk_band": "SAFE"
      },
      "created_at": "2025-11-29T10:35:00Z",
      "updated_at": "2025-11-29T10:35:00Z"
    }
  ],
  "offset": 0,
  "limit": 20,
  "count": 23
}
```

### 5.3 获取单个反应记录

```http
GET /realms/{realm_name}/food-reactions/{reaction_id}
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

**响应** (200 OK)

```json
{
  "id": "uuid-of-reaction",
  "realm_id": "uuid-of-realm",
  "device_id": "ios-uuid",
  "user_id": "uuid-of-user",
  "analysis_item_id": "uuid-of-item",
  "eaten_at": "2025-11-29T10:30:00Z",
  "feeling": "OKAY",
  "symptom_onset": "LT_1H",
  "notes": "Slight bloating but overall fine",
  "symptoms": ["BLOATING", "PAIN"],
  "analysis_item": {
    "id": "uuid-of-item",
    "dish_name": "Caesar Salad",
    "risk_score": 5,
    "risk_band": "SAFE",
    "safety_level": "SAFE"
  },
  "created_at": "2025-11-29T10:35:00Z",
  "updated_at": "2025-11-29T10:35:00Z"
}
```

### 5.4 更新反应记录

```http
PUT /realms/{realm_name}/food-reactions/{reaction_id}
Authorization: Bearer <token>
Content-Type: application/json
X-Device-Id: <device_id>
```

**请求体**

```json
{
  "feeling": "MILD_ISSUES",
  "symptom_onset": "H1_3H",
  "symptoms": ["BLOATING", "GAS"],
  "notes": "Updated notes"
}
```

**响应** (200 OK)

同 5.3 获取单个反应记录响应格式

### 5.5 删除反应记录

```http
DELETE /realms/{realm_name}/food-reactions/{reaction_id}
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

**响应** (204 No Content)

---

## 6. 统计接口

### 6.1 获取个人触发统计概览

```http
GET /realms/{realm_name}/food-stats/overview
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

**响应** (200 OK)

```json
{
  "accuracy_level": 78,             // 0-100
  "target_accuracy": 85,            // 目标准确度
  "meals_to_target": 7,             // 为达到目标还需记录的餐次数估算

  "tracked_reactions": 23,
  "triggered_foods": 3,

  "triggers": [
    {
      "trigger_category": "Dairy Products",
      "emoji": "🥛",
      "issue_count": 8,
      "total_exposures": 9,
      "risk_percent": 89
    },
    {
      "trigger_category": "Garlic",
      "emoji": "🧄",
      "issue_count": 6,
      "total_exposures": 7,
      "risk_percent": 86
    },
    {
      "trigger_category": "Coffee",
      "emoji": "☕️",
      "issue_count": 4,
      "total_exposures": 6,
      "risk_percent": 67
    }
  ],

  "safe_foods": [
    {
      "trigger_category": "Low-FODMAP Veggies",
      "emoji": "🥦",
      "safe_exposures": 12
    }
  ]
}
```

> `emoji` 字段可以在服务端根据 `trigger_category` 简单映射，也可交由前端本地映射。

### 6.2 获取症状统计

```http
GET /realms/{realm_name}/food-stats/symptoms
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

**Query 参数**

**Filter 参数**：
- `filter[start_date]` (datetime, optional) - 开始日期（ISO 8601）
- `filter[end_date]` (datetime, optional) - 结束日期（ISO 8601）
- `filter[symptom_code]` (string, optional) - 按症状代码筛选
- `filter[symptom_code][in]` (string, optional) - 多个症状，逗号分隔

**排序参数**：
- `sort` (string, optional, default '-count') - 排序字段，支持：
  - `symptom_code` / `-symptom_code` - 按症状代码
  - `count` / `-count` - 按出现次数
  - `percentage` / `-percentage` - 按百分比

**响应** (200 OK)

```json
{
  "items": [
    {
      "symptom_code": "BLOATING",
      "count": 15,
      "percentage": 65.2
    },
    {
      "symptom_code": "PAIN",
      "count": 8,
      "percentage": 34.8
    },
    {
      "symptom_code": "GAS",
      "count": 5,
      "percentage": 21.7
    }
  ],
  "total_reactions": 23
}
```

### 6.3 获取时间序列统计

```http
GET /realms/{realm_name}/food-stats/timeline
Authorization: Bearer <token>
X-Device-Id: <device_id>
```

**Query 参数**

**Filter 参数**（必需）：
- `filter[start_date]` (datetime, required) - 开始日期（ISO 8601）
- `filter[end_date]` (datetime, required) - 结束日期（ISO 8601）
- `filter[granularity]` (string, optional, default 'day') - 时间粒度：'day' | 'week' | 'month'
- `filter[feeling][in]` (string, optional) - 按感觉筛选，逗号分隔

**排序参数**：
- `sort` (string, optional, default 'date') - 排序字段，支持：
  - `date` / `-date` - 按日期
  - `total_reactions` / `-total_reactions` - 按总反应数
  - `positive_reactions` / `-positive_reactions` - 按正面反应数
  - `negative_reactions` / `-negative_reactions` - 按负面反应数

**响应** (200 OK)

```json
{
  "items": [
    {
      "date": "2025-11-29",
      "total_reactions": 3,
      "positive_reactions": 1,
      "negative_reactions": 2
    },
    {
      "date": "2025-11-30",
      "total_reactions": 5,
      "positive_reactions": 3,
      "negative_reactions": 2
    }
  ],
  "start_date": "2025-11-29T00:00:00Z",
  "end_date": "2025-12-05T23:59:59Z"
}
```

---

## 7. 错误响应格式

所有接口在发生错误时，统一返回以下格式：

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {}  // 可选，包含额外错误详情
  }
}
```

**常见错误码**：
- `BAD_REQUEST` (400) - 请求参数错误
- `UNAUTHORIZED` (401) - 未授权
- `FORBIDDEN` (403) - 禁止访问
- `NOT_FOUND` (404) - 资源未找到
- `INTERNAL_SERVER_ERROR` (500) - 服务器内部错误
- `VALIDATION_ERROR` (400) - 数据验证失败

**示例错误响应**：

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid pagination parameters",
    "details": {
      "offset": "must be >= 0",
      "limit": "must be > 0 and <= 100"
    }
  }
}
```

---

## 8. 接口设计原则总结

### 8.1 RESTful 设计原则

1. **资源命名**：使用名词复数，如 `/food-analysis/requests`、`/food-reactions`
2. **HTTP 方法**：
   - `GET` - 查询资源
   - `POST` - 创建资源
   - `PUT` - 完整更新资源
   - `PATCH` - 部分更新资源（如需要）
   - `DELETE` - 删除资源
3. **资源层级**：使用嵌套资源表示关系，如 `/food-analysis/requests/{request_id}/items`
4. **状态码**：正确使用 HTTP 状态码表示操作结果

### 8.2 分页规范

所有列表接口统一使用：
- `offset` (int, ≥ 0, default 0) - 偏移量
- `limit` (int, > 0, default 20, max 100) - 每页数量
- 响应包含：`items`（数组）、`offset`、`limit`、`count`（总数）

### 8.2.1 Filter 查询规范

所有列表接口支持 filter 查询，格式：`filter[field_name][operator]=value`

**支持的运算符**：
- `eq` - 等于（默认，可省略）：`filter[field]=value` 等同于 `filter[field][eq]=value`
- `ne` - 不等于：`filter[field][ne]=value`
- `gt` - 大于：`filter[field][gt]=value`
- `gte` - 大于等于：`filter[field][gte]=value`
- `lt` - 小于：`filter[field][lt]=value`
- `lte` - 小于等于：`filter[field][lte]=value`
- `in` - 在列表中：`filter[field][in]=value1,value2,value3`
- `like` - 模糊匹配（区分大小写）：`filter[field][like]=%pattern%`
- `ilike` - 模糊匹配（不区分大小写，PostgreSQL）：`filter[field][ilike]=%pattern%`

**多条件组合**：多个 filter 参数之间为 AND 关系

**示例**：
```
GET /realms/{realm_name}/food-analysis/items?filter[risk_band]=SAFE
GET /realms/{realm_name}/food-analysis/items?filter[risk_score][gte]=50&filter[risk_score][lte]=80
GET /realms/{realm_name}/food-analysis/items?filter[dish_name][ilike]=%chicken%
GET /realms/{realm_name}/food-reactions?filter[feeling][in]=MILD_ISSUES,BAD
GET /realms/{realm_name}/food-analysis/items?filter[created_at][gte]=2025-11-01T00:00:00Z
```

### 8.2.2 排序规范

所有列表接口支持排序，格式：`sort=field_name` 或 `sort=-field_name`

- `sort=field_name` - 升序（ASC）
- `sort=-field_name` - 降序（DESC）
- 多字段排序：`sort=field1,-field2`（逗号分隔，按顺序应用）

**默认排序**：每个接口都有默认排序（通常在接口文档中说明），如果没有指定 `sort` 参数，使用默认排序

**示例**：
```
GET /realms/{realm_name}/food-analysis/items?sort=-risk_score
GET /realms/{realm_name}/food-analysis/items?sort=risk_band,-risk_score,created_at
GET /realms/{realm_name}/food-reactions?sort=-eaten_at,feeling
```

### 8.3 设备标识与认证

- **设备标识**：所有接口通过 `X-Device-Id` Header 传递设备标识
- **自动映射**：服务端自动处理设备到用户的映射（通过 `device_profiles` 表）
- **自动创建**：如果 `X-Device-Id` 没有绑定的用户，服务端默认创建一个匿名用户并绑定设备
- **用户隔离**：所有查询操作自动按 `user_id` 过滤，确保数据隔离
- **实现流程**：
  1. 从 `X-Device-Id` Header 获取设备标识
  2. 查询 `device_profiles` 表获取 `user_id`
  3. 如果不存在，创建匿名用户并创建设备配置
  4. 使用 `user_id` 进行后续所有操作

### 8.4 多租户支持

- 所有接口路径包含 `{realm_name}`，实现多租户隔离
- 服务端通过 `Identity` 验证用户对 realm 的访问权限
- 所有数据查询自动按 `realm_id` 过滤

### 8.5 审计字段

所有资源响应包含审计字段（如适用）：
- `created_at` - 创建时间
- `updated_at` - 更新时间
- `created_by` - 创建者（可选）
- `updated_by` - 更新者（可选）

---

## 9. 与现有 food_analysis 能力的集成要点

- **分析请求与结果**：沿用现有表 `food_analysis_requests` 与 `food_analysis_results`，新增的 `food_analysis_items` 与 `food_analysis_triggers` 由服务层在每次分析完成时自动生成。
- **多租户与审计**：
  - 所有新表均带 `realm_id`，通过 `Identity` 中的 realm 权限做授权校验。
  - 审计字段 `created_at/updated_at/created_by/updated_by` 方便后续后台运营面板。
- **设备与用户**：
  - 移动端只需关心 `device_id`，通过 `X-Device-Id` Header 传递。
  - 服务端通过 `device_profiles` 保持与 FerrisKey `users` 的一一映射。
  - **自动创建机制**：如果 `X-Device-Id` 没有绑定的用户，服务端自动创建匿名用户并绑定设备，无需客户端手动注册。
- **分页规范**：
  - 所有列表接口统一 `offset ≥ 0`，`limit > 0`（默认 20，最大 100），响应携带 `items, offset, limit, count`。
- **Filter 与排序**：
  - 所有列表接口支持 filter 查询（使用 `filter[field][operator]=value` 格式）。
  - 所有列表接口支持排序（使用 `sort=field` 或 `sort=-field` 格式）。
  - Filter 和排序参数在接口文档中详细说明。

通过以上数据库与接口设计，即可完整支撑四个关键 UI：从菜单/食物分析，到反应记录，再到个性化触发统计。
