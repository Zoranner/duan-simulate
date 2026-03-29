---
id: ISSUE-017
title: DomainContext 缺少 spawn/destroy 能力，域无法自主完成实体生命周期管理
type: architecture
priority: p1-high
status: resolved
reporter: framework-consumer
created: 2026-03-29
updated: 2026-03-30
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

**结论**：部分采纳——问题成立，文档短期补充；`ctx.spawn()` API 方向有争议，暂不纳入实现，等待架构讨论

**分析**：

reporter 描述的"决策在域内、执行在应用层"现象是真实存在的。框架的 `DomainContext` 当前确实不提供任何生命周期操作接口，这使得"域是权威"在实体创建层面无法自洽——战斗域知道应当发射导弹的全部信息，却必须借道 `FireEvent` + 应用层 `step_with` 回调才能完成实体创建。这是一个有代价的设计取舍，而非疏忽。

**架构约束的依据（来自文档）**：

`simulation-loop.md` 的"计算阶段的写入边界"一节明确规定：
> "域不能直接发起生命周期操作（创建/销毁实体）。这确保：生命周期操作集中在事件处理阶段，生命周期变更有迹可查；未来并行化时，依赖关系图即为安全的并发边界。"

这两条理由有实质价值：
1. **可追溯性**：当前所有实体创建都对应一个可见事件（`FireEvent`），可以被监听、记录和回放。若允许 `ctx.spawn()`，事件通道中不再有对应记录，仿真行为难以追溯。
2. **并行化安全**：compute() 阶段未来可按依赖关系图并行化，若域可以直接 spawn 实体，需要额外的同步机制。

**reporter 建议的缓冲队列方案的局限**：

reporter 提出将 `ctx.spawn()` 实现为帧末统一提交的缓冲队列，认为可与当前 `World::spawn()` 时序保持一致。但这种方案存在一个结构性问题：**spawn 行为脱离了事件通道，成为无法被外部观察者感知的隐式世界状态变更**。这与框架"所有跨边界影响均通过事件记录"的可追溯性原则不符。

**更合适的方向**：

若要保持域对"创建什么实体"的完整控制权，同时维护可追溯性，可以考虑引入"实体创建命令事件"模式：域在 compute() 中发出携带完整初始化数据的 `SpawnCommand` 事件，框架在事件处理阶段统一执行创建，并在事件通道中留有记录。这是一个值得讨论的架构方向，但设计细节需要专门评审。

**行动计划**：

- [x] 在 `guides/custom-domain.md` 的"域的写入边界"小节补充：详细说明 `compute()` 阶段不能 spawn/destroy 的两条核心理由（可追溯性 + 并行化安全），并提供"事件 + `step_with` 回调"完整代码示例
- [ ] 将"SpawnCommand 事件模式"作为架构讨论议题，评估是否值得专门支持，保留路线图入口

**关闭理由**（如拒绝或 wontfix）：不适用——架构约束有意维持，短期文档补充采纳，长期 API 方向待讨论。

**架构哲学一致性**：建议 architecture-auditor 复核——本决策涉及"域即权威"原则在生命周期操作上的适用边界，与"事件驱动传播"原则之间存在主观权衡空间，值得从体系架构视角独立审查。
