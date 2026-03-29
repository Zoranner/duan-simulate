---
id: ISSUE-025
title: Entity::add_domain / remove_domain 在实体加入仿真后调用，修改无效且无任何提示
type: architecture
priority: p1-high
status: open
reporter: architecture-auditor
created: 2026-03-30
updated: 2026-03-30
---

## 问题描述

`Entity` 提供了 `add_domain()` 和 `remove_domain()` 两个方法，用于修改实体的 `domains: HashSet<String>` 字段。

这两个方法在**实体 spawn 之后调用**，只修改 `Entity` 内部的 `HashSet`，对 `DomainRegistry` 没有任何影响：

- 域注册表中的 `Domain.entities` 集合不会更新
- `on_attach` / `on_detach` 生命周期钩子不会触发
- 实体不会被附加到新声明的域，也不会从原域移除

这是一个**静默的无效操作**，不产生错误、不触发警告、不改变仿真行为。

## 架构层面的问题

从体系架构视角，这里存在两个独立的问题：

### 问题一：API 语义欺骗

`add_domain("physics")` 的字面语义是"将实体加入物理域"。用户有充分理由相信调用此方法后实体会参与物理域的计算。但事实上什么都不会发生。

这不是"使用不当"，而是 API 名称和实际行为的根本性背离。在"域即权威"的框架中，"声明加入某个域"是核心操作，为其提供一个安静失效的假接口，是严重的误导设计。

### 问题二：`Entity.domains` 字段的双重含义混乱

`Entity.domains` 这个字段在实体的不同生命周期阶段有完全不同的含义：

- **spawn 之前**：`domains` 是"意图声明"，`World::spawn()` 会读取它来决定将实体附加到哪些域
- **spawn 之后**：`domains` 变成了一个"历史快照"，只记录 spawn 时的声明状态，不再有任何功能性作用

这种双重含义没有在任何文档或 API 设计中得到明确区分。框架没有阻止用户在 spawn 后修改这个字段，而修改它既不出错也不生效，是一个完美的静默陷阱。

## 影响程度

- [x] 中等（影响开发效率或理解，有变通方式）

> 注：开发者在调试"为什么域没有处理新实体"时，如果发现是 `add_domain()` 调用时机的问题，会花费大量时间，且框架层完全无线索。这在多人协作开发多域系统时尤其危险。

## 建议方向

有三个方向，选择其一：

**方向一：从根本上修复——让 `add_domain/remove_domain` 在 spawn 后也能生效**

这要求 `World` 提供受控的域成员管理 API，能够在运行时触发 `DomainRegistry` 更新和 `on_attach/on_detach` 钩子。这是最完整的修复，但实现成本最高。

**方向二：防止 spawn 后调用——区分构建期和运行期接口**

将 `add_domain / remove_domain` 移出 `Entity`，改为只在构建器（`EntityBuilder`）上存在，`Entity` 自身不暴露修改 `domains` 的接口。spawn 后，`domains` 字段设为只读（改为 `pub(crate)` 或完全私有）。

**方向三（最低成本）：在调用时 panic 或明确禁止**

如果框架决定不支持运行时域成员变更，那么应该在 `add_domain / remove_domain` 内部检查实体是否已处于 `Active` 状态，若是则 `panic!`，使误用立即可见，而非静默失效。
