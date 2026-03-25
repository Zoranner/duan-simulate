# DUAN 仿真体系文档

DUAN 是一个通用的仿真体系框架，采用**域驱动设计（Domain-Driven Design）**，以域（Domain）为核心计算单元，构建可扩展的仿真平台。

## 文档目录

### 介绍

- [概览](./introduction/overview.md) - 什么是 DUAN 仿真体系
- [设计目标](./introduction/goals.md) - 框架的设计目标和核心理念
- [适用场景](./introduction/use-cases.md) - 典型应用场景

### 核心概念

- [设计哲学](./concepts/philosophy.md) - 域驱动设计的核心思想
- [实体](./concepts/entity.md) - 仿真对象的基本单元
- [组件](./concepts/component.md) - 实体的数据组成
- [域](./concepts/domain.md) - 权威计算单元
- [域注册表](./concepts/domain-registry.md) - 域的管理与调度
- [事件](./concepts/event.md) - 域间通信机制
- [时间](./concepts/time.md) - 仿真时间管理
- [生命周期](./concepts/lifecycle.md) - 实体的状态变迁

### 架构

- [架构概览](./architecture/overview.md) - 系统整体结构
- [仿真循环](./architecture/simulation-loop.md) - 每帧的执行流程
- [世界](./architecture/world.md) - 仿真世界的容器

### 使用指南

- [场景配置](./guides/scenario.md) - 如何配置仿真场景
- [自定义域](./guides/custom-domain.md) - 如何实现自定义域
- [自定义组件](./guides/custom-component.md) - 如何定义组件

### 示例参考

> **注意**：本节的域和组件仅为示例，展示如何使用框架。实际使用时，用户应根据具体仿真场景自行定义和实现。

- [示例域](./examples/domains.md) - 常见域的参考实现
- [示例组件](./examples/components.md) - 常见组件的参考设计

### 参考

- [术语表](./reference/glossary.md) - 核心术语定义
- [设计评审](./reference/design-review.md) - 已识别的设计问题和改进建议

---

## 快速了解

DUAN 仿真体系的核心理念可以概括为：

1. **域是权威** - 每个领域有一个权威的域来负责计算和判定
2. **域运行时定义** - 框架不预设域类型，用户在运行时注册
3. **实体自声明归属** - 实体声明自己要加入哪些域
4. **事件驱动传播** - 域的计算结果通过事件系统传播

更多信息请参阅 [设计哲学](./concepts/philosophy.md)。
