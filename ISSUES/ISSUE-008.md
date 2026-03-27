---
id: ISSUE-008
title: 域服务接口的定义与调用方式在文档和示例中均无完整代码示范
type: documentation
priority: p1-high
status: fixed
reporter: framework-consumer
created: 2026-03-27
updated: 2026-03-27
---

## 问题描述

`domain.md` 和 `custom-domain.md` 多次提到域可以"提供服务接口"供其他域查询，并在参考实现中列举了阵营域、探测域等服务型域。但整个文档体系中，**没有任何代码示例**展示：

1. 服务接口的定义方式（是直接在 `struct` 上实现方法？还是需要实现某个 trait？）
2. 域在 `compute` 中如何通过 `ctx.registry` 拿到另一个域的引用
3. 拿到引用后如何调用其服务方法（涉及类型系统——`ctx.registry` 返回的是 `dyn DomainRules`，如何向下转型到具体域类型？）

在规划舰队对抗示例时，我需要让探测域在 `compute` 中调用阵营域的 `is_hostile(a, b)` 服务。但我完全不知道：

- `ctx.registry.get_domain("faction")?` 返回什么类型？是 `&dyn DomainRules` 吗？
- 如果是 `&dyn DomainRules`，需要 `downcast_ref::<FactionRules>()` 吗？
- 还是 `ctx.registry.get_domain_as::<FactionRules>()` 有类型安全的泛型版本？

文档中提到两种查找方式："按实现类型查找（编译期类型安全，推荐）"和"按名称字符串查找"，但没有任何代码示例说明这两种方式的实际语法是什么。

## 影响程度

- [x] 中等（影响开发效率或理解，有变通方式）

## 复现场景

规划"探测域依赖阵营域"的实现步骤时：

```rust
// 假设的探测域 compute 实现——但我不知道 get_domain_as 是否存在，签名是什么
fn compute(&mut self, ctx: &mut DomainContext) {
    let faction_domain = ctx.registry.get_domain_as::<FactionRules>("faction")?;

    let entity_ids: Vec<EntityId> = ctx.own_entity_ids().collect();
    for observer_id in &entity_ids {
        for target_id in ctx.entities.all_active_ids() {  // 这个方法存在吗？
            if faction_domain.is_hostile(*observer_id, target_id) {  // 能这样调用吗？
                // 探测判定...
            }
        }
    }
}
```

每一行都充满了对框架 API 的猜测。这是开发多域协作示例时必须解决的基础问题，但文档完全没有覆盖。

## 建议方案

**短期可改进**：

在 `guides/custom-domain.md` 的"参考实现"章节，在探测域参考实现后面补充**完整的跨域服务调用代码片段**，至少包含：

```rust
// 方式一：按类型查找（编译期类型安全，推荐）
let faction = ctx.registry
    .get_domain::<FactionRules>()
    .expect("探测域依赖阵营域，但阵营域未注册");

if faction.is_hostile(observer_id, target_id) { ... }

// 方式二：按名称查找（动态场景）
let faction = ctx.registry
    .get_domain_by_name("faction")
    .and_then(|d| d.as_any().downcast_ref::<FactionRules>());
```

同时，在 `domain.md` 的"域注册表"章节补充：两种查找方式对应的方法名和返回类型（哪怕只是伪代码级别的签名），明确"按实现类型查找"的方法名是什么。

**需架构讨论**：

`ctx.entities` 目前只暴露管辖域内的实体，但探测域需要"遍历所有潜在目标"（包括不在探测域中的实体）。文档中没有提及如何从 `ctx.entities` 或其他接口获取全量活跃实体列表。这可能是一个更深层的设计问题，值得专门讨论（可能需要 ISSUE-009 单独追踪）。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：采纳。问题成立，代码中已有完整实现，但文档完全未展示调用语法，是显著的文档缺失。

**分析**：

经核查实现代码（`src/domain.rs`），`DomainContext` 上已存在两个便捷方法：

- `ctx.get_domain::<T>()` —— 按实现类型查找，返回 `Option<&T>`，直接得到具体类型，无需 downcast（第 130–132 行）
- `ctx.get_domain_by_name(name)` —— 按名称查找，返回 `Option<&Domain>`（第 137–139 行）；若要从 `&Domain` 获取具体类型，需经 `domain.rules.as_any().downcast_ref::<T>()`

Reporter 猜测的 `ctx.registry.get_domain_as::<T>()` 不存在，但 `ctx.get_domain::<T>()` 已实现相同效果。`DomainRules` trait 要求所有实现类提供 `as_any` 方法（第 73–78 行），名称查找路径的向下转型因此是可行的。

文档问题：
1. `domain.md` 的"域注册表"章节只提到"支持两种查找方式"，但没有说明具体是 `ctx.get_domain::<T>()` 和 `ctx.get_domain_by_name()`，也没有代码示例
2. `custom-domain.md` 的探测域参考实现完全没有展示跨域服务查询的写法

这是 reporter 困惑的直接来源，不是 API 设计问题。

**行动计划**：

1. 在 `guides/custom-domain.md` 的探测域参考实现后补充"跨域服务调用"代码片段，展示：
   - `ctx.get_domain::<T>()` 的标准用法（方式一，推荐）
   - `ctx.get_domain_by_name()` + `as_any().downcast_ref::<T>()` 的用法（方式二，动态场景）
2. 在 `concepts/domain.md` 的"域上下文"章节，在 `registry` 行的描述中补充具体方法名

**关闭理由**（如拒绝或 wontfix）：
