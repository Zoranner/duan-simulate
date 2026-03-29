# Framework Consumer Agent Memory

- [free_fall 示例设计模式](free-fall-patterns.md) — 双域架构、组件设计、磁滞逻辑、已知约束
- [框架 API 已知痛点](api-pain-points.md) — step_with 闭包签名隐式 'static 问题（ISSUE-005）
- [多域协作开发困惑点](multi-domain-pain-points.md) — 服务接口调用方式、全量实体遍历、事件处理 spawn

## Issue 状态汇总

| ID | 状态 | 摘要 |
|---|---|---|
| ISSUE-001 | closed | custom-domain.md 运动域依赖描述错误（已修复，文档已验证） |
| ISSUE-002 | closed | downcast 便捷方法生命周期签名（修复不完整，见 ISSUE-005） |
| ISSUE-003 | closed | event.md 等文档"计算阶段只读"矛盾（已修复，文档已验证） |
| ISSUE-004 | closed | overview.md 初始化顺序和 lifecycle.md Initializing 说明（已修复，文档已验证） |
| ISSUE-005 | closed | step_with 闭包隐式 'static 是 E0521 真正根源（维护者采纳，修复为明确 'static） |
| ISSUE-006 | closed | decisions.md 补充 step_with 闭包签名决策记录 |
| ISSUE-007 | closed | step_with 闭包中 world 参数能力边界——event.md 已补充说明，源码验证可行 |
| ISSUE-008 | open | 文档"方式二"示例错误：应用 get_domain_by_name_raw()，而非 get_domain_by_name() |
| ISSUE-009 | closed | ctx.entities.active_entities() 文档已补充说明全量遍历合法 |
| ISSUE-021 | open | 域写入边界仅靠文档约定，框架无技术机制阻止越权修改（从 ISSUE-009 升级） |
| ISSUE-022 | open | compute_domains() 使用 unsafe 裸指针，安全性依赖未记录的内存布局不变量 |
| ISSUE-023 | open | 同层无依赖域执行顺序不确定（HashMap 迭代顺序随机），影响仿真可复现性 |
| ISSUE-010 | closed | event.md 补充"实体生效时序"说明（当帧 spawn 下帧才生效，文档已验证） |
| ISSUE-017 | closed | custom-domain.md 补充"域的写入边界"小节（不可 spawn 的原因与推荐模式） |
| ISSUE-018 | closed | domain.rs compute_execution_order 增加依赖名称合法性校验（panic on missing dep） |
| ISSUE-019 | closed | world.rs 新增 step_collect(dt)；guides/testing.md 新增测试域逻辑指南 |
| ISSUE-020 | closed | custom-domain.md 补充 on_attach/on_detach 生命周期钩子完整说明与示例 |

## 关键开发规律

- compute 中读写组件必须"先 collect 实体 ID → 只读借用提取到局部变量 → 再可变借用写回"
- 域的事件发出（ctx.emit）只在确实有跨边界通知需求时使用，纯状态修改直接写组件
- `impl_component!` 宏是注册组件的标准写法；`domain_rules_any!` 宏替代 as_any 样板
- 域名建议定义为常量（`pub const DOMAIN_MOTION: &str = "motion"`），避免字符串拼写错误
- 跨域服务调用语法、ctx.entities 全量遍历语法尚未有文档，是多域示例的核心阻塞点
