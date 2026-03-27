# Framework Consumer Agent Memory

- [free_fall 示例设计模式](free-fall-patterns.md) — 双域架构、组件设计、磁滞逻辑、已知约束
- [框架 API 已知痛点](api-pain-points.md) — step_with 闭包签名隐式 'static 问题（ISSUE-005）
- [多域协作开发困惑点](multi-domain-pain-points.md) — 服务接口调用方式、全量实体遍历、事件处理 spawn

## Issue 状态汇总

| ID | 状态 | 摘要 |
|---|---|---|
| ISSUE-001 | resolved | custom-domain.md 运动域依赖描述错误（已修复，文档已验证） |
| ISSUE-002 | resolved | downcast 便捷方法生命周期签名（修复不完整，见 ISSUE-005） |
| ISSUE-003 | resolved | event.md 等文档"计算阶段只读"矛盾（已修复，文档已验证） |
| ISSUE-004 | resolved | overview.md 初始化顺序和 lifecycle.md Initializing 说明（已修复，文档已验证） |
| ISSUE-005 | resolved | step_with 闭包隐式 'static 是 E0521 真正根源（维护者采纳，修复为明确 'static） |
| ISSUE-006 | resolved | decisions.md 补充 step_with 闭包签名决策记录 |
| ISSUE-007 | open | step_with 闭包中 world 参数的能力边界未文档化（事件处理阶段 spawn） |
| ISSUE-008 | open | 域服务接口定义与跨域调用方式无完整代码示范（registry 查询语法未知） |
| ISSUE-009 | open | 探测/战斗类域无法枚举全量活跃实体（ctx.entities 仅暴露管辖域内实体） |

## 关键开发规律

- compute 中读写组件必须"先 collect 实体 ID → 只读借用提取到局部变量 → 再可变借用写回"
- 域的事件发出（ctx.emit）只在确实有跨边界通知需求时使用，纯状态修改直接写组件
- `impl_component!` 宏是注册组件的标准写法；`domain_rules_any!` 宏替代 as_any 样板
- 域名建议定义为常量（`pub const DOMAIN_MOTION: &str = "motion"`），避免字符串拼写错误
- 跨域服务调用语法、ctx.entities 全量遍历语法尚未有文档，是多域示例的核心阻塞点
