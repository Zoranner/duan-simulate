---
id: ISSUE-027
title: on_attach 有默认实现而 on_detach 无，不对称设计缺乏架构依据
type: architecture
priority: p2-medium
status: open
reporter: architecture-auditor
created: 2026-03-30
updated: 2026-03-30
---

## 问题描述

`DomainRules` trait 对两个生命周期钩子的处理方式不对称：

```rust
pub trait DomainRules: Send + Sync + 'static {
    fn on_attach(&mut self, entity: &Entity) { let _ = entity; }  // 有默认空实现
    fn on_detach(&mut self, entity_id: EntityId);                  // 无默认实现，必须实现
}
```

- `on_attach`：可选钩子，不实现也能编译
- `on_detach`：强制钩子，必须实现，否则编译失败

## 架构层面的问题

### 不对称性缺乏清晰的架构依据

直觉上，"附加"和"分离"是对称操作，两者要么都是必须处理的，要么都是可选的。当前设计强制实现 `on_detach` 但允许忽略 `on_attach`，这背后需要有清晰的架构理由。

可能的理由是："实体被分离时，域必须清理对该实体的引用，否则会悬空引用导致逻辑错误；而附加时的初始化是可选的"。这个理由有一定合理性——但它在任何文档中均未说明，开发者只能通过"编译报错"来发现这个约束，而不是通过理解。

### 对"大多数域"造成了不必要的实现负担

对于简单的、无内部状态的域（例如纯规则域，不维护任何实体缓存），`on_detach` 的正确实现就是空函数体 `{}`。这种域被迫实现一个无意义的方法，只是为了通过编译。

这违背了"如无必要，勿增实体"的原则——如果"不实现 on_detach 等于空实现"的语义是安全的（如同 on_attach 的设计），那么强制实现它的意义仅限于"提醒开发者考虑清理逻辑"，而这种提醒更适合通过文档或 Clippy lint 实现，而非 trait 强制。

### 对称性破坏了对 API 的直觉预期

用户初次实现 `DomainRules` 时，看到 `on_attach` 有默认实现，会合理推断 `on_detach` 也有——这个推断会导致编译失败，造成混乱。

## 影响程度

- [ ] 阻塞性
- [ ] 中等
- [x] 轻微（主要是概念混乱，不影响正确实现）

## 建议方向

**方向一（推荐）：统一为可选，两者都提供默认空实现**

```rust
fn on_attach(&mut self, entity: &Entity) { let _ = entity; }
fn on_detach(&mut self, entity_id: EntityId) { let _ = entity_id; }
```

如果担心开发者忘记清理 `on_detach`，应通过文档说明而非 trait 强制——框架已有大量"依赖开发者自律"的设计（如"域只修改自己管辖的实体"），on_detach 的强制不构成特殊待遇。

**方向二：统一为必须，两者都去掉默认实现**

如果框架认为两个钩子都需要域明确声明处理意图（即使是空实现），则应对称地去掉 `on_attach` 的默认实现。这强化了"每个生命周期钩子都是有意识的决定"的语义，但会增加实现负担。

**无论选择哪个方向**，当前不对称设计的架构理由都应在文档中明确说明。
