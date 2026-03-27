---
id: ISSUE-013
title: compute 中两相借用模式缺乏文档说明——新用户容易踩坑
type: documentation
priority: p2-medium
status: open
reporter: framework-consumer
created: 2026-03-27
updated: 2026-03-27
---

## 问题描述

在 `DomainRules::compute` 中，若需要遍历实体并对其进行可变操作，由于 Rust borrow checker 的约束，必须采用"两相借用"模式：

```rust
fn compute(&mut self, ctx: &mut DomainContext) {
    // 第一阶段：只读借用，收集需要处理的实体 ID
    let ids: Vec<EntityId> = ctx.own_entity_ids().collect();

    // 释放不可变借用后，第二阶段：用 ID 再去可变访问
    for id in ids {
        let entity = ctx.entities.get_mut(id); // 或通过 ID 做其他操作
        // ...
    }
}
```

直接写成如下形式会编译失败：

```rust
for entity in ctx.own_entities() {  // 持有不可变引用
    ctx.entities.update_something(entity.id, ...);  // 试图可变借用 → 编译错误
}
```

这个模式在 `free_fall` 示例中有出现，`naval_combat` 里也多处用到，但**文档中完全没有提及**。

对于熟悉 ECS 或游戏引擎开发的用户来说这是常识，但对于从系统仿真背景进入的用户，第一次遇到编译错误时会花费大量时间排查。

## 影响程度

- [ ] 阻塞性
- [x] 中等（影响开发效率或理解，有变通方式）
- [ ] 轻微

## 复现场景

开发任何需要在 `compute` 中同时读取和更新实体状态的域时都会遇到。例如 `MotionRules` 需要先读取 Position/Velocity，再写回更新后的 Position。

## 建议方案

**短期可改进**：

在 `guides/custom-domain.md` 的"编写 compute 方法"部分，增加一个"两相借用模式"小节，内容包括：

1. 说明为什么需要两阶段（Rust 不允许同时持有可变和不可变引用）
2. 给出标准写法模板：
   ```rust
   // 推荐：先收集 ID，再按 ID 操作
   let ids: Vec<EntityId> = ctx.own_entity_ids().collect();
   for id in ids {
       // 现在可以安全地 get / get_mut
   }
   ```
3. 说明这是使用该框架的惯用模式，不是 bug

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：

**分析**：

**行动计划**：

- [ ]

**关闭理由**（如拒绝或 wontfix）：
