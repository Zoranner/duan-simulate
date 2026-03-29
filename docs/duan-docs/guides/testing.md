# 测试域逻辑

本文档描述如何在标准 `#[test]` 中测试仿真域逻辑，无需启动完整应用。

## 核心能力

`World` 可以直接在测试中构建和使用——不依赖 Tauri、前端或任何 IO 设施。测试的关键能力是：

- **执行并收集事件**：`world.step_collect(dt)` 执行一帧并返回本帧产生的所有自定义事件
- **读取组件状态**：通过 `world.get_entity(id)` → `entity.get_component::<T>()` 断言实体状态
- **控制时间步进**：可以精确控制每帧的 `dt`，实现单步调试

## 最小测试场景

以下是一个完整的测试模式，演示如何验证碰撞域的判定逻辑：

```rust
#[cfg(test)]
mod tests {
    use duan::{Entity, World};
    use crate::components::{Position, Velocity, Collider};
    use crate::domains::{MotionRules, CollisionRules};
    use crate::events::GroundCollisionEvent;

    #[test]
    fn test_ball_hits_ground() {
        // 1. 构建最小仿真场景
        let mut world = World::builder()
            .with_domain("motion", MotionRules::earth())
            .with_domain("collision", CollisionRules::new())
            .build();

        // 2. spawn 所需实体
        let _ground = world.spawn(
            Entity::new("ground")
                .with_domain("collision")
                .with_component(Position::new(0.0, 0.0, 0.0))
                .with_component(Collider::ground(0.8, 0.05)),
        );

        let ball = world.spawn(
            Entity::new("ball")
                .with_domain("motion")
                .with_domain("collision")
                .with_component(Position::new(0.0, 0.5, 0.0))
                .with_component(Velocity::new(0.0, -5.0, 0.0))
                .with_component(Collider::sphere(0.1, 1.0)),
        );

        // 3. 步进若干帧，直到碰撞发生
        let mut collision_event = None;
        for _ in 0..20 {
            let events = world.step_collect(0.05);
            collision_event = events
                .iter()
                .find_map(|e| e.downcast::<GroundCollisionEvent>().map(|c| c.impact_velocity));
            if collision_event.is_some() {
                break;
            }
        }

        // 4. 断言事件
        let impact_vel = collision_event.expect("期望球碰到地面");
        assert!(impact_vel > 0.0, "碰撞速度应为正值");

        // 5. 断言组件状态
        let entity = world.get_entity(ball).expect("球实体应仍存在");
        let pos = entity.get_component::<Position>().unwrap();
        assert!(pos.y >= 0.0, "碰撞后球不应低于地面");
    }
}
```

## step_collect 的返回值

`step_collect(dt)` 返回 `Vec<Arc<dyn CustomEvent>>`，包含**本帧域计算阶段产生的全部自定义事件**。

使用 `downcast::<T>()` 方法将事件转换为具体类型：

```rust
let events = world.step_collect(1.0);

// 检查是否存在特定类型的事件
let hit = events.iter().find_map(|e| e.downcast::<HitEvent>());
assert!(hit.is_some(), "期望命中事件被发出");

// 断言特定字段
if let Some(hit) = hit {
    assert_eq!(hit.target_id, expected_target);
    assert!(hit.damage > 0.0);
}

// 统计事件数量
let hit_count = events.iter().filter(|e| e.downcast::<HitEvent>().is_some()).count();
assert_eq!(hit_count, 2, "期望两次命中");
```

## 断言组件状态

通过 `world.get_entity` 读取实体的组件状态：

```rust
let entity = world.get_entity(entity_id).expect("实体应存在");

// 读取位置
let pos = entity.get_component::<Position>().unwrap();
assert!((pos.x - expected_x).abs() < 0.001, "x 误差超出容忍范围");

// 检查实体是否仍然存活（未被销毁）
assert!(world.get_entity(entity_id).is_some(), "实体不应被销毁");
assert!(world.get_entity(destroyed_id).is_none(), "实体应已被销毁");
```

## 可重复的单步执行

固定 `dt` 可以得到完全确定性的仿真结果，适合对精确帧序列进行验证：

```rust
#[test]
fn test_missile_expires_after_timeout() {
    let mut world = /* ... */;
    let missile = world.spawn(/* 携带 5 秒寿命的导弹 */);

    // 步进 4 秒，导弹应存活
    for _ in 0..40 {
        world.step(0.1);
    }
    assert!(world.get_entity(missile).is_some(), "4 秒时导弹应存活");

    // 再步进 2 秒，导弹应已超时销毁
    for _ in 0..20 {
        world.step(0.1);
    }
    assert!(world.get_entity(missile).is_none(), "6 秒时导弹应已销毁");
}
```

## 同时使用 step_with 和事件断言

若需要在收集事件的同时执行生命周期操作（如 spawn），可以用 `step_with` 加手动收集：

```rust
let mut collected_hits = Vec::new();

world.step_with(dt, |event, world| {
    if let Some(fire) = event.downcast::<FireEvent>() {
        // 在回调中执行 spawn
        world.spawn(Entity::new("missile").with_domain("motion"));
    }
    if let Some(hit) = event.downcast::<HitEvent>() {
        // 收集命中信息用于后续断言
        collected_hits.push(hit.target_id);
    }
});

assert!(!collected_hits.is_empty());
```

需要同时 spawn 实体和断言事件时，优先使用 `step_with`；只需断言事件、不需要生命周期操作时，`step_collect` 更简洁。

## 最佳实践

- **每个测试聚焦一个行为**：只 spawn 完成该测试所需的最少实体和域
- **使用固定 dt**：避免测试因时间步长变化而不稳定
- **断言边界条件**：测试不仅要验证"正常路径"，也要验证域在边界输入下的行为
- **域依赖声明要正确**：测试中注册的域必须满足所有依赖声明，否则 `World::build()` 会 panic
