---
id: ISSUE-007
title: 事件处理闭包中 _world 参数的能力边界未文档化
type: documentation
priority: p2-medium
status: resolved
reporter: framework-consumer
created: 2026-03-27
updated: 2026-03-27
---

## 问题描述

在规划"舰队对抗与导弹拦截"示例时，我需要在事件处理阶段**创建新实体**（舰船发射导弹后，战斗域发出"开火事件"，事件处理器在闭包中 spawn 导弹实体）。

`step_with` 的闭包签名是 `|event, world|`，其中 `world` 参数类型是 `&mut World`。但在整个文档体系中，我找不到任何对该参数能力边界的说明：

- `world.spawn()` 在闭包中是否支持？
- `world.register_domain()` 在闭包中是否支持（肯定不该，但没明确禁止）？
- 闭包内 `world.get_entity()` 是否会因为借用冲突而出错（闭包外也有 `world` 的借用）？

`event.md` 中提到事件处理器可以"创建新实体"，但那是对通用事件处理器的说明。`overview.md` 中的数据流图对 `step_with` 闭包的描述是"自定义事件"处理，两者没有明确地说 `step_with` 闭包中的 `world` 参数可以做哪些操作。

## 影响程度

- [x] 中等（影响开发效率或理解，有变通方式）

## 复现场景

在规划涉及"事件触发实体生成"的场景（典型：武器命中触发爆炸实体，战斗域发出开火事件触发导弹 spawn）时，我试图在 `step_with` 闭包中调用 `world.spawn()`，但无法从文档中确认这是否被支持，以及是否会有内部借用冲突。

具体场景：

```rust
world.step_with(dt, |event, world| {
    if let Some(fire) = event.as_any().downcast_ref::<FireEvent>() {
        // 能在这里创建导弹实体吗？
        let missile_id = world.spawn(
            Entity::new("missile")
                .with_domain("motion")
                .with_domain("collision")
                .with_component(Position::new(fire.launch_x, fire.launch_y, fire.launch_z))
                .with_component(Velocity::from_direction(fire.target_direction, fire.speed))
        );
    }
});
```

这是否是框架支持的合法用法，完全依赖对内部实现的猜测，文档没有给出答案。

## 建议方案

**短期可改进**：

在以下两处补充说明：

1. **`event.md` 的"注册与执行上下文"章节**：明确说明 `step_with` 闭包中的 `world` 参数支持的操作范围，至少列出：
   - 支持：`world.spawn()`、`world.get_entity()`、`world.sim_time()`
   - 不支持：`world.register_domain()`（初始化阶段之后不允许）
   - 警告：闭包内不应持有 `world` 跨帧引用（闭包生命周期约束）

2. **`architecture/overview.md` 的数据流部分**：在"事件处理阶段"的描述中添加一句，说明 `step_with` 闭包在此阶段可以执行实体创建操作。

3. **`guides/scenario.md` 或新增的 `guides/idioms.md` 章节**：提供"在事件处理中 spawn 新实体"的完整惯用法示例，尤其是导弹/爆炸物这类"由事件触发生成"的实体模式。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：采纳（部分）——问题成立，`world.spawn()` 确实可以在闭包中调用且不存在借用冲突，文档应明确说明。建议方案 1 采纳，建议方案 2 和 3 不采纳。

**分析**：

核查 `src/world.rs` 的 `process_events` 实现（第 339-352 行）：

```rust
let events = self.events.drain();  // ← 事件先全部取出
for event in events {
    if let DomainEvent::Custom(event_arc) = &event {
        handler(event_arc.as_ref(), self);  // ← 此时 self 无其他活跃借用
    }
    ...
}
```

`self.events.drain()` 已将事件完整取出，之后 `self` 再无来自事件通道的活跃借用。传入闭包的 `&mut World` 是完整的可变引用，`world.spawn()`、`world.get_entity()`、`world.get_entity_mut()` 均不存在借用冲突，是合法调用。

`world.register_domain()` 技术上调用不会 panic，但该操作语义属于初始化阶段，仿真循环开始后调用会破坏域执行顺序的一致性（执行顺序在 `build()` 时已固化），框架文档应明确禁止。

关于建议方案的取舍：

- 方案 1（`event.md` 的"注册与执行上下文"章节）：采纳。ISSUE-004 已确认事件处理器不是注册式的，该章节描述的就是 `step_with` 闭包，是补充能力边界说明的最自然位置。
- 方案 2（`overview.md` 数据流部分）：不采纳。数据流图用于说明阶段流转，不是列举 API 能力的合适场所；在图旁加操作列表会破坏该文档的语义密度。
- 方案 3（新增 `guides/idioms.md`）：不采纳。新增文件违背"如无必要勿增实体"原则，"事件触发实体生成"是正常的事件处理器用法，无需单独的惯用法文件。

**行动计划**：

已修改 `docs/duan-docs/concepts/event.md` 的"注册与执行上下文"章节：
- 将抽象的"注册式处理器"表述更正为准确的 `step_with` 闭包调用方式
- 明确列出闭包中支持调用的操作（`spawn`、`destroy`、`get_entity`、`sim_time`）
- 明确标注不应调用的操作（`register_domain`）及其原因（初始化后行为未定义）

**关闭理由**（如拒绝或 wontfix）：不适用，问题已修复。
