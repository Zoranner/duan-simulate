---
id: ISSUE-001
title: custom-domain.md 参考实现中"运动域"的依赖声明与框架实现不符
type: doc-change
priority: p3-low
status: resolved
reporter: arch-designer
created: 2026-03-27
updated: 2026-03-27
---

## 问题描述

`docs/duan-docs/guides/custom-domain.md` 参考实现章节中，"运动域"条目写道：

> **依赖**：空间域。

但在 `examples/free_fall/src/domains/motion.rs` 的实际实现中，`MotionRules::dependencies()` 返回空列表——运动域无依赖，是最基础的域。

这条参考实现示例是在文档演进过程中残留的不准确描述，与实际示例代码不一致，会给参考文档的读者造成误导。

## 影响程度

- [x] 轻微（体验欠佳，但不影响核心功能）

## 复现场景

阅读 `custom-domain.md` 的"参考实现 → 运动域"章节，与 `examples/free_fall/src/domains/motion.rs` 对照时发现。

## 建议方案

**短期可改进**：将参考实现中"运动域"的依赖行改为：

> **依赖**：无（基础域，常被碰撞域等依赖）。

同时可在"产生事件"行补充说明：运动域直接修改组件状态，不需要通过事件传递——这与设计哲学"域是权威，直接写入"一致，对读者有正面示范价值。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：已修复，文档当前状态正确，变更方向与框架演进一致，无异议。

**分析**：

经核查，`docs/duan-docs/guides/custom-domain.md` 第 228–230 行的运动域参考实现描述如下：

- **依赖**：无（基础域，常被碰撞域等依赖）。
- **产生事件**：通常不产生——运动域作为权威直接修改自身管辖实体的位置和速度组件，无需绕道事件系统。

与 `examples/free_fall/src/domains/motion.rs` 中 `dependencies()` 返回空列表完全一致。Issue 所描述的"依赖：空间域"这一错误内容在当前文档版本中已不存在，两项建议（修正依赖声明 + 补充产生事件说明）均已落实。

此 doc-change 通知确认了一次有价值的文档修正，补充的"域直接修改组件，无需绕道事件系统"说明对读者理解权威域写入边界有正向示范价值，方向正确。

**行动计划**：无需额外操作，文档已处于正确状态。
