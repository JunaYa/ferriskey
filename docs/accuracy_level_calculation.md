# Accuracy Level 计算实现方案

## 概述

`accuracy_level` 用于衡量模型预测风险与实际用户反馈的吻合度，范围 0-100。

## 数据映射规则

### 模型预测（`food_analysis_items.risk_band`）

将风险分组映射为布尔值「是否预测会引发问题」：

- `HIGH` → 预测会引发问题（`true`）
- `MODERATE` → 预测会引发问题（`true`）
- `SAFE` → 预测不会引发问题（`false`）

### 用户反馈（`food_reactions.feeling`）

将用户感觉映射为布尔值「是否实际引发问题」：

- `MILD_ISSUES` → 实际引发了问题（`true`）
- `BAD` → 实际引发了问题（`true`）
- `GREAT` → 实际没有引发问题（`false`）
- `OKAY` → 实际没有引发问题（`false`）

## 计算逻辑

### 1. 数据范围

只统计满足以下条件的反应记录：
- `food_reactions.analysis_item_id IS NOT NULL`（必须关联到分析项）
- `food_reactions.realm_id = ?`（指定租户）
- `food_reactions.user_id = ?`（指定用户）

### 2. 匹配判断

对于每条反应记录，判断预测与反馈是否匹配：

```rust
fn is_prediction_match(risk_band: &str, feeling: &str) -> bool {
    let predicted_issue = matches!(risk_band, "HIGH" | "MODERATE");
    let actual_issue = matches!(feeling, "MILD_ISSUES" | "BAD");
    predicted_issue == actual_issue
}
```

### 3. 准确率计算

```
accuracy_level = (匹配的记录数 / 总记录数) * 100
```

如果总记录数为 0，返回 0 或特殊值（如 -1 表示数据不足）。

## SQL 实现示例

```sql
WITH reaction_items AS (
  SELECT
    fr.id as reaction_id,
    fai.risk_band,
    fr.feeling
  FROM food_reactions fr
  INNER JOIN food_analysis_items fai ON fr.analysis_item_id = fai.id
  WHERE fr.realm_id = $1
    AND fr.user_id = $2
    AND fr.analysis_item_id IS NOT NULL
),
accuracy_calc AS (
  SELECT
    COUNT(*) as total_count,
    SUM(
      CASE
        WHEN (risk_band IN ('HIGH', 'MODERATE') AND feeling IN ('MILD_ISSUES', 'BAD'))
          OR (risk_band = 'SAFE' AND feeling IN ('GREAT', 'OKAY'))
        THEN 1
        ELSE 0
      END
    ) as matched_count
  FROM reaction_items
)
SELECT
  CASE
    WHEN total_count = 0 THEN 0
    ELSE ROUND((matched_count::numeric / total_count::numeric) * 100)
  END as accuracy_level
FROM accuracy_calc;
```

## Rust 实现建议

### 1. 在 Repository 层添加查询方法

```rust
// core/src/domain/food_analysis/ports.rs
#[cfg_attr(test, mockall::automock)]
pub trait FoodAnalysisRepository: Send + Sync {
    // ... existing methods ...

    fn calculate_accuracy_level(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Option<u8>, CoreError>> + Send;
}
```

### 2. 在 Service 层实现计算逻辑

```rust
// core/src/domain/food_analysis/services.rs
async fn calculate_accuracy_level(
    &self,
    realm_id: Uuid,
    user_id: Uuid,
) -> Result<u8, CoreError> {
    // 查询所有有 analysis_item_id 的反应记录及其关联的分析项
    let reactions_with_items = self
        .food_analysis_repository
        .get_reactions_with_items(realm_id, user_id)
        .await?;

    if reactions_with_items.is_empty() {
        return Ok(0); // 或返回特殊值表示数据不足
    }

    let mut matched = 0;
    let mut total = 0;

    for (risk_band, feeling) in reactions_with_items {
        total += 1;

        let predicted_issue = matches!(risk_band.as_str(), "HIGH" | "MODERATE");
        let actual_issue = matches!(feeling.as_str(), "MILD_ISSUES" | "BAD");

        if predicted_issue == actual_issue {
            matched += 1;
        }
    }

    let accuracy = if total > 0 {
        ((matched as f64 / total as f64) * 100.0).round() as u8
    } else {
        0
    };

    Ok(accuracy)
}
```

