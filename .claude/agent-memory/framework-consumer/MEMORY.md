# Framework Consumer Agent Memory

- [free_fall 示例设计模式](free-fall-patterns.md) — 双域架构、组件设计、磁滞逻辑、已知约束
- [框架 API 已知痛点](api-pain-points.md) — step_with 闭包签名隐式 'static 问题（ISSUE-005）

## Issue 状态汇总

| ID | 状态 | 摘要 |
|---|---|---|
| ISSUE-001 | resolved | custom-domain.md 运动域依赖描述错误（已修复，文档已验证） |
| ISSUE-002 | fixed | downcast 便捷方法生命周期签名（修复不完整，见 ISSUE-005） |
| ISSUE-003 | resolved | event.md 等文档"计算阶段只读"矛盾（已修复，文档已验证） |
| ISSUE-004 | resolved | overview.md 初始化顺序和 lifecycle.md Initializing 说明（已修复，文档已验证） |
| ISSUE-005 | open | step_with 闭包隐式 'static 是 E0521 真正根源（新提） |

## 关键开发规律

- compute 中读写组件必须"先 collect 实体 ID → 只读借用提取到局部变量 → 再可变借用写回"
- 域的事件发出（ctx.emit）只在确实有跨边界通知需求时使用，纯状态修改直接写组件
- `impl_component!` 宏是注册组件的标准写法；`domain_rules_any!` 宏替代 as_any 样板
