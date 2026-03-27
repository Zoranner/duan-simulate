---
name: 框架 API 已知痛点
description: 在实际开发中遇到并确认的框架 API 问题，包括变通方案
type: reference
---

## CustomEvent::downcast 生命周期问题（ISSUE-002，open）

**现象**：`event.downcast::<MyEvent>()` 在 `step_with` 闭包中编译报错 E0521，
Rust 对 `impl dyn CustomEvent` 推断出 `'static` 约束，与闭包内短生命周期的 `event` 冲突。

**变通方案**：
```rust
world.step_with(dt, |event, _world| {
    if let Some(c) = event.as_any().downcast_ref::<MyEvent>() {
        // ...
    }
});
```

**已提 Issue**：ISSUE-002，建议修改 `downcast` 的生命周期签名或更新文档示例。
