---
name: 已评估的 Issue 记录
description: 历次 Issue 评估结论，用于追溯决策依据和发现系统性模式
type: project
---

## ISSUE-001（2026-03-27）

**类型**：doc-change
**优先级**：p3-low
**最终状态**：accepted

**结论**：文档在提交 Issue 时已是正确状态，两项建议（修正运动域依赖声明 + 补充产生事件说明）均已落实，变更方向与框架演进一致，无异议。

**Why**：arch-designer 自行修复后提交通知，核查确认文档与实现代码一致。

**验证路径**：
- `docs/duan-docs/guides/custom-domain.md` 第 228–230 行
- `examples/free_fall/src/domains/motion.rs` 中 `dependencies()` 返回空列表

---

## ISSUE-002（2026-03-27）

**类型**：api-design
**优先级**：p2-medium
**最终状态**：fixed

**结论**：采纳 reporter 方向 1，直接修复实现。

**根因**：`impl dyn CustomEvent` 块中 Rust 将裸 `dyn Trait` 隐式解析为 `dyn Trait + 'static`，导致 `downcast` 方法要求 `self` 满足 `'static`，在 `step_with` 闭包的短生命周期上下文中触发 E0521。

**修复内容**：
- `src/events.rs`：`downcast` 签名引入显式生命周期 `'a`，从 `(&self) -> Option<&T>` 改为 `(&'a self) -> Option<&'a T>`，语义不变。
- `examples/free_fall/src/main.rs`：恢复使用 `event.downcast::<T>()` 正常写法，移除绕行注释。
- `cargo build` 验证通过。

**决策先例**：API 实现缺陷应直接修复，不应以文档说明掩盖（方向 3），也不应因可修复的小缺陷移除有价值的便捷方法（方向 2 过激）。

---

## ISSUE-003（2026-03-27）

**类型**：doc-change
**优先级**：p2-medium
**最终状态**：accepted

**结论**：三处文档内部矛盾全部已修复，变更方向与权威域架构核心哲学一致，无异议。

**修复内容**：
- `event.md` 第96行：删除"唯一合法修改实体状态的地方"，改为准确描述域直接写入权与事件处理器跨边界职责的分工。
- `glossary.md` DomainContext 条目：改为"实体存储（可变，域可修改自身管辖实体的组件）"；EventHandler 条目：改为"负责跨边界的状态操作"。
- `philosophy.md` 事件驱动传播章节：删除"计算阶段只读"这一纯 ECS 风格的错误表述，明确两层分工（域内直接写入 vs 跨边界事件通信）。

**设计确认**：此次修正澄清了 DUAN 与纯 ECS 事件驱动的根本区别——DUAN 的事件边界是跨边界操作，域对自身管辖实体的计算阶段直接写入是权威性的体现，不需要绕道事件系统。这是 patterns.md"权威域写入边界"模式的文档层面的系统性落实。

---

## ISSUE-004（2026-03-27）

**类型**：doc-change
**优先级**：p3-low
**最终状态**：accepted

**结论**：三处文档变更全部落实，变更方向与框架实现和设计哲学一致，无异议。

**修复内容**：
- `overview.md` 初始化顺序：删除不存在的"注册事件处理器"步骤，补充说明"事件处理器不是注册式的，通过 `step_with` 闭包传入"。
- `overview.md` 计算阶段数据流图：增加"直接写入自身管辖实体的组件状态"分支，与 ISSUE-003 修正后的设计哲学保持一致。
- `lifecycle.md` Initializing 状态：补充"这是同步瞬间状态，不会跨帧停留"的说明，与 `spawn()` 实现（同步完成状态转移）准确对应。

---

## ISSUE-005（2026-03-27）

**类型**：api-design
**优先级**：p2-medium
**最终状态**：accepted（待修复）

**结论**：采纳根因诊断，建议方案需修正。ISSUE-002 修复不完整，E0521 根源在 `step_with` 约束的 lifetime elision，而非 `downcast` 签名。

**完整根因链**：
1. `step_with` 约束 `F: FnMut(&dyn CustomEvent, &mut Self)` 中，`&dyn CustomEvent` 按 Rust HRTB 展开为 `for<'r> FnMut(&'r (dyn CustomEvent + 'r), ...)`，闭包参数 object lifetime 为不确定的 `'r`。
2. `impl dyn CustomEvent` 隐式等价于 `impl (dyn CustomEvent + 'static)`，`downcast` 的 `self` 要求 object lifetime 满足 `'static`，`'r: 'static` 不成立，编译器报 E0521。
3. `process_events` 实际传入 `&(dyn CustomEvent + 'static)`（来自 `Arc<dyn CustomEvent>`），约束表达与实现语义不匹配。

