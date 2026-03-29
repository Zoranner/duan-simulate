---
id: ISSUE-008
title: 域服务接口的定义与调用方式在文档和示例中均无完整代码示范
type: documentation
priority: p1-high
status: resolved
reporter: framework-consumer
created: 2026-03-27
updated: 2026-03-30
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

## 维护者评估（初次，2026-03-27）

**结论**：采纳。问题成立，代码中已有完整实现，但文档完全未展示调用语法，是显著的文档缺失。

（初次评估的行动计划已执行，但因方式二示例存在错误，Issue 被重新打开。）

---

## 追加说明（framework-consumer，2026-03-30）

**重新打开原因：文档中"方式二"示例代码存在具体编译错误。**

维护者在 `custom-domain.md` 补充的"方式二：按名称查找"示例如下：

```rust
if let Some(domain) = ctx.get_domain_by_name("faction") {
    if let Some(faction) = domain.rules.as_any().downcast_ref::<FactionRules>() {
        // ...
    }
}
```

但通过阅读 `src/domain.rs` 第 141-144 行可以确认，`ctx.get_domain_by_name<T>()` 是泛型方法，返回的是 `Option<&T>`——即已经转换好的具体类型，不存在 `.rules` 字段。上述代码**无法编译**。

若要获取原始 `&Domain` 再手动向下转型，需要调用的是 `ctx.get_domain_by_name_raw(name)`（第 150-152 行），它返回 `Option<&Domain>`。正确的方式二示例应为：

```rust
// 方式二：按名称查找（获取原始域引用，再手动向下转型）
if let Some(domain) = ctx.get_domain_by_name_raw("faction") {
    if let Some(faction) = domain.rules.as_any().downcast_ref::<FactionRules>() {
        // 调用服务方法...
    }
}
```

此外，既然 `get_domain_by_name<T>()` 本身已完成类型转换，实际上方式二和方式一的使用场景描述也需要重新澄清——两者的区别在于"按类型查找（有多实例时取最后注册的）"与"按名称精确定位特定实例"，而非"有无手动 downcast"。

**请求维护者修正文档示例，并重新校对两种方式的使用场景说明。**

---

## 维护者评估（再次，2026-03-30）

**结论**：采纳。Reporter 的追加说明完全正确，初次修复引入了错误示例，本次彻底修正。

**分析**：

Reporter 的技术指摘准确无误：

- `ctx.get_domain_by_name<T>(name)` 是泛型方法，返回 `Option<&T>`（已完成 downcast），不存在 `.rules` 字段——初次写入的示例用该方法返回值访问 `.rules.as_any()` 无法编译
- `ctx.get_domain_by_name_raw(name)` 才是返回 `Option<&Domain>` 的方法，用于需要访问原始域对象的场景

此外，初次修复对"两种方式"的场景说明存在误导——两者的本质区别是**按实现类型查找 vs 按名称精确定位**（后者用于同一类型多实例场景），而非"有无 downcast"。

本次修正一并处理 ISSUE-021 引起的 API 变更（`&ctx.entities` → `ctx.entities()`）。

**行动计划**：

- [x] 修正 `custom-domain.md` 探测域示例：方式二改为正确的 `ctx.get_domain_by_name::<T>(name)` 使用场景，`get_domain_by_name_raw` 作为独立的"元数据访问"用法单独说明
- [x] 重新澄清两种方式的使用场景：方式一（按类型）vs 方式二（按名称+类型，多实例时必用）
- [x] 更新 `concepts/domain.md` 域注册表章节，列出三种查找方式的方法名、返回类型和适用场景
- [x] 同步更新 `&ctx.entities` → `ctx.entities()` 的服务调用写法（ISSUE-021 联动）
