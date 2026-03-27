---
id: ISSUE-014
title: domain_rules_any! 是纯样板代码——应由 derive macro 替代
type: dx
priority: p3-low
status: resolved
reporter: framework-consumer
created: 2026-03-27
updated: 2026-03-27
---

## 问题描述

每个 `DomainRules` 实现结尾都必须加一行：

```rust
impl DomainRules for MyRules {
    fn compute(&mut self, ctx: &mut DomainContext) { ... }
    fn try_attach(&self, entity: &Entity) -> bool { ... }
    // ...

    domain_rules_any!(MyRules);  // ← 每次都要写，无语义
}
```

`domain_rules_any!` 展开后只是机械实现 `as_any(&self) -> &dyn Any` 和 `as_any_mut(&mut self) -> &mut dyn Any`，用于运行时类型转换。这两个方法的实现对所有类型完全相同，没有任何定制空间。

类似地，`impl_component!` 也是纯样板，但组件数量较多时累积起来尤为突出。

在 `naval_combat` 示例中，6 个域文件每个都有这一行。新手在写第一个域时往往需要从示例复制，很难理解这行的必要性。

## 影响程度

- [ ] 阻塞性
- [ ] 中等
- [x] 轻微（体验欠佳，但不影响核心功能）

## 复现场景

编写任意 `DomainRules` 实现时都会遇到，特别是在刚开始学习框架、不熟悉宏体系的阶段。

## 建议方案

**短期可改进**：

在 `guides/custom-domain.md` 中明确说明 `domain_rules_any!` 是必须的样板行，简要解释原因（Rust 的 `dyn Any` 向下转型需要 `as_any` 方法），避免用户困惑。

**需架构讨论**：

提供 `#[derive(DomainRules)]` 过程宏（proc macro），让用户只需标注 derive，不再需要手动调用 `domain_rules_any!`。类似地，`impl_component!` 也可以改写为 `#[derive(Component)]`。

这需要额外的 proc-macro crate，有一定工程成本，但能显著提升 API 的人体工学，特别是对框架新用户。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：采纳短期方案（文档说明必要性）。proc macro 方案（`#[derive(DomainRules)]`）纳入路线图，但当前阶段不实施。

**分析**：

问题成立，`domain_rules_any!` 的必要性对新用户完全不透明——每个域文件末尾这一行，既无法从宏名推断语义，也没有任何文档说明为什么需要它。一句话解释可消除困惑：Rust trait 对象的向下转型需要每个具体类型提供 `as_any(&self) -> &dyn Any` 方法，而 Rust 不允许在 trait 定义中为所有实现类提供默认实现（避免歧义），因此每个实现类必须手动提供，宏消除了机械重复。

proc macro 方案（`#[derive(DomainRules)]`）从 DX 角度有价值，但：
1. 需要独立 proc-macro crate，增加编译时间和 crate 图复杂度
2. 框架当前处于架构设计早期阶段，proc macro 基础设施会带来额外维护负担
3. 待 3 个以上示例积累、`domain_rules_any!` 的样板性被多位开发者验证后，再评估投入是否合理

**行动计划**：

- [x] 在 `guides/custom-domain.md` 的"实现注意事项"章节新增 `domain_rules_any!` 说明：解释其必要性（`as_any` 用于运行时 downcast，Rust 要求每个具体类型手动实现，宏消除机械重复），说明这是每个 `DomainRules` 实现必须包含的固定行

**关闭理由**（如拒绝或 wontfix）：proc macro 方案延后评估，文档已覆盖短期问题。
