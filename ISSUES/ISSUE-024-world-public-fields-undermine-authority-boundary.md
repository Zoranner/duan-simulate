---
id: ISSUE-024
title: World 字段全 pub 暴露，使 DomainContext 的权威边界形同虚设
type: architecture
priority: p1-high
status: open
reporter: architecture-auditor
created: 2026-03-30
updated: 2026-03-30
---

## 问题描述

`World` 结构体除 `next_entity_id` 外，所有字段均为 `pub`：

```rust
pub struct World {
    pub clock: TimeClock,
    pub domains: DomainRegistry,
    pub entities: EntityStore,
    pub events: EventChannel,
    pub timer_manager: TimerManager,
    next_entity_id: u64,  // 唯一非 pub 字段
}
```

这在以下三个层面制造了根本性的封装矛盾：

### 矛盾一：`DomainContext` 的写入边界保护被完全绕过

ISSUE-021 进行了一次有代价的 breaking change，将 `DomainContext.entities` 改为 `pub(crate)`，新增 `get_own_entity_mut()` 方法，以确保域只能写入自己管辖的实体。这个改动建立在"域即权威"的原则上，有充分的架构依据。

但用户持有的是 `&mut World`，始终可以：

```rust
// ISSUE-021 精心构建的边界在这里一句话绕过
world.entities.get_mut(any_entity_id).unwrap()
    .get_component_mut::<Position>()
    .unwrap()
    .x = 999.0;
```

`DomainContext` 内的保护是局部的——它只在 `compute()` 执行期间有效。`compute()` 之外，`World.entities` 是完全公开的可变引用。ISSUE-021 花了大力气保护的是一扇窗，而正门始终大开。

### 矛盾二：`World.clock` 全 pub，仿真时间可被任意篡改

`TimeClock` 字段全 pub，意味着：

```rust
world.clock.sim_time = 9999.0;   // 跳跃仿真时间
world.clock.step_count = 0;      // 重置步进计数
```

框架提供了 `tick()` 等受控时间推进接口，但 pub 字段使这些接口形同摆设。对于一个强调"可复现性"（ISSUE-023 修复的初衷）的仿真框架，允许时间被随意篡改是自相矛盾的。

### 矛盾三：`World.domains` 全 pub，域实体集合可被直接修改

`DomainRegistry::iter_mut()` 暴露了可变的 `Domain` 对象，包括 `Domain.entities: HashSet<EntityId>`。用户可以直接修改哪些实体属于哪个域，完全绕过 `on_attach/on_detach` 生命周期钩子。

## 问题根因

从体系架构的角度，这是一个**"文档哲学"与"API 设计"脱节**的系统性问题：

- 文档反复强调"域即权威"、"边界分明"
- 而对持有 `World` 引用的调用层，框架自愿放弃了所有边界保护

这种不一致不只是实现细节，它动摇了整个框架的架构叙事：如果框架层本身不执行边界，那么在 `DomainContext` 层强制执行边界的意义何在？

## 影响程度

- [x] 中等（影响开发效率或理解，有变通方式）

> 注：框架的目标用户（仿真应用开发者）持有 `World` 引用，始终能绕过任何域边界。这不是"高级用户绕过"，而是正常使用路径下的默认行为。

## 建议方向

**收窄 `World` 的公开字段范围：**

1. `World.entities` 改为私有，提供受控的读写方法（类似 `DomainContext` 的两级接口）
2. `World.clock` 改为私有，只暴露 `sim_time()` 和 `step_count()` 只读访问
3. `World.domains` 若需公开，仅暴露只读访问（`&DomainRegistry`），去掉 `iter_mut()` 等写入接口

框架的 `spawn()`、`destroy()`、`register_domain()` 等高层操作已经存在，这些才是应该公开的接口。直接暴露原始字段等于绕开了这些精心设计的接口，失去了中间层提供的任何语义保证。
