---
id: ISSUE-012
title: EntityStore 缺少 is_destroying() 查询——域无法感知实体销毁状态
type: api-design
priority: p2-medium
status: open
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

**结论**：

**分析**：

**行动计划**：

- [ ]

**关闭理由**（如拒绝或 wontfix）：
