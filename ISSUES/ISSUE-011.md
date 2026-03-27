---
id: ISSUE-011
title: 域服务方法需要显式传入 &EntityStore——调用侧啰嗦，难以封装
type: api-design
priority: p2-medium
status: open
reporter: framework-consumer
created: 2026-03-27
updated: 2026-03-27
---

## 问题描述

域的服务方法（即可被其他域在 `compute` 中调用的只读查询方法）目前需要调用方显式传入 `&EntityStore`：

```rust
// SpaceRules
pub fn distance(&self, id_a: EntityId, id_b: EntityId, entities: &EntityStore) -> Option<f64>

// FactionRules
pub fn is_hostile(&self, id_a: EntityId, id_b: EntityId, entities: &EntityStore) -> bool
```

调用侧写法：

```rust
let space = ctx.get_domain::<SpaceRules>().unwrap();
let dist = space.distance(a, b, &ctx.entities);

let faction = ctx.get_domain::<FactionRules>().unwrap();
let hostile = faction.is_hostile(a, b, &ctx.entities);
```

每次调用都要拖一个 `&ctx.entities` 参数，在服务方法较多或调用链较深时显得冗余。

根本原因：域服务方法只有 `&self`，没有访问 `DomainContext` 的途径，所以必须由调用方将 `EntityStore` 传进来。这是当前架构下唯一合法的模式。

## 影响程度

- [ ] 阻塞性
- [x] 中等（影响开发效率或理解，有变通方式）
- [ ] 轻微

## 复现场景

开发 `DetectionRules::compute` 时，需要同时调用 `SpaceRules::entities_in_range` 和 `FactionRules::is_hostile`，两次调用都要传 `&ctx.entities`。若后续还有 3~4 个服务方法，代码的视觉噪声会明显增加。

## 建议方案

**短期可改进**：

在 `guides/custom-domain.md` 中将"服务方法签名约定"作为显式模式记录，说明：
- 为什么服务方法必须接受 `&EntityStore` 参数
- 推荐的命名和参数顺序约定（`&self, id_a, id_b, ..., entities: &EntityStore`）

**需架构讨论**：

考虑为"服务调用"提供轻量包装，例如引入 `DomainService<'a>` 结构，在 `ctx.get_domain_service::<T>()` 时同时捕获 `&ctx.entities`，使调用方无需每次传 `entities`。但这涉及生命周期设计，需要评估是否值得增加复杂度。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：

**分析**：

**行动计划**：

- [ ]

**关闭理由**（如拒绝或 wontfix）：
