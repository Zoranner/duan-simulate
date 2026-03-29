---
id: ISSUE-020
title: on_attach/on_detach 生命周期钩子缺少使用场景指导，导致开发者绕过或误用
type: documentation
priority: p3-low
status: open
reporter: framework-consumer
created: 2026-03-29
updated: 2026-03-29
---

## 问题描述

`DomainRules` trait 提供了两个生命周期钩子：

```rust
fn on_attach(&mut self, entity_id: EntityId, entity: &Entity) {}
fn on_detach(&mut self, entity_id: EntityId) {}
```

然而在整个 `taishixum-app` 项目中（包含 7 个域：`MotionRules`、`DetectionRules`、`CombatRules`、`InterceptRules`、`CollisionRules`、`TrackingRules`、`FactionRules`、`SpaceRules`），**没有任何一个域使用了这两个钩子**。

这不是因为没有合适的场景，而是因为：
1. 文档中没有说明这两个钩子的**典型使用场景**是什么
2. 没有说明与 `try_attach` 的职责边界：`try_attach` 决定是否挂载，`on_attach` 在挂载后做什么？
3. 没有说明在 `on_attach` 中可以（和不可以）访问哪些上下文

在实际开发 `TrackingRules` 时，我需要为每个被挂载的敌方实体初始化一个空的历史轨迹缓存。直觉上这应该在 `on_attach` 中完成，但由于不清楚 `on_attach` 时是否能访问实体的组件数据、是否能安全地修改组件，最终选择了在每帧 `compute()` 里做"懒初始化"检查，绕过了 `on_attach`。

这是一个典型的"钩子存在但不敢用"的情况，根本原因是文档缺失导致的信心不足。

## 影响程度

- [ ] 阻塞性
- [ ] 中等
- [x] 轻微（体验欠佳，但不影响核心功能）

## 复现场景

在 `TrackingRules` 中，期望在实体首次挂载时初始化历史轨迹容器：

```rust
impl DomainRules for TrackingRules {
    fn on_attach(&mut self, entity_id: EntityId, entity: &Entity) {
        // 期望在这里为新实体初始化追踪状态
        // 但不确定：
        // 1. entity 参数包含组件数据吗？（还是只是 ID 信息）
        // 2. 能在这里修改 entity 的组件吗？
        // 3. on_attach 调用时，实体是否已经是 Active 状态？
        self.track_cache.insert(entity_id, TrackHistory::new());
    }

    fn compute(&mut self, ctx: &mut DomainContext) {
        // 因为不确定 on_attach 的语义，实际上用了如下模式：
        for entity_id in ctx.own_entity_ids() {
            self.track_cache.entry(entity_id).or_insert_with(TrackHistory::new);
            // ...
        }
    }
}
```

对 `on_attach` 语义的不确定，使得开发者倾向于用 `compute()` 中的懒初始化来规避风险，而不是使用设计好的钩子。

## 建议方案

**短期可改进**（文档层面）：

在 `guides/custom-domain.md` 或 `concepts/domain.md` 中补充 `on_attach`/`on_detach` 的说明，明确以下几点：

1. **调用时机**：`on_attach` 何时被调用（`try_attach` 返回 `true` 后的哪个阶段）？
2. **参数内容**：`entity: &Entity` 参数包含实体的完整组件数据吗？是否已经是 `Active` 状态？
3. **可以做什么**：典型的合法操作是什么（如：初始化域内缓存、记录实体 ID、读取组件初始值）？
4. **不能做什么**：有哪些操作不能在 `on_attach` 中执行（如：修改其他实体、发出事件）？
5. **典型示例**：至少提供一个使用 `on_attach` 的完整域实现示例

**建议的文档示例**：

```rust
// 示例：使用 on_attach 初始化域内每实体缓存
impl DomainRules for TrackingRules {
    fn on_attach(&mut self, entity_id: EntityId, entity: &Entity) {
        // 读取组件初始值来初始化追踪状态
        if let Some(pos) = entity.get_component::<Position>() {
            self.histories.insert(entity_id, vec![(pos.x, pos.y)]);
        } else {
            self.histories.insert(entity_id, vec![]);
        }
    }

    fn on_detach(&mut self, entity_id: EntityId) {
        // 清理域内与该实体相关的缓存，避免内存泄漏
        self.histories.remove(&entity_id);
    }
}
```

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：

**分析**：

**行动计划**：

**关闭理由**（如拒绝或 wontfix）：
