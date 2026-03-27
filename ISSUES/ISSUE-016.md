---
id: ISSUE-016
title: get_domain::<T>() 无法区分同类型多实例，阻碍多阵营域架构
type: api-design
priority: p2-medium
status: resolved
reporter: framework-consumer
created: 2026-03-27
updated: 2026-03-27
---

## 问题描述

`ctx.get_domain::<T>()` 按**类型**查找域实例，当同一类型被注册为多个具名实例时（如 `"red_command"` 和 `"blue_command"` 均为 `CommandRules`），返回值是不确定的（取决于注册顺序或内部遍历顺序），无法可靠地获取特定实例。

这使得"同一战术逻辑、不同阵营配置"的域架构无法通过一个类型实现，只能绕过去。

## 影响程度

- [ ] 阻塞性
- [x] 中等（影响开发效率或理解，有变通方式）
- [ ] 轻微

## 复现场景

在 `naval_combat` 中，红蓝双方各需要一个独立的指挥域实例（各自维护阵营级探测池和目标分配表）。自然的实现方式是：

```rust
world
    .with_domain("red_command", CommandRules::new(TacticProfile::aggressive()))
    .with_domain("blue_command", CommandRules::new(TacticProfile::defensive()))
```

然后在 `SteeringRules::compute` 中，根据实体的阵营查询对应实例：

```rust
// 期望写法（目前无法实现）
let cmd = ctx.get_domain_by_name::<CommandRules>("red_command")?;
```

由于 `get_domain::<T>()` 不支持按名字区分，实际只能用新建独立类型的方式绕过：

```rust
pub struct RedCommandRules(pub CommandRules);
pub struct BlueCommandRules(pub CommandRules);
// 各自实现 DomainRules，仅 try_attach 不同，其余完全委托给内部 CommandRules
```

这引入了不必要的类型噪声，且随着阵营数量增加会线性膨胀。

## 建议方案

**短期可改进**：

在 `DomainContext` 上增加按名字查找的 API：

```rust
// 按域注册名查找，返回指定类型的引用
ctx.get_domain_by_name::<CommandRules>("red_command") -> Option<&CommandRules>
ctx.get_domain_by_name_mut::<CommandRules>("red_command") -> Option<&mut CommandRules>
```

框架内部已知每个域的注册名（`with_domain("name", ...)` 时记录），查找时先过滤名字、再进行类型转换即可。

**需架构讨论**：

是否应区分"域类型"和"域实例名"的概念？当前依赖声明 `dependencies()` 返回 `Vec<&str>`（实例名），而查询 `get_domain::<T>()` 用类型。两者是不同维度的标识符，混用容易产生语义歧义。可考虑统一为名字查找，或提供两种查找方式并明确各自语义。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：部分采纳（accept-doc + 小范围 API 补全）

**分析**：

经过代码审查，问题存在，但情况比描述的更具体：

**问题一：类型索引覆盖行为**

`DomainRegistry` 的 `type_index` 是 `HashMap<TypeId, String>`，当同一类型注册两个实例时（如两次 `register::<CommandRules>`），后者会静默覆盖前者的索引条目。`get::<T>()` 始终返回最后注册的那个实例。这不是"不确定"，而是"确定地错误"——前一个实例永远无法通过类型查找到，但它仍被存储在 `domains` 中，占用资源。这是一个真实的静默错误，应当在文档中明确警告，或在 `register` 时检测并 panic/warn。

**问题二：按名查找的 API 已存在但不完整**

`DomainContext::get_domain_by_name(name)` 已在实现中存在（`domain.rs` 第 137 行），返回 `Option<&Domain>`。Reporter 期望的 `get_domain_by_name::<T>(name) -> Option<&T>` 泛型版本尚未提供，需要手动 `domain.rules.as_any().downcast_ref::<T>()`，使用负担较高。文档（`domain.md` 第 196 行）已记录了这种两步用法，但作为 API 设计而言，缺少泛型便利版本是一个值得补全的遗漏。

**问题三：`dependencies()` 用名字、`get_domain()` 用类型的混用**

这是一个真实的设计张力，但不是设计错误。两者服务于不同目的：`dependencies()` 描述执行顺序约束，是面向注册名的拓扑关系；`get_domain::<T>()` 是面向类型的服务查询，编译期安全。两者标识符维度不同，但各自语义是清晰的。当前文档未明确解释这一张力的来源和选择理由，导致 reporter 认为这是"语义歧义"。实际上框架对两者都支持，这是设计上的兼容而非混用。需要在文档中补充说明。

**问题四：场景真实性**

Reporter 描述的 `RedCommandRules`/`BlueCommandRules` 绕道方式是当前实际情况下不得不为的做法。但从架构角度审视，naval_combat 的 `CommandRules` 实际上已经通过内部 `HashMap<u8, ...>`（按 team 分组）处理了多阵营问题，并不需要注册两个 `CommandRules` 实例。Reporter 所描述的"阵营配置不同"场景（如 `TacticProfile::aggressive()` vs `defensive()`）目前在 naval_combat 中并未实现，是一个假设场景。因此这不是当前代码的阻塞性问题，而是一个中期需要支持的场景。

**行动计划**：

1. **文档补充（高优先级）**：在 `domain.md` 的"域注册表"一节补充警告：同一类型不能注册多个实例，`get_domain::<T>()` 在此场景下行为是未定义的（静默使用最后注册者）。明确说明两种查找方式的使用场景和边界。

2. **文档补充（中优先级）**：在 `domain.md` 中明确解释 `dependencies()` 用名字、`get_domain()` 用类型这一设计张力，说明两者是服务于不同目的的标识符维度，消除认知歧义。

3. **API 补全（中优先级）**：在 `DomainContext` 上增加泛型便利方法：
   ```rust
   pub fn get_domain_by_name<T: DomainRules>(&self, name: &str) -> Option<&T> {
       self.registry.get_by_name(name)
           .and_then(|d| d.rules.as_any().downcast_ref::<T>())
   }
   ```
   这是对已有能力的便利封装，不引入新概念，负担小。同时考虑在 `DomainRegistry::register` 中增加同类型重复注册的检测（debug_assert 或 log warn）。

4. **不采纳的部分**：Reporter 提议"统一为名字查找"或"统一为类型查找"。这两个方向都不宜采纳：统一为名字查找会丢失编译期类型安全；统一为类型查找无法支持多实例场景。维持两种查找方式并明确各自语义是更合理的方向。
