---
name: 框架 API 已知痛点
description: 在实际开发中遇到并确认的框架 API 问题，包括变通方案
type: reference
---

## step_with 闭包签名隐式 'static 问题（ISSUE-005，open）

**现象**：`event.downcast::<MyEvent>()` 在 `step_with` 闭包中编译报错 E0521。

**根本原因**：`step_with` 的闭包约束 `F: FnMut(&dyn CustomEvent, &mut Self)` 中，
`&dyn CustomEvent` 被 Rust 的 lifetime elision 规则展开为 `&(dyn CustomEvent + 'static)`，
要求事件对象满足 `'static`，而闭包内实际持有的是短生命周期引用。
修改 `downcast` 签名（ISSUE-002 的修复方向）不能解决此问题。

**变通方案**：
```rust
world.step_with(dt, |event, _world| {
    if let Some(c) = event.as_any().downcast_ref::<MyEvent>() {
        // ...
    }
});
```

**正确的修复方向**（ISSUE-005 建议）：
```rust
// 用 HRTB 消除隐式 'static
pub fn step_with<F>(&mut self, dt: f64, mut handler: F)
where
    F: for<'e> FnMut(&'e (dyn CustomEvent + 'e), &mut Self),
```

**历史**：先前被错误地标为 ISSUE-002 的签名问题，维护者修了 `downcast` 方法签名
但问题依然存在；已以 ISSUE-005 重新提出，指向真正的根本原因。
