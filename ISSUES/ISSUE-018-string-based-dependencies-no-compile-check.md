---
id: ISSUE-018
title: dependencies() 使用字符串 key 声明依赖，拼写错误无法被编译期捕获
type: api-design
priority: p2-medium
status: resolved
reporter: framework-consumer
created: 2026-03-29
updated: 2026-03-30
---

## 问题描述

`DomainRules::dependencies()` 通过返回 `Vec<&'static str>` 来声明执行顺序依赖：

```rust
fn dependencies(&self) -> Vec<&'static str> {
    vec!["motion", "detection", "faction"]
}
```

这些字符串与 `World::with_domain("name", ...)` 注册时使用的名字进行匹配。整个依赖声明机制完全依赖**运行时字符串比对**，而非任何编译期约束。

这意味着：

1. **拼写错误完全静默**：写成 `"detectoin"` 或 `"facton"` 不会报任何错误，只会使依赖声明失效，排序时被当作"无依赖"处理。这类 bug 极难发现——域仍然能运行，只是可能在某些帧内先于它依赖的域执行，产生偶发性计算结果异常。

2. **重构时缺乏保护**：域名修改（如将 `"detection"` 改为 `"detect"`）时，所有依赖它的域的 `dependencies()` 返回值都需要手动同步更新。IDE 的重命名重构功能对字符串字面量无法自动覆盖。

3. **IDE 无法跳转/索引**：在 VSCode 或 CLion 中，无法通过 `"motion"` 字符串跳转到对应域的注册位置，也无法通过"查找所有引用"找到谁依赖了某个域。

在 `taishixum-app` 的 `tracking.rs` 中：

```rust
fn dependencies(&self) -> Vec<&'static str> {
    vec!["motion"]
}
```

这与 `scenario/mod.rs` 里的 `.with_domain("motion", MotionRules::new())` 配对。如果有人把域名改为 `"movement"`，`TrackingRules` 的依赖就会静默失效，轨迹历史在运动更新前就被记录——产生"总是落后一帧"的位置漂移 bug，极难定位。

## 影响程度

- [ ] 阻塞性
- [x] 中等（影响开发效率或理解，有变通方式）
- [ ] 轻微

## 复现场景

1. 新建一个域，在 `dependencies()` 中引用另一个域名
2. 将被依赖的域的注册名稍作修改
3. 编译通过，运行时无警告，但执行顺序已不正确

## 建议方案

**短期可改进**：

在 `World::build()` 或首次 `step()` 时，增加依赖声明合法性校验：

```rust
// 伪代码：在 World 构建完成后，验证所有 dependencies() 返回的名字都已注册
fn validate_dependencies(&self) {
    for (name, domain) in &self.domains {
        for dep in domain.rules.dependencies() {
            if !self.domains.contains_key(dep) {
                panic!("域 '{}' 声明依赖 '{}'，但该域未注册", name, dep);
            }
        }
    }
}
```

至少能在运行时第一帧就暴露问题，而不是静默失效。

**需架构讨论**：

提供类型安全的依赖声明方式：

```rust
// 方案 A：使用类型而非字符串
fn typed_dependencies(&self) -> Vec<TypeId> {
    vec![TypeId::of::<MotionRules>(), TypeId::of::<DetectionRules>()]
}

// 方案 B：提供辅助宏
fn dependencies(&self) -> Vec<&'static str> {
    domain_deps![MotionRules, DetectionRules]
    // 宏展开为：将类型映射到注册名的编译期或运行期查找
}
```

这需要框架内部建立"类型 → 注册名"的双向映射，有一定工程成本，但能彻底消除字符串拼写风险，并使 IDE 的跳转和引用追踪成为可能。

---

<!-- 以下由 core-maintainer 填写，reporter 不要修改 -->

## 维护者评估

**结论**：采纳（短期运行时校验）；类型安全依赖声明（方案 A/B）不采纳，原因见下

**分析**：

问题描述准确。通过阅读 `src/domain.rs` 的 `compute_execution_order()` 实现（第 329-333 行）可以确认：拓扑排序时，若 `dependencies()` 返回的字符串在 `domains` 中不存在，`domains.get(name)` 返回 `None`，**循环直接跳过，不产生任何错误**。这意味着拼写错误的依赖名称会让依赖声明静默失效，域可能在依赖域之前执行，产生难以定位的帧内计算顺序 bug。

这是一个需要修复的实现缺陷，而非设计层面的取舍。

**为何不采纳类型安全方案（方案 A：TypeId）**：

`philosophy.md` 明确将"域标识使用字符串"列为框架的设计原则之一，原因是支持运行时注册和多实例场景。TypeId 方案存在本质局限：当同一 `DomainRules` 类型注册了多个实例（如两个不同配置的 `MotionRules`），TypeId 无法区分——`dependencies()` 无法表达"依赖名称为 'motion_fast' 的那个运动域"。引入 TypeId 依赖会强迫框架放弃多实例支持，这是不可接受的代价。

方案 B（辅助宏）本质上仍依赖字符串，需额外维护"类型 → 注册名"映射，工程复杂度大于收益。

**正确的修复方向**：

在 `compute_execution_order()` 或 `DomainRegistry::execution_order()` 的首次计算中，增加依赖合法性校验：若某域声明的依赖名称未出现在已注册域列表中，立即 `panic!` 或至少 `eprintln!` 警告。这能在仿真第一帧就暴露问题，而不是静默失效。

**行动计划**：

- [x] 在 `DomainRegistry::compute_execution_order()` 的 `visit` 函数内，对每个 `dep` 名称校验是否存在于 `self.domains`；若不存在，`panic!("域 '{}' 声明依赖 '{}'，但该域未注册", name, dep)` 确保立即暴露拼写错误
  - 实现位置：`src/domain.rs`，`compute_execution_order` 内嵌的 `visit` 闭包
  - 校验在 `World::build()` 时（通过 `WorldBuilder::build` 调用 `execution_order()`）即触发，在配置阶段就能发现问题，而不是等到首帧才发现
  - 新增测试：`test_dependency_validation_passes_for_registered_deps` 和 `test_dependency_validation_panics_for_missing_dep`（`#[should_panic]`）
