- [已评估的 Issue 记录](decisions.md) — 历次 Issue 评估结论及关键判断
- [框架设计模式](patterns.md) — 在 Issue 处理中确认或发现的稳定模式

## 已关闭 Issue 摘要

| Issue | 结论 | 要点 |
|-------|------|------|
| ISSUE-008 | 采纳 | 修正 `custom-domain.md` 方式二示例（错误用 `get_domain_by_name` 访问 `.rules`，应用 `get_domain_by_name_raw`）；澄清三种查找方式场景 |
| ISSUE-021 | 采纳（方向二） | breaking change：`ctx.entities` 改为 `pub(crate)`，新增 `entities()` 只读方法和 `get_own_entity_mut()` 受限写入方法，在类型系统层面强制权威边界 |
| ISSUE-022 | 部分采纳 | unsafe 保留（已验证 sound），拒绝重构（RefCell/Option 方案各有代价），以完整 INVARIANT 文档替代 |
| ISSUE-023 | 采纳 | `compute_execution_order()` 外层循环改为字典序遍历，消除 HashMap 迭代顺序不确定性 |

## 架构决策记录

### DomainContext.entities 设计（ISSUE-021）
- 结论：`pub(crate) entities: &'a mut EntityStore`，外部通过 `entities()` 只读、`get_own_entity_mut()` 受限写入
- 理由："域即权威"必须在类型层面强制执行，文档约定无法防止多开发者团队中的越权修改
- breaking change 判断：合理，写入意图应明确，调用点即可判断是否符合权威边界

### compute_domains() unsafe 保留（ISSUE-022）
- unsafe 是 sound 的，依赖三个不变量（无别名、结构稳定、只读路径不写入）
- RefCell 方案：破坏 `get_domain<T>()` 返回 `Option<&T>` 的语义（Ref<> 守卫生命周期问题）
- Option<Box> take/restore：技术可行但让 Domain.rules 类型变为 Option，API 污染大
- 正确解法：完整 INVARIANT 文档（Domain struct + compute_domains SAFETY 注释）

## 系统性问题记录

- `ctx.entities` 的读写权限分离是框架早期设计欠考虑的地方，现已通过 ISSUE-021 彻底修复
- `compute_domains()` 的 unsafe 是框架唯一的 unsafe 代码，须在未来并行化前重评估
- 文档示例中出现了与代码不一致的错误（ISSUE-008），提示：文档示例修改后应通过编译验证
