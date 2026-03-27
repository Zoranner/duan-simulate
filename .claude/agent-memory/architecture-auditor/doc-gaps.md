---
name: 已知文档偏差与盲区
description: 记录文档与实现之间已知的不一致，以及持续的文档盲区
type: project
---

## 已修正（2026-03-27，第一批）

**custom-domain.md 运动域依赖描述错误**
- 原文：`依赖：空间域`
- 修正为：`依赖：无（基础域，常被碰撞域等依赖）`
- 关联 ISSUE-001（维护者已确认）

**event.md / glossary.md / philosophy.md 内部矛盾（"计算阶段只读"错误）**
- event.md 第96行"唯一合法修改实体状态的地方"已改为正确的跨边界说明
- glossary.md DomainContext 词条已更正为"可变"，EventHandler 词条已去掉错误措辞
- philosophy.md 事件驱动传播章节已重写，明确区分域内直接写入和跨边界事件传播
- 关联 ISSUE-003（维护者已确认）

## 已修正（2026-03-27，第二批）

**overview.md 初始化顺序描述包含不存在的"注册事件处理器"步骤**
- 删除了错误步骤，补充说明事件处理器是 step_with 闭包传参，非注册式
- 关联 ISSUE-004

**overview.md 计算阶段数据流图遗漏域直接写入路径**
- 原图只展示事件路径，补充了"直接写入自身管辖实体组件"分支
- 关联 ISSUE-004

**lifecycle.md Initializing 状态未说明其瞬间同步特性**
- 补充说明该状态是 spawn() 内部的同步瞬间操作，不跨帧
- 关联 ISSUE-004

## 已修正（2026-03-27，第三批）

**decisions.md 缺少 step_with 闭包签名 / CustomEvent object lifetime 决策记录**
- ISSUE-005 维护者行动计划第4条要求补充，本批次完成
- 记录了 ISSUE-002（downcast 返回值生命周期）到 ISSUE-005（step_with 约束明确化）的完整根因链
- 关联 ISSUE-006（architecture-auditor doc-issue 通知）

## 持续盲区观察

**Mass 组件未被任何域使用**：free_fall 示例中小球挂载了 Mass 组件，但 MotionRules 不读取它（使用固定重力加速度）。如果将来有用户问"为什么有 Mass 组件但没效果"，可能需要在示例代码注释中补充说明。

**custom-domain.md 中缺少"域跨越边界修改实体"的限制说明**：文档虽然提到域只能修改自身管辖实体，但没有明确说明实现层如何实现这一约束（目前靠 own_entities 集合语义约束，不是编译期强制）。用户可能误以为框架会阻止越界，实际上是依赖开发者自律。
