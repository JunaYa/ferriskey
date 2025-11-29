### 4.3 `food_reaction_symptoms` 独立表设计分析

#### 4.3.1 当前设计：独立表方案

**优势：**

1. **数据规范化与完整性**
   - ✅ 符合第三范式（3NF），避免数据冗余
   - ✅ 通过 `UNIQUE (reaction_id, symptom_code)` 约束防止重复症状
   - ✅ 通过 `CHECK` 约束保证症状代码的有效性
   - ✅ 外键约束保证数据一致性（`ON DELETE CASCADE`）

2. **查询性能优势**
   - ✅ **按症状统计查询高效**：
     ```sql
     -- 统计某个症状出现的次数（用于统计页面）
     SELECT symptom_code, COUNT(*)
     FROM food_reaction_symptoms
     WHERE reaction_id IN (
         SELECT id FROM food_reactions WHERE user_id = ? AND realm_id = ?
     )
     GROUP BY symptom_code;
     ```
   - ✅ 可在 `symptom_code` 上建立索引，支持快速聚合
   - ✅ JOIN 查询时，PostgreSQL 优化器可以高效处理

3. **扩展性**
   - ✅ 未来如需为症状添加额外属性（如严重程度、持续时间），只需在 `food_reaction_symptoms` 表中添加字段
   - ✅ 支持症状的审计字段（`created_at`, `updated_at`, `created_by`, `updated_by`），便于追踪变更历史
   - ✅ 如果未来需要症状与触发成分的关联分析，独立表更容易扩展

4. **统计与分析能力**
   - ✅ **症状频率分析**：可以轻松统计「哪些症状最常见」
   - ✅ **症状组合分析**：可以分析「BLOATING + PAIN」同时出现的频率
   - ✅ **时间序列分析**：结合 `food_reactions.eaten_at`，可以分析症状随时间的变化趋势
   - ✅ **关联分析**：可以关联 `food_analysis_triggers`，分析「特定触发成分 → 特定症状」的关联性

5. **代码维护性**
   - ✅ Rust 代码中可以使用类型安全的枚举和结构体
   - ✅ Sea-ORM 可以生成清晰的实体关系映射
   - ✅ 业务逻辑清晰：症状是独立的实体，不是简单的字符串数组

**劣势：**

1. **写入复杂度**
   - ❌ 创建反应记录时需要两次写入：先写 `food_reactions`，再写多条 `food_reaction_symptoms`
   - ❌ 需要事务保证原子性（但这是标准做法）

2. **查询复杂度**
   - ❌ 读取反应记录时需要 JOIN 或额外查询：
     ```sql
     -- 需要 JOIN 或两次查询
     SELECT r.*, array_agg(s.symptom_code) as symptoms
     FROM food_reactions r
     LEFT JOIN food_reaction_symptoms s ON r.id = s.reaction_id
     WHERE r.id = ?
     GROUP BY r.id;
     ```
   - ❌ 对于「只显示反应列表，不关心症状详情」的场景，会有轻微性能开销

3. **存储开销**
   - ❌ 每条症状需要额外的 UUID（`id`）和审计字段，存储空间略大
   - ❌ 对于「平均每个反应只有 1-2 个症状」的场景，独立表的开销相对明显

#### 4.3.2 替代方案：PostgreSQL 数组字段

如果使用数组字段，`food_reactions` 表结构如下：

```sql
CREATE TABLE food_reactions (
    -- ... 其他字段 ...
    symptoms TEXT[] NOT NULL DEFAULT '{}',
    CONSTRAINT check_symptoms_array CHECK (
        -- 使用 PostgreSQL 的数组操作符验证
        symptoms <@ ARRAY['BLOATING', 'PAIN', 'GAS', 'URGENCY', 'NAUSEA', 'CRAMPING', 'OTHER']::TEXT[]
    )
);
```

**数组方案的优势：**

1. ✅ **写入简单**：一次 INSERT 即可完成
2. ✅ **读取简单**：直接 SELECT，无需 JOIN
3. ✅ **存储高效**：无额外 UUID 和审计字段开销
4. ✅ **PostgreSQL 数组支持**：可以使用 `ANY`, `@>`, `<@` 等操作符进行查询

**数组方案的劣势：**

1. ❌ **统计查询性能差**：
   ```sql
   -- 需要展开数组，性能较差
   SELECT unnest(symptoms) as symptom_code, COUNT(*)
   FROM food_reactions
   WHERE user_id = ? AND realm_id = ?
   GROUP BY symptom_code;
   ```
2. ❌ **无法建立有效索引**：PostgreSQL 的 GIN 索引对数组有效，但不如独立表的 B-tree 索引高效
3. ❌ **扩展性差**：无法为单个症状添加额外属性（如严重程度、出现时间）
4. ❌ **数据完整性约束弱**：虽然可以用 CHECK 约束，但无法防止重复值（需要应用层处理）
5. ❌ **审计能力弱**：无法追踪「哪个症状是什么时候添加/删除的」

#### 4.3.3 推荐方案：独立表（当前设计）

**结论：推荐使用独立表方案**，理由如下：

1. **业务需求匹配**：
   - 统计页面（图四）需要「按症状统计」，独立表查询性能明显优于数组
   - 未来可能需要「症状与触发成分的关联分析」，独立表更容易扩展

2. **数据质量**：
   - 独立表通过约束保证数据完整性，减少应用层错误
   - 审计字段支持数据追溯

3. **性能考虑**：
   - 写入性能差异可忽略（反应记录创建频率低）
   - 读取性能：对于「列表查询」，可以通过 `array_agg()` 或应用层聚合优化
   - 统计查询：独立表明显优于数组

4. **代码质量**：
   - 独立表对应 Rust 中的独立实体，类型安全，代码更清晰
   - Sea-ORM 的关系映射更直观

**优化建议：**

如果担心 JOIN 查询的性能，可以考虑：

1. **使用 `array_agg()` 在数据库层聚合**：
   ```sql
   SELECT
       r.*,
       COALESCE(array_agg(s.symptom_code) FILTER (WHERE s.symptom_code IS NOT NULL), '{}') as symptoms
   FROM food_reactions r
   LEFT JOIN food_reaction_symptoms s ON r.id = s.reaction_id
   WHERE r.user_id = ? AND r.realm_id = ?
   GROUP BY r.id
   ORDER BY r.eaten_at DESC;
   ```

2. **在应用层缓存**：对于频繁查询的反应列表，可以在应用层缓存症状数组

3. **使用视图**：创建一个包含症状数组的视图，简化查询
