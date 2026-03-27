# 惯用法

本文档收录 DUAN 框架推荐的标准写法和常见模式。这些惯用法不是框架强制的规则，而是经过实践验证的最佳实践，能让代码更清晰、更可维护。

## 标记组件（Marker Component）

**用途**：为实体显式声明语义角色，替代依赖"缺少某个组件"的隐式推断。

**问题场景**：碰撞域需要区分"地面"和"小球"，如果用 `!has_velocity()` 来判断是地面，当地面未来需要速度组件时，判断会悄悄失效。

**正确做法**：

```rust
// 定义零大小标记组件
#[derive(Debug, Clone)]
pub struct StaticBody;
impl_component!(StaticBody, "static_body");

// 实体在构建时显式声明角色
Entity::new("ground")
    .with_component(StaticBody)  // 明确声明为静态碰撞体
    .with_component(Position::new(0.0, 0.0, 0.0))
    .with_component(Collider::ground(0.8, 0.05))

// 域在 on_attach 中检查标记，而非推断缺失
fn on_attach(&mut self, entity: &Entity) {
    if entity.has_component::<StaticBody>() {
        self.ground_id = Some(entity.id);
    }
}
```

标记组件是零大小类型（ZST），没有运行时开销，只在类型系统和组件存储中占有一个槽位。

## 域内状态 vs 组件状态

域内状态和实体组件状态有明确的语义边界，选择哪种存储取决于数据的归属。

**组件状态**：属于实体的属性，对所有域可见，描述实体"是什么"。

```rust
// 位置、速度、质量——这些是实体自身的属性
pub struct Position { pub x: f64, pub y: f64, pub z: f64 }
pub struct Velocity { pub vx: f64, pub vy: f64, pub vz: f64 }
```

**域内状态**：属于该域对实体的私有记录，描述"从这个域的视角看，该实体的历史或中间状态"。

```rust
pub struct CollisionRules {
    // 域私有：碰撞域记录实体上一帧的位置，用于穿越检测
    // 这不是实体的属性，而是碰撞域的计算辅助信息
    prev_y: HashMap<EntityId, f64>,

    // 域私有：缓存的地面实体 ID，避免每帧重新查找
    ground_id: Option<EntityId>,
}
```

**判断规则**：
- 如果其他域也需要这个数据 → 放组件
- 如果数据只服务于本域的计算 → 放域内状态
- 实体销毁时，域内状态应在 `on_detach` 中清理

## compute 中的借用模式

Rust 借用检查器要求在同一作用域内不能同时持有不可变引用和可变引用。在 `compute` 中，读取实体数据后再写回时，需要先提取数据到局部变量：

```rust
fn compute(&mut self, ctx: &mut DomainContext) {
    // 先 collect，释放对 own_entities 的借用
    let entity_ids: Vec<EntityId> = ctx.own_entity_ids().collect();

    for entity_id in entity_ids {
        // 阶段一：只读借用，提取到局部变量
        let (x, y, vx, vy) = {
            let entity = match ctx.entities.get(entity_id) {
                Some(e) => e,
                None => continue,
            };
            let pos = match entity.get_component::<Position>() {
                Some(p) => p,
                None => continue,
            };
            let vel = match entity.get_component::<Velocity>() {
                Some(v) => v,
                None => continue,
            };
            (pos.x, pos.y, vel.vx, vel.vy)
        }; // 只读借用在此结束

        // 阶段二：计算新值
        let new_x = x + vx * ctx.dt;
        let new_y = y + vy * ctx.dt;

        // 阶段三：可变借用，写回结果
        if let Some(entity) = ctx.entities.get_mut(entity_id) {
            if let Some(pos) = entity.get_component_mut::<Position>() {
                pos.x = new_x;
                pos.y = new_y;
            }
        }
    }
}
```

这是处理 Rust 所有权约束的标准写法，不是框架附加的限制。

## 自定义事件的 downcast

处理自定义事件时，使用 `downcast<T>()` 方法代替手写 `as_any().downcast_ref::<T>()`：

```rust
world.step_with(dt, |event: &dyn CustomEvent, _world| {
    // 推荐写法
    if let Some(c) = event.downcast::<GroundCollisionEvent>() {
        println!("碰撞速度: {:.2} m/s", c.impact_velocity);
    }
});
```

## 域名与实体声明的一致性

域注册和实体归属声明都使用字符串，拼写错误只有运行时才能发现（debug 模式下会 panic）。建议将域名定义为常量，统一管理：

```rust
// 在域模块中定义常量
pub const DOMAIN_MOTION: &str = "motion";
pub const DOMAIN_COLLISION: &str = "collision";

// 注册和声明都引用同一个常量
world.register_domain(DOMAIN_MOTION, MotionRules::earth());

Entity::new("ball")
    .with_domain(DOMAIN_MOTION)
    .with_domain(DOMAIN_COLLISION)
```

## 事件处理器编写建议

- **保持简单**：只做与该事件直接相关的操作。复杂的跨实体计算应在域的 `compute` 中完成，处理器只负责把计算结果写入状态
- **事件数据应自包含**：处理器所需的信息应由域在发出事件时一并打包，避免处理器反查大量外部数据
- **不能调用域的服务接口**：事件处理阶段域上下文不可用；若处理器需要某个判断结果，应由域在计算时将其包含在事件数据中

## WorldBuilder 链式配置

推荐使用 WorldBuilder 的链式 API 在一处完成世界配置，`build()` 会自动验证域依赖关系：

```rust
let mut world = World::builder()
    .time_scale(1.0)
    .with_domain("motion", MotionRules::earth())
    .with_domain("collision", CollisionRules::new())
    .build(); // 此处若有循环依赖会立即 panic，而不是在首帧运行时才发现
```

也可以在 `build()` 后继续调用 `register_domain`，两种方式等价，按需混用。
