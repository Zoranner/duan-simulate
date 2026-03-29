---
id: ISSUE-022
title: compute_domains() 使用 unsafe 裸指针绕过借用检查，安全性依赖未记录的内存布局不变量
type: architecture
priority: p2-medium
status: open
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

**结论**：

**分析**：

**行动计划**：

- [ ] 

**关闭理由**（如拒绝或 wontfix）：
