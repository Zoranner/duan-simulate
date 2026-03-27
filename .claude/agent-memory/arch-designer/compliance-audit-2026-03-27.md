---
name: 框架实现合规审计（2026-03-27）
description: 对 src/ 和 examples/free_fall/ 的首次完整合规审计，整体高度合规
type: project
---

## 审计结论概要

整体合规度高，无严重偏差。发现1处文档与实现不符（已修正），1处有意味的实现细节值得持续关注。

## 符合设计哲学的关键点

- Entity：纯数据容器，无业务逻辑方法，only 组件容器 + 生命周期状态
- Component：trait 强制纯数据，无 impl 方法暴露行为（as_any 系列为内部类型转换机制，不是业务行为）
- Domain：DomainRules trait 分离 try_attach（纯谓词）和 on_attach（副作用），符合文档设计
- 仿真循环：world.rs 严格遵守5阶段顺序（时间推进→域计算→定时器→事件处理→清理）
- 拓扑排序：compute_execution_order 用 temp_mark 检测循环依赖，构建阶段立即验证
- 事件通道：只追加设计（push/drain），计算阶段不消费，符合设计
- DomainContext：own_entities 只读、entities 可变、registry 只读、events 只写、clock 只读，完全符合文档权限表

## 需持续关注的实现细节

**unsafe 块（world.rs 第301-320行）**：compute_domains 用裸指针分离 rules 和 own_entities 的借用，以满足 Rust 借用检查器。这是绕过借用检查的局部 unsafe，注释中有 SAFETY 说明，理由充分（两字段不存在别名）。这不是设计哲学偏差，但属于实现层的复杂性，日后如有重构机会可考虑消除。

**process_events 中 handle_event 的调用顺序（world.rs 第346-349行）**：自定义事件先调用用户 handler，再调用框架内置 handle_event。对于 Custom 变体，handle_event 什么也不做，顺序无实质影响，但代码结构上 Custom 事件被 handle_event 再 match 一次——轻微冗余，不是偏差。

## 示例合规分析

- MotionRules：直接修改组件状态，无状态域（符合"域直接写入"原则）
- CollisionRules：维护 prev_y 和 ground_id 跨帧内部状态，计算后直接修改组件并 emit 事件（符合文档描述的"混合型域"模式）
- 碰撞域的 on_attach 用于缓存 ground_id，而非探测：try_attach 确实是纯谓词，符合"try_attach 无副作用"要求
- Mass 组件注册但 MotionRules 未使用：不是设计问题，只是示例简化（重力加速度对所有物体相同，无需质量）

**Why**: 持续关注 unsafe 块，是因为将来如果框架 API 有重构，应优先考虑消除这段 unsafe 以降低维护风险。
**How to apply**: 如果收到关于 world.rs compute_domains 的合规或实现 Issue，结合此记录判断。
