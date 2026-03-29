---
id: ISSUE-019
title: 框架缺乏独立测试与调试支持，仿真逻辑只能通过完整应用集成才能验证
type: dx
priority: p1-high
status: resolved
reporter: framework-consumer
created: 2026-03-29
updated: 2026-03-30
---

## 问题描述

duan 框架目前没有提供任何用于**独立测试或调试单个域/仿真场景**的工具或辅助设施。在实际项目开发中，这导致以下工程问题：

### 问题一：无法单独验证域逻辑

要验证 `CollisionRules` 的碰撞判断是否正确，目前唯一的方式是：

1. 启动完整的 Tauri 应用（带 Nuxt 前端）
2. 在前端触发仿真运行
3. 通过 UI 观察效果，或在日志中寻找蛛丝马迹

无法像普通 Rust 库那样写 `#[test]` 来验证：

```rust
#[test]
fn test_missile_hits_ship_within_range() {
    // 期望能构建一个最小场景，步进几帧，然后断言
    let mut world = World::builder()
        .with_domain("collision", CollisionRules::new())
        .build();
    
    let ship = world.spawn(/* ... */);
    let missile = world.spawn(/* ... */);
    
    world.step(1.0);
    
    // 断言 HitEvent 被发出？断言 ship 被销毁？
    // 目前没有任何机制可以在测试中读取事件或检查世界状态
}
```

### 问题二：无法在不启动完整应用的情况下观察仿真状态

调试域逻辑时，开发者需要知道：
- 某帧内哪些事件被发出了？
- 某个实体的组件当前是什么值？
- 域执行顺序是否符合预期？

目前这些信息只能通过 `println!` 插入到域代码中，或者依靠前端 UI 间接观察。框架没有提供任何事件追踪、状态快照或执行日志的能力。

### 问题三：示例项目与测试的边界模糊

`examples/` 中的示例本质上是"演示用的可运行程序"，而非"可复现的测试用例"。当示例代码的行为出现异常时，无法自动回归检测，只能手动运行后肉眼对比。

在 `taishixum-app` 中，我们的做法是：在 `runtime.rs` 里加 `log::info!` 输出，然后通过 Tauri 应用日志来逐帧追踪——这几乎是不得不走的弯路，核心原因是框架没有提供测试钩子。

## 影响程度

- [ ] 阻塞性
- [x] 中等（影响开发效率或理解，有变通方式）
- [ ] 轻微

> 注：在项目规模小时尚可承受，但随着域数量增加，缺乏测试支持会逐步从"中等"升级为实质性阻碍。

## 复现场景

任何需要验证域逻辑正确性的时刻都会遇到这个问题，例如：
- 调整 `CollisionRules` 的碰撞距离阈值后，想确认新阈值下的行为
- 修改 `MotionRules` 的导弹末段制导逻辑后，想单步观察轨迹
- 在 CI/CD 中对仿真逻辑进行回归测试

## 建议方案

**短期可改进**：

1. **提供 `World` 的事件读取接口**：`step_with` 回调中已能消费事件，但没有"将当前帧所有事件收集为列表"的接口。可以提供：

   ```rust
   // 执行一帧并返回发出的所有事件
   let events = world.step_collect(dt);
   for event in &events {
       if let Some(hit) = event.downcast::<HitEvent>() { /* ... */ }
   }
   ```

2. **提供实体/组件的便捷读取接口**：当前可以通过 `world.entities.get()` 读取组件，但 API 较为底层。提供测试友好的辅助方法：

   ```rust
   // 在测试中快速断言某组件值
   let pos = world.get_component::<Position>(entity_id)?;
   assert!((pos.x - expected_x).abs() < 0.01);
   ```

**需架构讨论**：

提供 `SimulationTestHarness` 或类似的测试辅助结构：

```rust
let mut harness = SimulationTestHarness::new()
    .with_domain("motion", MotionRules::new())
    .with_domain("collision", CollisionRules::new());

let ship = harness.spawn(/* ... */);
let missile = harness.spawn(/* ... */);

let frame = harness.step(1.0);

// 断言事件
assert!(frame.events::<HitEvent>().any(|e| e.target_id == ship));

// 断言组件状态
assert!(harness.is_destroyed(ship));
```

这不一定需要全新的类型，可以是对 `World` 的薄封装，专为测试场景设计的便利 API。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：部分采纳——API 补全（`step_collect`）和测试文档采纳；独立 `SimulationTestHarness` 类型不采纳（过度设计）

**分析**：

问题真实且严重。`philosophy.md` 的"设计收益"一节明确承诺"域可以独立测试，不需要搭建完整的仿真环境"。这不是说说而已——它是框架的设计目标之一，当前不可达等于设计承诺落空。

**当前实际情况**：

`World` 已经可以在 `#[test]` 中直接构建和使用——`World::builder().with_domain(...).build()`、`world.spawn()`、`world.step(dt)` 在测试环境中均可调用，不依赖任何 Tauri 或前端。reporter 描述的"唯一方式是启动完整应用"并不完全准确，但 reporter 的核心诉求有效：**缺少事件观察接口**，使得 `world.step(dt)` 执行后无法在测试代码中断言任何事件相关的行为。

`step_with` 已提供事件观察能力，但其回调形式不适合 `#[test]` 中的结构化断言——无法将事件收集为列表后再统一 assert。这是一个有价值的 API 缺口。

**不采纳 `SimulationTestHarness`**：

`World` 本身已具备构建测试场景所需的所有能力。引入 `SimulationTestHarness` 作为独立类型会产生 API 表面积膨胀和文档维护负担，且不增加新能力，只是 `World` 的薄封装。保持 `World` 单一入口更符合"如无必要，勿增实体"原则。

**行动计划**：

- [x] 为 `World` 增加 `step_collect(dt)` 方法，返回当帧产生的事件列表（`Vec<Arc<dyn CustomEvent>>`），供测试代码断言
  - 实现位置：`src/world.rs`，新增 `step_collect` 公开方法和 `drain_and_process_events_collect` 私有辅助方法
  - 同步新增 `test_step_collect_returns_custom_events` 单元测试，验证基本功能
- [x] 在 `guides/` 中新增"测试域逻辑"章节（`docs/duan-docs/guides/testing.md`），展示 `step_collect`、事件断言、组件状态读取和单步执行的完整示例
  - 同步更新 `docs/duan-docs/index.md`，将新指南纳入文档导航
- [ ] 可选：为 `World` 增加 `get_component::<T>(entity_id)` 便利方法，降低测试中的组件读取成本（延后，当前通过 `get_entity` → `get_component` 两步操作已足够）
