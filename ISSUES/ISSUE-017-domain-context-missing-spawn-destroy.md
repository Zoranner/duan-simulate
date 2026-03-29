---
id: ISSUE-017
title: DomainContext 缺少 spawn/destroy 能力，域无法自主完成实体生命周期管理
type: architecture
priority: p1-high
status: open
reporter: framework-consumer
created: 2026-03-29
updated: 2026-03-29
---

## 问题描述

`DomainContext` 目前不提供任何创建或销毁实体的接口。这意味着，即使域在 `compute()` 中做出了"应当生成导弹"或"应当销毁残骸"的决策，它也无法直接执行——必须通过发出事件，再由应用层的 `step_with` 回调来处理。

这直接违反了框架的"域即权威（Domain-as-Authority）"设计哲学：**域拥有做出决策的全部信息，却没有执行决策的权限。** 实体的创建责任被迫泄漏到框架外部的业务代码中。

以 `taishixum-app` 中的舰战仿真为例：

- `CombatRules::compute()` 检测到目标、完成冷却判断，发出 `FireEvent`
- `InterceptRules::compute()` 确定了要拦截的导弹，发出 `InterceptFireEvent`
- 真正的 `spawn_missile()` / `spawn_interceptor()` 调用发生在 `runtime.rs` 的 `step_with` 回调里

这意味着导弹生成逻辑（包括初速度、初始姿态、组件配置）散落在应用层，而不是封装在域内。任何需要二次开发的人都必须同时理解域层和 `runtime.rs` 才能追踪完整的"开火"逻辑，认知成本大幅提升。

## 影响程度

- [x] 阻塞性（无法继续开发，或导致概念根本性混乱）

> 注：虽有变通方案（step_with 中 spawn），但这是一个**架构性**阻塞——它使"域即权威"在实体生命周期管理上根本无法成立。

## 复现场景

在 `CombatRules::compute()` 中，当确定需要发射导弹时，期望能写出如下代码：

```rust
fn compute(&mut self, ctx: &mut DomainContext) {
    for shooter_id in ctx.own_entity_ids() {
        // ...检测目标、判断冷却...
        if should_fire {
            // 期望：域直接生成导弹实体
            let missile_id = ctx.spawn(
                Entity::new("missile")
                    .with_domain("motion")
                    .with_domain("collision")
                    .with_component(MissileKind::AntiShip)
                    .with_component(Position::from(launch_pos))
                    .with_component(MissileState::new(target_id, shooter_id, ...))
            );
        }
    }
}
```

但 `DomainContext` 上没有 `spawn()` 方法，上述代码无法编译，只能改为发出 `FireEvent` 并在框架外处理。

## 建议方案

**需架构讨论**：

在 `DomainContext` 上提供受控的实体创建/销毁接口：

```rust
impl DomainContext {
    /// 在当前帧末（下帧生效）将新实体加入世界
    pub fn spawn(&mut self, entity: Entity) -> EntityId;

    /// 标记实体为待销毁（与 World::mark_destroy 语义一致）
    pub fn destroy(&mut self, id: EntityId);
}
```

生效时序应与 `World::spawn()` 一致（下帧才进入域计算），无需为了支持域 spawn 而破坏帧内一致性。

**设计约束建议**：

如果担心域内 spawn 导致循环依赖或状态不一致，可以考虑：
- `ctx.spawn()` 仅将实体放入"待创建队列"，帧末统一提交，与当前 `step_with` 中 spawn 的时序完全相同
- 从域视角看，这是"提交了一个创建意图"，而不是"立即创建"，语义清晰

**短期可改进**：

在文档中明确说明当前的架构限制（域无法 spawn 实体），并将"通过事件 + step_with 回调创建实体"作为框架推荐的变通模式显式记录，避免开发者无谓地寻找不存在的 API。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：

**分析**：

**行动计划**：

**关闭理由**（如拒绝或 wontfix）：
