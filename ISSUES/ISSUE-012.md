---
id: ISSUE-012
title: EntityStore 缺少 is_destroying() 查询——域无法感知实体销毁状态
type: api-design
priority: p2-medium
status: resolved
reporter: framework-consumer
created: 2026-03-27
updated: 2026-03-27
---

## 问题描述

框架提供了 `world.destroy(id, duration)` 来触发实体销毁，实体在过渡期内仍处于 `Active` 状态，可被域的 `compute` 读取到。

但 `EntityStore`（以及 `DomainContext::entities`）目前没有提供查询某个实体是否"正在销毁过渡中"的接口：

```rust
// 期望有这样的方法，但目前不存在：
ctx.entities.is_destroying(entity_id) -> bool
```

这导致在域的 `compute` 中无法区分"活跃舰船"和"已触发销毁但仍在过渡期的舰船"，在事件驱动逻辑中造成困扰。

## 影响程度

- [ ] 阻塞性
- [x] 中等（影响开发效率或理解，有变通方式）
- [ ] 轻微

## 复现场景

`CombatRules::compute` 中需要检查舰船 HP 归零时发出 `ShipDestroyedEvent`。但如果一艘船在同帧内被多枚导弹命中，`HitEvent` 的处理会多次扣血，导致 HP 多次归零。问题在于：

1. 第一枚导弹命中 → `HitEvent` → `world.destroy(ship_id, 0.5)` 已被调用
2. 同帧第二枚导弹命中 → `HitEvent` 再次触发 → 试图再次发出 `ShipDestroyedEvent`

由于没有 `is_destroying()` 接口，我不得不在 `CombatRules` 内部维护一个 `destroying: HashSet<EntityId>` 来手动去重：

```rust
pub struct CombatRules {
    fire_cooldowns: HashMap<EntityId, f64>,
    destroying: HashSet<EntityId>,   // 手动跟踪，本该框架提供
}
```

这是本应由框架提供的状态被迫推给了域实现。

## 建议方案

**短期可改进**：

在 `EntityStore` 上增加 `is_destroying(id: EntityId) -> bool` 方法，通过检查实体的 `Lifecycle` 状态（`Destroying` 变体）来实现。调用方可写：

```rust
if !ctx.entities.is_destroying(ship_id) {
    ctx.emit(DomainEvent::custom(ShipDestroyedEvent { ship_id }));
    // world.destroy 由 step_with 闭包调用
}
```

**需架构讨论**：

更彻底的方案是让 `world.destroy()` 在调用后立即将实体标记为某种"锁定"状态，使得后续的 `HitEvent` 处理自动跳过已标记销毁的目标。但这涉及事件处理器和框架状态管理的交互，需要更多讨论。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：采纳。`is_destroying()` 是对已有 `Lifecycle::Destroying` 状态的简单查询包装，实现成本低，消除了域实现中的框架状态泄漏反模式。

**分析**：

问题成立。`Lifecycle::Destroying` 状态在框架内已存在（`world.destroy()` 触发），但 `EntityStore` 未暴露对应查询接口，迫使域实现者自行维护 `HashSet<EntityId>` 跟踪销毁状态——这是框架本应承担的状态被推给了域实现。

`is_destroying()` 只是对 `entity.lifecycle` 字段的只读检查，完全不涉及架构变更。`Lifecycle` 枚举的 `Destroying` 变体语义已稳定（`Destroying` = 已调用 `world.destroy()`、实体已从所有域脱离、处于过渡期中），实现一致性无疑问。

Reporter 提出的"更彻底方案"（`world.destroy()` 后自动过滤后续事件）不采纳：框架不应替用户过滤事件，事件处理器是用户代码，判断是否跳过是用户的职责，框架提供状态查询能力即可。

**行动计划**：

- [x] 在 `src/entity.rs` 的 `EntityStore` 上新增 `is_destroying(id: EntityId) -> bool` 方法
- [x] 在 `concepts/lifecycle.md` 的"销毁中（Destroying）"状态描述中补充：域可通过 `ctx.entities.is_destroying(id)` 感知此状态，常见用途是避免对同一实体重复发出销毁事件

**关闭理由**（如拒绝或 wontfix）：
