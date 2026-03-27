---
id: ISSUE-005
title: ISSUE-002 修复不完整——step_with 闭包签名的隐式 'static 是 E0521 的真正根源
type: api-design
priority: p2-medium
status: resolved
reporter: framework-consumer
created: 2026-03-27
updated: 2026-03-27
---

## 问题描述

ISSUE-002 已被维护者标为 `resolved`（修改了 `downcast` 方法的生命周期签名），
但在实际开发 `free_fall` 示例时，`event.downcast::<GroundCollisionEvent>()`
在 `step_with` 闭包中**仍然触发 E0521**，修复未生效。

### 复现验证

将 `examples/free_fall/src/main.rs` 中的事件处理闭包改为使用 `downcast`：

```rust
world.step_with(SIM_DT, |event, _world| {
    if let Some(c) = event.downcast::<GroundCollisionEvent>() {
        // ...
    }
});
```

执行 `cargo build`，仍然报：

```
error[E0521]: borrowed data escapes outside of closure
  --> src\main.rs:64:30
   |
63 |         world.step_with(SIM_DT, |event, _world| {
   |                                  -----
   |                                  `event` is a reference that is only valid in the closure body
   |                                  has type `&'1 (dyn CustomEvent + '1)`
64 |             if let Some(c) = event.downcast::<GroundCollisionEvent>() {
   |                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |                              `event` escapes the closure body here
   |                              argument requires that `'1` must outlive `'static`
```

### 根本原因

ISSUE-002 的分析只关注了 `downcast` 方法本身，但 E0521 的真正根源在于
**`World::step_with` 的闭包约束签名**（`src/world.rs`，第 261–265 行）：

```rust
pub fn step_with<F>(&mut self, dt: f64, mut handler: F)
where
    F: FnMut(&dyn CustomEvent, &mut Self),
```

`F: FnMut(&dyn CustomEvent, &mut Self)` 中的 `&dyn CustomEvent` 按 Rust 的
隐式 lifetime elision 规则展开为 `&(dyn CustomEvent + 'static)`，
这要求闭包参数 `event` 引用的对象本身满足 `'static`。

而闭包实际收到的 `event` 是短生命周期 `&'1 (dyn CustomEvent + '1)`，
`'1` 是调用栈上的临时借用，不满足 `'static`。

此时调用 `impl dyn CustomEvent` 上的 `downcast`：

```rust
// impl dyn CustomEvent 被解析为 impl (dyn CustomEvent + 'static)
pub fn downcast<'a, T: 'static>(&'a self) -> Option<&'a T>
```

`self` 的类型 `dyn CustomEvent + '1` 必须满足 `dyn CustomEvent + 'static`，
`'1: 'static` 不成立，编译器报 E0521。

**结论**：修改 `downcast` 的签名不能解决这个问题，因为错误在调用侧的类型约束。

## 影响程度

- [x] 中等（影响开发效率或理解，有变通方式）

## 复现场景

开发 `free_fall` 示例，在 `step_with` 闭包中尝试使用框架提供的 `event.downcast`
便捷方法，立即触发 E0521。退回到 `event.as_any().downcast_ref::<T>()` 可正常编译。

## 建议方案

**修改 `step_with`、`do_step`、`process_events` 的闭包约束**，使用高阶 trait 约束（HRTB）
消除隐式 `'static` 要求：

```rust
// 修改前（src/world.rs）
pub fn step_with<F>(&mut self, dt: f64, mut handler: F)
where
    F: FnMut(&dyn CustomEvent, &mut Self),

// 修改后
pub fn step_with<F>(&mut self, dt: f64, mut handler: F)
where
    F: for<'e> FnMut(&'e (dyn CustomEvent + 'e), &mut Self),
```

`for<'e>` 告诉 Rust：闭包可以接受**任意**生命周期的 `dyn CustomEvent` 引用，
不限于 `'static`。同步修改 `do_step` 和 `process_events` 的 `F` 约束即可。

