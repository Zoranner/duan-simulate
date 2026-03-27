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

