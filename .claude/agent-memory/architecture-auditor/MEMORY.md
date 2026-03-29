# architecture-auditor 记忆索引

- [框架实现合规状态](compliance-audit-2026-03-27.md) — 首次完整审计结论，整体高度合规，记录关键符合点与例外处理
- [已知文档偏差](doc-gaps.md) — 反复出现或已知的文档盲区与不一致（含三批修正记录）
- [整体设计审计（2026-03-30）](design-audit-2026-03-30.md) — 全面架构审计结论，发现5个结构性问题，整体评分 6/10

## 核心发现（跨审计轮次稳定结论）

**"文档哲学"与"API 设计"长期脱节**  
框架文档反复强调"域即权威"、"边界分明"，但框架自身的 API 设计多次与之矛盾（World 字段全 pub、Entity::add_domain post-spawn 无效等）。这是需要持续关注的系统性问题。

**as_any 样板污染多个核心 trait**  
Component、DomainRules、CustomEvent 三个核心 trait 均含 `as_any/as_any_mut` 样板方法。这是 Rust 类型系统限制的工程妥协，宏缓解了实现负担，但设计层面的不优雅持续存在。

## 已提出但待处理的重要建议

| ISSUE | 问题 | 优先级 | 状态 |
|-------|------|--------|------|
| ISSUE-024 | World 字段全 pub 破坏权威边界 | p1-high | open |
| ISSUE-025 | Entity::add_domain/remove_domain post-spawn 静默无效 | p1-high | open |
| ISSUE-026 | DomainEvent::EntitySpawned 死代码 | p2-medium | open |
| ISSUE-027 | on_attach/on_detach 不对称无文档依据 | p2-medium | open |
| ISSUE-028 | component_type() 必须实现但框架内部不使用 | p2-medium | open |

## 已关闭 / 已处理

- ISSUE-022（unsafe + INVARIANT 文档）：维护者决策已接受，但 take/restore 模式可在未来消除 unsafe 时参考
- ISSUE-023（拓扑排序不确定性）：sort_unstable 已修复