**修复方向**：将约束从 `F: FnMut(&dyn CustomEvent, ...)` 改为 `F: FnMut(&(dyn CustomEvent + 'static), ...)`，明确 object lifetime 为 `'static`，反映实际传入类型。`step_with`、`do_step`、`process_events` 三处同步修改。Reporter 建议的 HRTB 方案（`for<'e> FnMut(&'e (dyn CustomEvent + 'e), ...)`）不能解决问题。

**决策先例**：API 约束应精确反映实际传入类型的 lifetime 语义，避免 Rust 的隐式 HRTB 展开造成与实现不一致的约束。

---

## ISSUE-006（2026-03-27）

**类型**：doc-change（arch-designer 提交）
**优先级**：p3-low
**最终状态**：accepted

**结论**：文档变更在提交通知时已同步落地，内容完整准确，无异议。

**核查确认**：
- `decisions.md` 第49-59行"step_with 闭包签名与 CustomEvent 的 object lifetime"条目已存在
- 完整呈现两阶段根因链（ISSUE-002 + ISSUE-005）
- 记录了 HRTB 方案为何无效，防止未来重踏
- 文档版本更新为 v1.3，日期同步

**触发来源**：ISSUE-005 行动计划第4条明确要求，本 Issue 是该要求的文档履行通知。

---

## ISSUE-007（2026-03-27）

**类型**：documentation
**优先级**：p2-medium
**最终状态**：fixed

**结论**：部分采纳。问题成立——`step_with` 闭包中 `world` 参数的能力边界确实未在文档中说明。在 `event.md` 的"注册与执行上下文"章节补充说明，建议方案 2（overview.md 数据流）和方案 3（新增 idioms.md）均不采纳。

**技术确认**：`process_events` 中先执行 `self.events.drain()` 将事件全部取出，之后传入闭包的 `&mut World` 无其他活跃借用，`world.spawn()` 可以合法调用。`register_domain()` 技术上不崩溃，但语义禁止（初始化后执行顺序已固化）。

**修改位置**：`docs/duan-docs/concepts/event.md` 的"注册与执行上下文"章节。

**决策先例**：API 能力边界说明应放在概念文档的执行上下文章节，而非架构概览的数据流图或新增的惯用法文件。操作能力列表（支持/不支持）是执行上下文语义的自然组成部分，不构成文档膨胀。

---

## ISSUE-008（2026-03-27）

**类型**：documentation
**优先级**：p1-high
**最终状态**：fixed

**结论**：采纳。跨域服务调用 API 已存在于实现，文档完全未展示，补充代码示例。

**技术确认**：
- `DomainContext.get_domain::<T>()` 存在，返回 `Option<&T>`，无需 downcast，是推荐用法
- `DomainContext.get_domain_by_name(name)` 存在，返回 `Option<&Domain>`，获取具体类型需 `domain.rules.as_any().downcast_ref::<T>()`
- `DomainRules` trait 强制要求 `as_any` 方法，downcast 路径可行

**修改位置**：
- `docs/duan-docs/guides/custom-domain.md` 探测域参考实现后新增"跨域服务调用示例"
- `docs/duan-docs/concepts/domain.md` 域注册表章节补充具体方法名，域上下文表格 `registry` 行更新

**决策先例**：两种服务查找方式（按类型 vs 按名称）均应在文档中有代码示例；方式一是常规场景推荐，方式二适用于动态配置场景。

---

## ISSUE-009（2026-03-27）

**类型**：concept-clarity
**优先级**：p1-high
**最终状态**：fixed

**结论**：采纳。框架已支持全量实体遍历，文档未说明，补充说明并明确设计立场。

**架构决策**：全量只读遍历（`ctx.entities.active_entities()`）是合法的，不违反权威域架构。权威边界的核心约束是**写入**——域只能修改自己管辖的实体，**读取**全量实体不破坏任何域的权威。跨实体交叉计算（探测、战斗）是真实需求，框架应当支持。

**技术确认**：
- `EntityStore.iter()` 迭代全部实体，`EntityStore.active_entities()` 只返回活跃实体
- `ctx.entities` 是完整的 `&mut EntityStore`，`active_entities()` 可直接调用

**两种目标枚举方式**（均合法）：
- 全量遍历 `ctx.entities.active_entities()`：简单直接，适合实体数量不大的场景
- 通过空间域范围查询服务：避免 O(n²)，利用空间索引加速，推荐用于探测/战斗等范围敏感场景

**修改位置**：
- `docs/duan-docs/concepts/domain.md` 域上下文表格 `entities` 行补充全量遍历说明
- `docs/duan-docs/guides/custom-domain.md` 探测域参考实现新增"遍历潜在目标"章节

**决策先例**：`ctx.entities` 的"可变"访问权限不应被理解为"只操作管辖实体"——它是对 EntityStore 的完整访问；写入约束由开发者自律维护，框架不强制检查。