### 3. 优化：使用 SQL 聚合计算

更高效的方式是在数据库层直接计算：

```rust
// core/src/infrastructure/food_analysis/repositories/food_analysis_repository.rs
async fn calculate_accuracy_level(
    &self,
    realm_id: Uuid,
    user_id: Uuid,
) -> Result<Option<u8>, CoreError> {
    let stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"
        WITH reaction_items AS (
          SELECT
            fr.id as reaction_id,
            fai.risk_band,
            fr.feeling
          FROM food_reactions fr
          INNER JOIN food_analysis_items fai ON fr.analysis_item_id = fai.id
          WHERE fr.realm_id = $1
            AND fr.user_id = $2
            AND fr.analysis_item_id IS NOT NULL
        ),
        accuracy_calc AS (
          SELECT
            COUNT(*) as total_count,
            SUM(
              CASE
                WHEN (risk_band IN ('HIGH', 'MODERATE') AND feeling IN ('MILD_ISSUES', 'BAD'))
                  OR (risk_band = 'SAFE' AND feeling IN ('GREAT', 'OKAY'))
                THEN 1
                ELSE 0
              END
            ) as matched_count
          FROM reaction_items
        )
        SELECT
          CASE
            WHEN total_count = 0 THEN 0
            ELSE ROUND((matched_count::numeric / total_count::numeric) * 100)::int
          END as accuracy_level
        FROM accuracy_calc
        "#,
        [realm_id.into(), user_id.into()],
    );

    let result = self.db.query_one(stmt).await?;

    match result {
        Some(row) => {
            let accuracy: Option<i32> = row.try_get("accuracy_level").ok();
            Ok(accuracy.map(|v| v as u8))
        }
        None => Ok(None),
    }
}
```

## 边界情况处理

1. **无反应记录**：返回 `0` 或 `None`（表示数据不足）
2. **所有反应记录都没有 `analysis_item_id`**：返回 `0` 或 `None`
3. **除零错误**：在 SQL 中使用 `CASE WHEN total_count = 0` 处理

## 性能优化建议

1. **索引**：确保 `food_reactions(analysis_item_id)` 有索引
2. **缓存**：如果准确率计算频繁，可以考虑缓存结果（TTL 如 5-10 分钟）
3. **增量计算**：如果数据量大，可以考虑维护一个统计表，在每次创建/更新反应记录时更新

## 测试用例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accuracy_calculation() {
        // 测试用例 1: 完全匹配
        // HIGH + BAD = 匹配
        // SAFE + OKAY = 匹配
        // 准确率 = 100%

        // 测试用例 2: 完全不匹配
        // HIGH + OKAY = 不匹配
        // SAFE + BAD = 不匹配
        // 准确率 = 0%

        // 测试用例 3: 部分匹配
        // HIGH + BAD = 匹配
        // HIGH + OKAY = 不匹配
        // SAFE + OKAY = 匹配
        // 准确率 = 66.67% ≈ 67%

        // 测试用例 4: 无数据
        // 返回 0 或 None
    }
}
```


1）、映射规则：
模型预测：risk_band 为 HIGH 或 MODERATE → 预测有问题；SAFE → 预测无问题
用户反馈：feeling 为 MILD_ISSUES 或 BAD → 实际有问题；GREAT 或 OKAY → 实际无问题

2）、映射规则：
accuracy_level = (预测与反馈匹配的记录数 / 总记录数) × 100

3）、数据范围：
仅统计 analysis_item_id IS NOT NULL 的反应记录（关联到分析项）
按 realm_id 和 user_id 过滤
