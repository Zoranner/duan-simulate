---
id: ISSUE-010
title: 事件处理器中 spawn 的实体无法在同帧内被域处理——缺少文档说明与设计指导
type: concept-clarity
priority: p1-high
status: resolved
reporter: framework-consumer
created: 2026-03-27
updated: 2026-03-29
---

## 问题描述

在规划"舰队对抗与导弹拦截"示例时，我遇到了一个关键的时序问题：

`step_with` 闭包在事件处理阶段执行，而域的 `compute` 在计算阶段执行。根据仿真循环的设计，事件处理阶段在计算阶段**之后**。

这意味着：在事件处理器中通过 `world.spawn()` 创建的导弹实体，**只有在下一帧的计算阶段才会被运动域、探测域等接管并开始计算**。

这本身可能是预期行为，但文档中完全没有说明这个时序差（即"spawn 的实体何时开始参与仿真循环"），给以下场景带来了设计困惑：

**场景 1：导弹的初始位置**

战斗域的"开火事件"包含发射位置。在闭包中 `spawn` 导弹时，导弹的 `Position` 应该设为当前帧的发射位置。但如果下一帧才被运动域处理，这段时间差（一个 `dt`）导弹不会移动——对于时间步 `dt = 0.1s`、速度 `2000m/s` 的反舰导弹，这意味着第一帧它停在发射点，第二帧才开始飞行。这是否是期望的行为？

**场景 2：连锁反应**

如果导弹在本帧 spawn 但本帧不移动，那么探测域在本帧的结算中是否能"看到"这枚新导弹？根据 `ctx.entities.active_entities()` 的语义，若该实体的 `Lifecycle` 状态在 spawn 后立即为 `Active`，则探测域在**同帧后续计算**中就能看到它，但该帧运动域已经执行过了，导弹不会移动。

这涉及到：
1. `world.spawn()` 后实体的 `Lifecycle` 状态是什么？立即 `Active` 还是要等下一帧？
2. 若立即 `Active`，当帧内后续域是否能读到该实体？（取决于事件处理阶段和计算阶段的相对位置）
3. 从框架设计角度，"当帧 spawn、下帧开始运动"是刻意的设计（避免同帧计算出现竞争条件）吗？

## 影响程度

- [x] 中等（影响开发效率或理解，有变通方式）

## 复现场景

规划导弹发射逻辑时，需要在战斗域的"开火事件"处理器中创建导弹实体：

```rust
world.step_with(dt, |event, world| {
    if let Some(fire_event) = event.downcast::<FireEvent>() {
        // 问题：这枚导弹什么时候开始被运动域处理？
        // 是本帧剩余的域计算？还是下一帧才开始？
        let _missile_id = world.spawn(
            Entity::new("missile")
                .with_domain("motion")
                .with_domain("interception")
                .with_component(Position::new(
                    fire_event.launch_x,
                    fire_event.launch_y,
                    fire_event.launch_z,
                ))
                .with_component(Velocity::new(
                    fire_event.vel_x,
                    fire_event.vel_y,
                    fire_event.vel_z,
                ))
                .with_component(Missile::new(
                    fire_event.target_id,
                    fire_event.warhead_yield,
                )),
        );
        // 此导弹在本帧内静止，下一帧才开始运动——这正确吗？
    }
});
```

## 建议方案

**短期可改进**：

在 `concepts/event.md` 的"处理器的职责"或"处理器产生的新事件"章节，明确说明：

1. 事件处理器中 `world.spawn()` 的实体在**下一帧**才会进入域的计算流程（即：当帧 spawn 的实体不参与当帧计算）
2. 这是有意的设计：所有在一个 tick 的事件处理阶段产生的实体，从下一个 tick 的计算阶段开始生效，保证同帧计算的一致性
3. 建议在设计事件数据时将"实体的完整初始状态"包含在事件中（而非依赖事件处理器之后去查询），使得接收方即使晚一帧也不会丢失信息

**需架构讨论**：

若框架希望支持"事件处理器 spawn 的实体立即参与当帧后续计算"，需要调整仿真循环架构（将事件处理插入计算阶段中间），这可能引入更多复杂性。当前设计（事件处理在计算后）的一致性更强，应在文档中明确背书。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：文档补充，已解决

**分析**：

reporter 的理解完全正确。事件处理阶段（阶段四）在域计算阶段（阶段二）之后，`world.spawn()` 虽然立即完成域归属和 `Active` 状态设置，但当帧的域 `compute` 已全部执行完毕，因此当帧 spawn 的实体只在**下一帧**的计算阶段才被域首次处理。这是有意设计：保证同帧计算基于一致的实体快照。

reporter 建议的"支持事件处理器 spawn 的实体立即参与当帧后续计算"需要将事件处理插入计算阶段中间，会破坏帧内一致性，与设计哲学相悖，不采纳。

**行动计划**：

- [x] 在 `concepts/event.md` 的"处理器"章节补充"实体生效时序"说明，明确当帧 spawn、下帧开始计算的行为，以及事件数据应自包含的建议

**解决说明**：文档已补充，设计行为有意且正确，无需变更框架。
