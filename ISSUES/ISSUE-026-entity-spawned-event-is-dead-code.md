---
id: ISSUE-026
title: DomainEvent::EntitySpawned 变体是死代码——框架从不 emit，handle_event 空操作
type: architecture
priority: p2-medium
status: open
reporter: architecture-auditor
created: 2026-03-30
updated: 2026-03-30
---

## 问题描述

`DomainEvent` 枚举包含以下变体：

```rust
pub enum DomainEvent {
    EntitySpawned { entity_id: EntityId, entity_type: String },  // ← 死代码
    EntityDestroyed { entity_id: EntityId, cause: DestroyCause },
    Timer { entity_id: EntityId, timer_id: String, callback: TimerCallback },
    Custom(Arc<dyn CustomEvent>),
}
```

`EntitySpawned` 存在于公开 API 中，但：

1. **框架内部的 `World::spawn()` 从不 emit 此事件**——实体在 `spawn()` 中直接进入 `Active` 状态，整个过程无事件产生
2. **`handle_event()` 对 `EntitySpawned` 是空操作**——即使用户手动 emit 此变体，框架也不做任何处理
3. **文档中无任何说明**——用户无法从 API 或文档中得知这个变体永远不会被框架触发

## 架构层面的问题

这是一个**具有欺骗性的公开 API**。

`DomainEvent::EntitySpawned` 与 `DomainEvent::EntityDestroyed` 并排出现，视觉上完全对称，强烈暗示"你可以监听实体的创建事件"。但实际上，监听 `EntitySpawned` 的代码永远不会被触发——不是因为条件不满足，而是因为框架根本不会 emit 它。

这违背了公开枚举变体的基本契约：如果一个变体存在于公开 API 中，它应当在某种合法的场景下有意义。

更深层的问题：这个变体的存在暗示着框架原本计划实现"spawn 时 emit 事件"的功能，但该功能从未完成，而死代码被保留下来，变成了陷阱。

## 影响程度

- [ ] 阻塞性
- [x] 中等（影响开发效率或理解，有变通方式）

> 注：用户在尝试处理实体生命周期时，可能会花时间编写对 `EntitySpawned` 的处理逻辑，然后困惑地发现它从不触发。这类"代码正确但永不执行"的错误极难调试。

## 建议方向

**方向一（推荐）：删除 `EntitySpawned` 变体**

如果框架设计上"spawn 时不产生事件"（理由是 spawn 是同步操作，不需要跨边界通知），则应该删除此变体，使公开 API 诚实反映实际行为。

**方向二：真正实现 EntitySpawned 事件**

如果保留此变体的初衷是"spawn 时通知监听者"，则应在 `World::spawn()` 中实际 emit 此事件。但这需要考虑时序问题：spawn 发生在事件处理阶段之外，需要设计明确的通知机制。

这两个方向之间没有中间路线——死代码既不应该保留，也不应该仅加文档说明"此变体不被触发"。