修复后，`event.downcast::<T>()` 将在 `step_with` 闭包中正常工作，无需再使用
`event.as_any().downcast_ref::<T>()` 绕行。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：采纳根因诊断，但建议方案需要修正——修复目标是使 `step_with` 约束明确传入 `'static` object lifetime，而非使用 HRTB 扩大可接受 lifetime 范围。

**分析**：

**ISSUE-002 修复的局限性确认**：

ISSUE-002 修复了 `downcast` 方法的返回值生命周期（`(&'a self) -> Option<&'a T>`），解决了返回引用的生命周期绑定问题。但 `impl dyn CustomEvent` 块在 Rust 中隐式等价于 `impl (dyn CustomEvent + 'static)`，方法 `self` 的 object lifetime 仍然要求满足 `'static`。这是独立于返回值生命周期的约束，ISSUE-002 的修复没有触及。

**E0521 的完整根因链**：

1. `step_with` 约束 `F: FnMut(&dyn CustomEvent, &mut Self)` 中，`&dyn CustomEvent` 按 Rust 的 lifetime elision 规则（RFC 599），在 HRTB 展开后等价于 `for<'r> FnMut(&'r (dyn CustomEvent + 'r), ...)`。
2. 闭包参数 `event` 的类型在闭包体内被视为 `&'r (dyn CustomEvent + 'r)` for some unknown `'r`，object lifetime 为 `'r`，不保证满足 `'static`。
3. `impl dyn CustomEvent`（即 `impl (dyn CustomEvent + 'static)`）上的 `downcast` 方法要求 `self` 的 object lifetime 满足 `'static`，即 `'r: 'static`，无法满足，编译器报 E0521。

ISSUE-005 reporter 的根因诊断正确：问题根源在 `step_with` 的约束表达方式，而非 `downcast` 方法本身。

**建议方案的修正**：

Reporter 的建议（改为 `for<'e> FnMut(&'e (dyn CustomEvent + 'e), ...)`）并不能解决问题。HRTB 只是改变了 Rust 展开 lifetime 的方式，闭包参数 `event` 的 object lifetime 仍然是某个不确定的 `'e`，仍然无法调用 `impl (dyn CustomEvent + 'static)` 上的方法。

正确的修复方向：让闭包参数的 object lifetime 明确为 `'static`，反映 `process_events` 中实际传入的事件类型（`event_arc.as_ref()` 返回 `&(dyn CustomEvent + 'static)`，因为 `DomainEvent::custom<E: CustomEvent + 'static>` 存储的是 `'static` object）。

有两种等价的写法：

```rust
// 写法一：明确 object lifetime 'static
pub fn step_with<F>(&mut self, dt: f64, mut handler: F)
where
    F: FnMut(&(dyn CustomEvent + 'static), &mut Self),

// 写法二：利用默认 object lifetime（在 trait bound 上下文中默认 'static）
// 当前写法 F: FnMut(&dyn CustomEvent, &mut Self) 在 trait bound 上下文
// 已经是 'static，但实际效果有编译器行为差异，建议使用写法一保持明确
```

`do_step` 和 `process_events` 的 `F` 约束需同步修改。修改后闭包参数 `event: &(dyn CustomEvent + 'static)` 可以正常调用 `impl (dyn CustomEvent + 'static)` 上的 `downcast` 方法。

注意：此修复是语义明确化，不改变实际行为——`process_events` 目前就是传入 `'static` object，只是约束表达得不够精确。

**行动计划**：

1. 将 `src/world.rs` 中 `step_with`、`do_step`、`process_events` 的闭包约束从 `F: FnMut(&dyn CustomEvent, &mut Self)` 改为 `F: FnMut(&(dyn CustomEvent + 'static), &mut Self)`。
2. 更新 `examples/free_fall/src/main.rs` 验证 `event.downcast::<GroundCollisionEvent>()` 可正常编译。
3. 执行 `cargo build` 验证。
4. ISSUE-002 在 decisions.md 中标注的修复说明需补充：根因比原分析更深，完整修复需同时调整 `step_with` 约束。

**架构哲学一致性**：已自验证
