---
id: ISSUE-015
title: 事件 vs 服务调用的选择准则缺失——导致设计决策模糊
type: concept-clarity
priority: p2-medium
status: resolved
reporter: framework-consumer
created: 2026-03-27
updated: 2026-03-27
---

## 问题描述

框架提供了两种域间通信机制：

1. **事件（Event）**：域 A 在 `compute` 中发出事件，事件在帧末由 `step_with` 闭包处理，或触发其他域的响应
2. **服务调用（Service Call）**：域 B 在 `compute` 中通过 `ctx.get_domain::<A>()` 获取域 A 的引用，直接调用其服务方法查询状态

这两种机制在功能上有重叠，但文档中没有给出"何时用事件、何时用服务调用"的明确指导原则。

在实际开发 `naval_combat` 时，我遇到了具体的决策困境：

- `DetectionRules` 既发出 `DetectionEvent`（通知"发现了目标"），又提供 `get_detected(id)` 服务方法
- `CombatRules` 实际上使用的是 `get_detected()` 服务方法，而不是监听 `DetectionEvent`
- 这意味着 `DetectionEvent` 在整个仿真中从未被消费（最终在 `display.rs` 中作为 `LogEntry::Detection` 存在，但 main.rs 里也未使用）

这揭示了一个设计模糊地带：**我是否应该通过事件来驱动战斗逻辑，而不是通过服务调用？** 两种方式都能工作，但产生了不同的时序和耦合特征，框架没有给出选择依据。

## 影响程度

- [ ] 阻塞性
- [x] 中等（影响开发效率或理解，有变通方式）
- [ ] 轻微

## 复现场景

在设计探测→战斗这条信息链时：

- **方案 A（服务调用）**：`CombatRules::compute` 直接调用 `DetectionRules::get_detected(id)`，同步获取当前帧的探测结果，立即做出开火决策。
- **方案 B（事件驱动）**：`DetectionRules` 发出 `DetectionEvent`，`CombatRules` 在某种机制下订阅该事件并在下一帧响应。

两种方案的时序不同（同帧 vs 跨帧），耦合程度不同，但框架都支持。在没有指导原则的情况下，只能凭经验猜测"正确"做法。

## 建议方案

**短期可改进**：

在 `concepts/event.md` 或 `guides/custom-domain.md` 中增加"事件 vs 服务调用"对比说明，明确两者的适用场景：

| | 事件 | 服务调用 |
|---|---|---|
| 时序 | 跨帧（异步） | 同帧（同步） |
| 适用 | 状态变化通知、触发外部响应（spawn/destroy）| 查询当前帧计算结果，驱动本域决策 |
| 耦合 | 松耦合（发出方不知道谁处理） | 紧耦合（调用方依赖具体域类型） |
| 典型用途 | FireEvent、HitEvent、ShipDestroyedEvent | distance()、get_detected()、is_hostile() |

**建议的选择准则**：
- **用事件**：当一个域需要通知"发生了某件事"，且响应可能涉及实体的 spawn/destroy/健康扣减等世界状态变更
- **用服务调用**：当一个域需要查询另一个域"当前帧计算出的状态"来做本域的决策，结果需要同帧可用

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：采纳，且优先级提至 p1。事件 vs 服务调用的选择准则缺失是框架设计哲学层面的重要文档缺口，影响所有多域协作场景的设计决策。

**分析**：

问题高度成立。Reporter 在 `naval_combat` 中观察到的 `DetectionEvent` 从未被消费现象，是这种设计模糊导致的：既然可以通过服务调用同帧获取探测结果，事件就变成了"可选的通知"，其消费语义不清晰，开发者无从判断是否遗漏了什么。

正确理解：两种机制不是竞争关系，而是互补——

- `DetectionRules` 发出 `DetectionEvent`（通知事件，面向日志/回放/外部观察者）是正确的
- `CombatRules` 在 `compute` 中调用 `get_detected(id)` 服务（同帧查询，驱动决策）也是正确的
- 两件事职责不同，都应该存在，`DetectionEvent` 不是多余的

Reporter 提出的对比表格框架正确，选择准则基本准确，采纳并在 `concepts/event.md` 中落地（该话题属于框架通信机制的核心概念，而非仅是使用指南）。

**行动计划**：

- [x] 在 `concepts/event.md` 末尾新增"事件与服务调用的选择"章节，包含对比表格、两条选择准则、以探测→战斗链为例的并存说明

**关闭理由**（如拒绝或 wontfix）：
