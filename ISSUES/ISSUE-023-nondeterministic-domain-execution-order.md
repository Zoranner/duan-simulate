---
id: ISSUE-023
title: 同层无依赖域的执行顺序不确定，影响仿真可复现性
type: architecture
priority: p1-high
status: open
reporter: framework-consumer
created: 2026-03-30
updated: 2026-03-30
---

## 问题描述

`DomainRegistry::compute_execution_order()` 的拓扑排序实现（`src/domain.rs` 第 344-352 行）使用如下外层循环作为排序起点：

```rust
for name in self.domains.keys() {
    visit(name, &self.domains, &mut visited, &mut temp_mark, &mut order);
}
```

`self.domains` 的类型是 `HashMap<String, Domain>`。Rust 的 `HashMap` 不保证迭代顺序——即使在相同的编译版本和操作系统下，不同运行之间的迭代顺序也可能因哈希随机化而不同（Rust 默认启用 hashmap 随机种子以防范 Hash DoS 攻击）。

**直接后果**：对于没有依赖关系的"同层域"，拓扑排序的访问起点是随机的，导致它们的执行顺序在不同运行中可能不同。

示例：若注册了 `motion`、`detection`、`faction` 三个无相互依赖的域，理论上它们的执行顺序可能是：
- 运行 A：`motion` → `detection` → `faction`
- 运行 B：`faction` → `motion` → `detection`
- 运行 C：`detection` → `faction` → `motion`

## 影响程度

- [x] 中等（影响开发效率或理解，有变通方式）

> 注：在域之间完全无共享状态读写的理想情况下，执行顺序不影响结果。但在实际仿真中，同层域之间往往存在"读取对方已更新的组件"的隐式时序依赖，执行顺序的不确定性会导致仿真结果在不同运行之间出现细微差异，使 bug 的复现变得困难。
>
> 对于仿真系统而言，"给定相同输入，得到相同输出"通常是核心需求（如用于回放、对比实验、自动化测试）。当前的不确定性威胁这一目标。

## 复现场景

在 `taishixum-app` 中，如果 `TrackingRules` 和 `CombatRules` 都没有对对方声明依赖，它们在不同运行中可能以不同顺序执行。如果 `TrackingRules` 依赖"当帧 `CombatRules` 已更新敌方状态"，执行顺序不同将导致追踪历史记录出现"落后一帧"的位置漂移——而这个 bug 在某些运行中出现，在另一些运行中消失，极难定位。

## 建议方案

**短期可改进**：

将外层循环的遍历顺序固定化。最简单的方案是对域名排序后再遍历：

```rust
let mut names: Vec<&str> = self.domains.keys().map(|s| s.as_str()).collect();
names.sort();  // 字典序固定遍历起点
for name in names {
    visit(name, &self.domains, &mut visited, &mut temp_mark, &mut order);
}
```

这能保证：给定相同的域注册顺序和依赖关系，每次计算出的执行顺序都是确定的。代价极小（只有初始化阶段的一次排序，运行时不影响性能）。

**需架构讨论**：

**是否应该允许开发者显式控制同层域的执行顺序？**

当前框架的执行顺序完全由 `dependencies()` 声明决定，没有"优先级"或"提示顺序"的概念。如果开发者需要控制同层域的执行顺序，唯一方式是添加显式依赖——但这会引入语义上不真实的依赖，只是为了控制执行顺序。

一个可选的增强是提供"执行顺序提示"（hint，不是约束），但这是较大的 API 设计变更，可以作为长期议题。短期的确定性修复（字典序）应优先。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：

**分析**：

**行动计划**：

- [ ] 

**关闭理由**（如拒绝或 wontfix）：
