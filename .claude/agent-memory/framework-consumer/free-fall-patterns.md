---
name: free_fall 示例设计模式
description: 自由落体示例的双域架构、组件设计及已知的框架使用约束
type: project
---

## 示例位置

`examples/free_fall/`

## 架构：双域设计

- **motion 域**：无依赖，最先执行。半隐式欧拉积分，直接写 Position/Velocity 组件
- **collision 域**：依赖 motion，后执行。穿越检测 + 弹跳响应 + 发出 GroundCollisionEvent

## 关键组件分工

- `Position` / `Velocity` / `Mass`：物理状态
- `Collider`：碰撞参数（name, offset_y, restitution, friction）
- `StaticBody`：零大小标记，显式声明静态体，避免用"缺少 Velocity"的隐式推断

## 实体声明模式

实体通过 `.with_domain("name")` 声明归属，域通过 `try_attach` 中的组件检查决定是否接纳。
地面只加 collision 域，小球同时加 motion + collision 域。

## 仿真主循环模式

两阶段：Phase 1 全速推进缓存 `RenderFrame`，Phase 2 按 sim_time 时间戳定时回放。
终止条件用具名常量（REST_HEIGHT_THRESHOLD / REST_VELOCITY_THRESHOLD）而非魔法数字。

## 已知框架约束

`CustomEvent::downcast::<T>()` 在 `step_with` 闭包中因生命周期问题无法编译（ISSUE-002）。
必须用 `event.as_any().downcast_ref::<T>()` 替代。
