---
name: 框架设计模式
description: 在 Issue 处理中确认或发现的稳定设计模式，用于指导后续评估
type: project
---

## 权威域写入边界（已确认）

域在 compute 阶段可以直接修改自身管辖实体的组件状态——这是"域是权威拥有者"的直接体现，不需要绕道事件系统。只有生命周期操作（创建/销毁实体）才必须通过事件系统在事件处理阶段完成。跨边界通知（外部观察者反馈、跨域信息传递）也走事件系统，但这是通信机制而非写入限制。

**体现文档**（经 ISSUE-001、ISSUE-003 系统落实）：
- `docs/duan-docs/guides/custom-domain.md` 运动域参考实现的"产生事件"说明与"域的写入边界"章节
- `docs/duan-docs/concepts/event.md` 第96行事件处理器定义
- `docs/duan-docs/reference/glossary.md` DomainContext 和 EventHandler 词条
- `docs/duan-docs/architecture/philosophy.md` 事件驱动传播章节
- `docs/duan-docs/architecture/simulation-loop.md` 阶段二描述与"计算阶段的写入边界"约束

## 基础域与派生域的依赖模式（已确认）

- 基础域（运动域、空间域、阵营域）：无依赖，最先执行
- 计算域（探测域、战斗域）：依赖基础域，通过声明依赖关系保证执行顺序
- 协调域（威胁评估域）：依赖多个域，综合信息产出高层事件

执行顺序由域注册表根据依赖关系自动推导，非手动配置。

## 权威边界的读写非对称性（ISSUE-009 确认）

权威域架构的核心约束是**写入边界**，而非读取边界：

- 域只能**写入**自己管辖的实体组件状态（权威边界）
- 域可以**读取**任意实体（包括非管辖实体），不违反权威边界
- 域可以**查询**任意已注册域的服务接口（`ctx.get_domain::<T>()`）

这一非对称性与 `ctx.registry` 只读查询其他域服务的设计逻辑一致——读取信息不破坏权威。跨实体交叉计算（探测、战斗）依赖全量只读遍历，是框架设计的合法用法。

## 跨域服务调用的两种模式（ISSUE-008 确认）

在 `compute` 中调用其他域的服务：

- 方式一（推荐）：`ctx.get_domain::<T>()`，返回 `Option<&T>`，编译期类型安全，无需 downcast
- 方式二（动态场景）：`ctx.get_domain_by_name(name)` 返回 `Option<&Domain>`，再经 `domain.rules.as_any().downcast_ref::<T>()` 获取具体类型

`DomainRules` trait 强制要求所有实现类提供 `as_any` 方法，downcast 路径可行。方式二的使用场景：域名在运行时由配置决定，编译时不确定具体类型。

## Rust impl dyn Trait 与 FnMut 约束的 lifetime 交互陷阱（ISSUE-002、ISSUE-005 确认）

**陷阱一：impl dyn Trait 的隐式 'static**

在 `impl dyn SomeTrait { ... }` 块中定义方法时，若不显式标注生命周期，Rust 将 `dyn SomeTrait` 解析为 `dyn SomeTrait + 'static`，方法的 `&self` 参数的 object lifetime 隐式要求 `'static`。仅修复返回值生命周期标注（`(&'a self) -> Option<&'a T>`）不能消除 `self` 上的 `'static` object lifetime 要求。

**陷阱二：FnMut(&dyn Trait, ...) 约束中的 HRTB 展开**

`F: FnMut(&dyn SomeTrait, ...)` 中的 `&dyn SomeTrait` 按 Rust lifetime elision（RFC 599）展开为 `for<'r> FnMut(&'r (dyn SomeTrait + 'r), ...)`。闭包参数的 object lifetime 变为不确定的 `'r`，无法满足 `impl (dyn SomeTrait + 'static)` 上的方法调用要求，即使实际传入的是 `'static` object 也会触发 E0521。

**完整修复模式**：
- `impl dyn Trait` 上的方法需引入显式生命周期：`(&'a self) -> Option<&'a T>`（ISSUE-002 修复）
- 接受 `&dyn Trait` 的闭包约束需明确 object lifetime：`F: FnMut(&(dyn SomeTrait + 'static), ...)` 而非 `F: FnMut(&dyn SomeTrait, ...)`（ISSUE-005 待修复）
- 当且仅当约束侧与 `impl dyn Trait` 侧的 object lifetime 一致时，闭包中才能正常调用 `impl dyn Trait` 上的便捷方法。
