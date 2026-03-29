---
name: 整体设计审计（2026-03-30）
description: 对 DUAN 框架整体设计的全面架构审计，基于完整源码分析
type: project
---

## 审计范围

基于完整源码（component.rs / entity.rs / domain.rs / events.rs / world.rs）和全部 ISSUE 历史（ISSUE-001 ~ ISSUE-023），对框架设计做全维度审计。

## 整体评分：6/10

**优点**：核心概念模型清晰（Entity/Component/Domain/Event 四层分离）；5 阶段仿真循环设计合理；拓扑排序执行顺序机制正确；事件驱动跨边界通信设计自洽。

**短板**：World 层封装全面崩溃；多个 API 存在静默误导；trait 设计被调试辅助方法污染；一处未完成功能遗留死代码。

## 最严重的 5 个问题

### P1: World 字段全 pub 使 DomainContext 的权威边界形同虚设（ISSUE-024）
ISSUE-021 花了大力气保护 DomainContext.entities，但 World.entities 始终是 pub，任意调用方一句 world.entities.get_mut(id) 即可绕过所有保护。这是架构叙事与 API 设计之间最严重的矛盾。同样，world.clock.sim_time 可随意篡改（破坏可复现性），world.domains.iter_mut() 可直接修改域实体集合（绕过 on_attach/on_detach）。

### P1: Entity::add_domain/remove_domain post-spawn 静默无效（ISSUE-025）  
spawn 后调用 add_domain()，只修改 Entity.domains HashSet，DomainRegistry 感知不到，实体不会被附加到新域。这是一个 API 语义与实际行为完全背离的陷阱，且无任何运行时提示。

### P2: DomainEvent::EntitySpawned 是死代码（ISSUE-026）
框架 spawn() 从不 emit 此变体，handle_event 对此变体是空操作。公开枚举中的死变体会让用户编写永不执行的监听代码。

### P2: on_attach/on_detach 设计不对称无文档依据（ISSUE-027）
on_attach 有默认空实现，on_detach 无默认实现（必须实现）。不对称设计背后可能有"必须清理引用"的理由，但未文档化，靠编译失败让用户发现。

### P2: component_type() 是 trait 必须方法但框架不使用（ISSUE-028）
Component trait 要求实现 component_type() -> &'static str，但框架内部用 TypeId 查找组件，此字符串仅供调试/序列化，且无唯一性保证。核心 trait 被辅助方法污染。

## unsafe 消除路径（供参考）

compute_domains() 中 unsafe 的根因是需要同时持有 &mut domain.rules 和 &self.domains（用于其他域的读访问）。可行的安全 Rust 替代方案：take/restore 模式——执行前将当前域的 rules 从 HashMap 中 remove，执行后 insert 回去。代价是每域每帧各一次 HashMap 操作，对小型域集合可接受。但 get_domain<CurrentDomain>() 在 compute 期间会返回 None（当前域已被 take 出来），需要评估是否可接受。

## 关于 ISSUE-017（域不能 spawn/destroy）的架构评价

维护者的决策（保留约束，文档说明两条理由：可追溯性 + 并行化安全）在架构上有依据。"SpawnCommand 事件模式"是合理的长期方向，使域能够"表达意图"而非"直接执行"，符合框架的事件驱动哲学。不建议重新开启此讨论，除非有具体的实际需求推动。

## 关于 get_own_entity_mut() 返回 None 的评价

越权访问返回 None 而非 panic，从架构角度是可讨论的：越权写入是编程错误（bug），而非运行时异常，应该在开发阶段 panic 暴露。建议至少在 debug_assertions 模式下 panic，release 模式保持 None。这是一个值得向 core-maintainer 提出的改进点（但与现有 ISSUE 相关，不单独开 ISSUE）。
