---
id: ISSUE-021
title: 域写入边界仅靠文档约定，框架无技术机制阻止越权修改
type: architecture
priority: p1-high
status: resolved
reporter: framework-consumer
created: 2026-03-30
updated: 2026-03-30
---

## 问题描述

ISSUE-009 在评估时，维护者将"写入边界由开发者自律维护，框架不强制检查"定性为"正常的设计决策"，但这一结论需要进一步审视。

`DomainContext.entities` 是公开字段，类型为 `&'a mut EntityStore`——这是对整个实体存储的完整可变引用，没有任何过滤或访问控制：

```rust
pub entities: &'a mut EntityStore,  // src/domain.rs 第 93 行
```

在 `compute()` 中，任何域都可以调用 `ctx.entities.get_mut(任意实体ID)` 并修改任意实体的任意组件，包括明确属于其他域管辖的实体。框架完全不阻止这种操作，也不产生任何警告或错误。

这与框架文档中反复强调的"域即权威（Domain-as-Authority）"核心设计哲学存在根本性矛盾：

- 文档说："域是它所管辖实体的权威，拥有完整的读写控制权"
- 实现现实：任何域都可以在 `compute()` 中越权修改其他域的管辖实体，且零代价

ISSUE-009 的维护者回复将这一现象描述为"合法的，全量只读遍历不破坏权威边界"——这对只读遍历是正确的，但"写入只靠自律"这个更深层的问题被混进了一起，并以"设计决策"的说辞一并关闭了，而没有得到独立的正视。

## 影响程度

- [x] 中等（影响开发效率或理解，有变通方式）

> 注：在单人或小团队开发时影响可控；但当多个开发者分别负责不同域时，任何一个域的越权修改都可能产生难以复现的仿真行为错误，且框架层面毫无线索。

## 复现场景

一个"优化版"的运动域可能顺手更新它认为"关联性很强"的碰撞组件：

```rust
impl DomainRules for MotionRules {
    fn compute(&mut self, ctx: &mut DomainContext) {
        for entity_id in self.own_entity_ids(ctx) {
            // 合法：更新自身管辖的位置组件
            if let Some(entity) = ctx.entities.get_mut(entity_id) {
                // ...
            }
            
            // 越权但合法：修改另一个域管辖的碰撞组件
            // 框架不会报错，仿真结果悄悄出错
            if let Some(entity) = ctx.entities.get_mut(some_collision_entity_id) {
                if let Some(col) = entity.get_component_mut::<CollisionState>() {
                    col.is_active = false; // 越权修改！
                }
            }
        }
    }
}
```

这种越权操作在编译期和运行时都不会触发任何提示，是一类静默失效的设计缺陷。

## 建议方案

**需架构讨论**：

这是一个有明确设计代价的问题，需要在以下几个方向中做取舍：

**方向一：运行时越权检测（debug 模式）**

在 `EntityStore::get_mut()` 或类似写入接口中，接受一个可选的"当前调用方域名"参数，在 `#[cfg(debug_assertions)]` 时检查目标实体的域归属，若实体归属域与调用方不符，`panic!` 或 `warn!` 告警。

优点：无 release 性能开销；能在开发阶段暴露越权写入。
缺点：API 改动较大；调用方传入当前域名需要 `DomainContext` 透传。

**方向二：收窄 DomainContext.entities 的写入接口**

提供两级接口：
- `ctx.entities`：只读，任何域均可使用（全量遍历）
- `ctx.own_entities_mut()`：可写，框架保证只返回当前域管辖实体的可变引用

将"只读全量访问"与"可变受限访问"在 API 层面分离，使越权写入在编译期就变成不可能。

缺点：可能增加 API 复杂度；需要调整现有代码中所有 `ctx.entities.get_mut()` 的调用。

**方向三：维持现状，但在文档中更诚实地表述**

如果框架决定维持现状，应在文档中明确说明：
1. "域即权威"是设计意图，不是技术约束
2. 写入越权不会被框架检测，是开发者的责任
3. 提供 Code Review 检查清单（如：`get_mut()` 调用应只针对 `ctx.own_entity_ids()` 返回的实体）

**短期可改进**：

无论采用哪个方向，当前文档中对"域即权威"的描述都应该更准确地区分"设计哲学约定"和"技术机制保障"，避免给开发者错误的安全感。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：采纳（方向二）。这是核心架构问题，已彻底修复，为破坏性变更（breaking change）。

**分析**：

Reporter 的判断准确：`pub entities: &'a mut EntityStore` 使任何域都可以无成本地越权修改其他域的管辖实体，与"域即权威"的核心哲学构成根本性矛盾。ISSUE-009 将此问题归入"合法设计"是不恰当的——只读遍历确实合法，但"写入只靠自律"这一更深层的漏洞不应被同等对待。

**对三个方向的评估**：

- 方向一（运行时检测）：只在 debug 模式保护，release 模式仍可越权，是补丁而非根治。
- 方向二（收窄接口）：从类型系统层面消除越权写入的可能性，与"域即权威"的哲学完全对齐，是彻底的解决方案。
- 方向三（诚实文档）：不解决问题，只是承认问题，不符合"不打临时补丁"的原则。

选择方向二，具体实现：

1. `DomainContext.entities: &'a mut EntityStore` 改为 `pub(crate)`——外部代码无法直接访问可变引用
2. 新增 `pub fn entities(&self) -> &EntityStore`——保留全量只读遍历能力
3. 新增 `pub fn get_own_entity_mut(&mut self, entity_id: EntityId) -> Option<&mut Entity>`——框架在返回可变引用前校验 `own_entities.contains(entity_id)`，越权访问返回 `None`

这是 breaking change：所有使用 `ctx.entities.get()` 的代码改为 `ctx.entities().get()`，所有使用 `ctx.entities.get_mut()` 的代码改为 `ctx.get_own_entity_mut()`。代价合理，因为这强制开发者在写入时明确意图，且从调用点即可看出是否符合权威边界。

**行动计划**：

- [x] `DomainContext.entities` 字段从 `pub` 改为 `pub(crate)`
- [x] 新增 `entities() -> &EntityStore` 只读访问方法
- [x] 新增 `get_own_entity_mut(entity_id) -> Option<&mut Entity>` 受限写入方法（含归属校验）
- [x] 更新 `docs/duan-docs/guides/custom-domain.md` 中所有示例代码
- [x] 更新 `docs/duan-docs/concepts/domain.md` 域上下文表格
- [x] 运行 `cargo clippy --all-targets --all-features -- -D warnings` 验证无新增警告
