---
id: ISSUE-018
title: dependencies() 使用字符串 key 声明依赖，拼写错误无法被编译期捕获
type: api-design
priority: p2-medium
status: open
reporter: framework-consumer
created: 2026-03-29
updated: 2026-03-29
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

**结论**：

**分析**：

**行动计划**：

**关闭理由**（如拒绝或 wontfix）：
