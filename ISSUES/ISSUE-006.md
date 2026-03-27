---
id: ISSUE-006
title: decisions.md 补充 step_with 闭包签名与 CustomEvent object lifetime 决策记录
type: doc-change
priority: p3-low
status: resolved
reporter: arch-designer
created: 2026-03-27
updated: 2026-03-27
---

## 修改了哪些文件

**`docs/duan-docs/reference/decisions.md`**（新增一条决策记录，版本更新至 v1.3）

## 修改摘要

在"已解决的问题"章节新增条目：**step_with 闭包签名与 CustomEvent 的 object lifetime**。

该条目记录了 ISSUE-002 和 ISSUE-005 揭示的完整根因链：

- ISSUE-002 修复了 `downcast` 方法的返回值生命周期绑定，但未解决 E0521
- ISSUE-005 发现真正根因是 `step_with` 闭包约束未明确 object lifetime，导致 Rust 无法将闭包参数匹配到 `impl (dyn CustomEvent + 'static)` 上的方法
- 最终决策：约束改为 `F: FnMut(&(dyn CustomEvent + 'static), &mut Self)`，是语义明确化而非行为变更
- 记录了 HRTB 方向为何无法解决此问题（参考 ISSUE-005 维护者评估），避免未来重踏同一路径

## 触发来源

ISSUE-005 维护者行动计划第4条明确要求：`decisions.md` 中标注的 ISSUE-002 修复说明需补充，根因比原分析更深，完整修复需同时调整 `step_with` 约束。本次文档更新履行该要求。

## 是否涉及概念定义或架构边界调整

否。本次修改仅在 `decisions.md` 的"已解决的问题"章节追加一条记录，不涉及概念定义或架构边界的调整。属于技术决策可追溯性的维护工作。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：接受（无异议）

**核查情况**：

文档变更在提交本 Issue 通知时已同步完成落地。核查 `docs/duan-docs/reference/decisions.md` 第49-59行，"step_with 闭包签名与 CustomEvent 的 object lifetime"条目已存在，版本标注为 v1.3，最后更新日期 2026-03-27。

**内容质量确认**：

该条目准确履行了 ISSUE-005 行动计划第4条的要求：

- **两阶段根因链完整呈现**：ISSUE-002 修复 `downcast` 返回值生命周期（第一阶段）→ ISSUE-005 发现真正根因在 `step_with` 约束的 lifetime elision（第二阶段），记录了完整的排查演进过程，有助于未来读者理解为何需要两次修复。
- **最终决策表述精确**："`'static` 明确化而非行为变更"这一定性准确——因为 `process_events` 实际传入的已是 `'static` object，约束修改只是消除了隐式推断的歧义。
- **HRTB 无效的理由说明到位**：记录了此路不通的原因，避免未来维护者重踏同一路径，这是 decisions.md 最有价值的功能之一。
- **文档版本与日期已同步更新**：v1.3 / 2026-03-27，无遗漏。

变更方向与框架演进一致，记录质量符合 decisions.md 的定位（技术决策可追溯性，而非概念定义或架构边界调整）。
