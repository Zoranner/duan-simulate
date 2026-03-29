---
id: ISSUE-022
title: compute_domains() 使用 unsafe 裸指针绕过借用检查，安全性依赖未记录的内存布局不变量
type: architecture
priority: p2-medium
status: resolved
reporter: framework-consumer
created: 2026-03-30
updated: 2026-03-30
---

## 问题描述

在阅读 `src/world.rs` 的 `compute_domains()` 实现时，发现以下 `unsafe` 代码块（第 344-368 行）：

```rust
let (rules_ptr, own_entities_ptr) = match self.domains.get_by_name_mut(domain_name) {
    Some(domain) => (
        &mut *domain.rules as *mut dyn DomainRules,
        &domain.entities as *const HashSet<EntityId>,
    ),
    None => continue,
};

let mut ctx = DomainContext {
    own_entities: unsafe { &*own_entities_ptr },
    entities: &mut self.entities,
    registry: &self.domains,
    events: &mut self.events,
    clock: &self.clock,
    dt,
};

unsafe {
    (*rules_ptr).compute(&mut ctx);
}
```

代码注释说明了使用 unsafe 的理由：

> SAFETY：两个指针指向 Domain 结构体的不同字段（rules 和 entities），不存在别名。compute_domains 期间注册表结构不变（无 insert/remove），域自身的实体集合通过 ctx.registry（不可变引用）也不会被修改。

这段注释是合理的，unsafe 使用本身可能是正确的。但有两个问题需要评估：

**问题一：安全性断言依赖未在类型系统中表达的不变量**

"两个指针指向 Domain 结构体的不同字段"这一断言依赖于：
1. `Domain` 结构体中 `rules` 和 `entities` 字段不共享内存（这是正确的，但如果未来有人将 `Domain` 重构为 `Arc` 持有的共享结构，这个假设会悄悄失效）
2. `compute_domains` 期间没有任何代码路径触发 `insert/remove`（这依赖调用方约定，不是编译期约束）

任何修改 `Domain` 内存布局或 `DomainRegistry` 并发访问语义的变更都可能让这段 unsafe 的正确性断言悄悄失效，而编译器不会给出任何警告。

**问题二：对外暴露的 API 表面积是"安全 Rust"，但内部存在未记录的 unsafe**

框架向用户展示的所有公开 API 都是安全的 Rust 接口。用户无法从 API 文档中了解到框架内部使用了 unsafe 裸指针，也无法评估这对框架扩展性或可维护性的影响。这不是说用户需要知道实现细节，而是：

- 贡献者在修改 `Domain` 结构体或 `DomainRegistry` 时，需要意识到这里的 unsafe 约束
- 当前代码中 SAFETY 注释已经存在，但注释的受众是"知道这里有 unsafe"的读者，对于只阅读 `Domain` 结构体定义的贡献者，没有任何提示

## 影响程度

- [ ] 阻塞性
- [x] 中等（影响开发效率或理解，有变通方式）
- [ ] 轻微

> 注：当前实现的 unsafe 可能是完全正确的。这个 Issue 的核心诉求是评估是否有更安全的替代方案，以及是否需要在 `Domain`/`DomainRegistry` 的修改路径上增加安全文档。

## 复现场景

当前代码在常规使用下不会触发问题。风险点在于：

1. **未来重构**：如果有人将 `DomainRegistry` 内部从 `HashMap<String, Domain>` 改为其他数据结构，裸指针的别名假设可能失效
2. **并行化探索**：如果未来考虑并行执行 `compute_domains()`，当前的裸指针方案是一个必须先解决的障碍

## 建议方案

**需架构讨论**：

**方向一：调查是否存在安全 Rust 的替代实现**

根本原因是：需要在持有 `&mut domain.rules` 的同时，也传入 `&domain.entities` 和 `&self.domains` 等其他引用。`RefCell<Box<dyn DomainRules>>` 或将 `rules` 提取到独立容器可能消除对裸指针的需求。

**方向二：维持现状，但增强安全文档**

如果安全 Rust 方案成本过高，至少应：
1. 在 `Domain` 结构体定义处添加注释，说明字段布局对 `compute_domains()` 中 unsafe 代码的依赖
2. 在 `DomainRegistry` 的结构修改指南中明确提示 unsafe 约束
3. 考虑是否需要 `#[repr(C)]` 或类似标注来稳定结构体布局

**短期可改进**：

在 `Domain` 结构体和 `DomainRegistry` 的关键修改点附近添加 `// INVARIANT` 注释，明确说明哪些属性对 `compute_domains()` 的 unsafe 正确性至关重要，帮助未来的修改者不会无意间打破假设。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：部分采纳。unsafe 本身保留，以完整 INVARIANT 文档替代结构重构。

**分析**：

经评估，`compute_domains()` 中的 unsafe 是**声音的（sound）**：`rules_ptr` 和 `own_entities_ptr` 分别指向同一 `Domain` 结构体的不同堆分配字段，不存在别名；`Box<dyn DomainRules>` 和 `HashSet<EntityId>` 的堆地址在 compute 期间稳定（无 HashMap 重分配）；`ctx.registry` 只读路径不产生可变别名。

**评估消除 unsafe 的可行性**：

- `RefCell<Box<dyn DomainRules>>` 方案：可消除 `compute_domains` 中的 unsafe，但 `get_domain<T>()` / `get_domain_by_name<T>()` 无法返回 `Option<&T>`（`Ref<>` 守卫不能跨函数返回），需要引入 unsafe 转移或改变 API 签名，问题转移而非消除。
- `Option<Box<dyn DomainRules>>` take/restore 方案：技术可行，但会让 `Domain.rules` 的类型变为 `Option<...>`，所有访问路径都需要处理 `None`，API 污染代价大于实际收益。
- 分离 rules 到独立容器方案：需要大规模重构 `DomainRegistry` 和 `DomainContext`，成本不符合 p2-medium 级别问题的投入比。

**最终决策**：保留 unsafe，但以系统性文档填补"贡献者盲区"——这才是 Issue 的核心诉求。

**行动计划**：

- [x] 在 `Domain` 结构体文档中添加 `# INVARIANT` 区块，明确列出三条安全约束（字段无别名、结构稳定、只读路径不写入）及其与 `compute_domains` unsafe 的关联
- [x] 将 `compute_domains()` 中的 SAFETY 注释扩展为完整的三条分项说明，并添加"未来并行化前必须重评估"的警示
- [x] 运行 `cargo clippy --all-targets --all-features -- -D warnings` 验证无新增警告

**架构哲学一致性**：已自验证
