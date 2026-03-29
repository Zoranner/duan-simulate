# 自定义域

本文档描述如何实现自定义域。

## 域的基本要求

### 需要实现的功能

每个域需要实现以下功能：

**计算接口**

每帧调用，执行域的计算逻辑。

**准入接口（`try_attach`）**

纯谓词，判断实体是否满足准入条件，返回 `true/false`。不应有副作用，框架可能在不同时机多次调用它做探测。

**初始化接口（`on_attach`）**

准入成功后由框架调用一次，用于在域内初始化与该实体相关的状态（如缓存实体 ID、建立内部索引等）。

**脱离接口**

实体从域中脱离时的回调。

**依赖声明接口**

返回该域依赖的其他域的名称列表。

### 域的属性

**名称**

域的唯一标识字符串。

**实体列表**

归属该域的实体集合。

**算法实现**

域的具体计算逻辑。

## 设计域的步骤

### 第一步：确定域的职责

明确域负责什么：

- 这个域解决什么问题？
- 这个域做什么判定？
- 这个域提供什么服务？

**示例**：探测域负责判断一个实体能否探测到另一个实体。

### 第二步：确定准入条件

确定实体需要什么组件才能加入这个域：

- 实体必须有哪些组件？
- 组件需要满足什么条件？

**示例**：探测域要求实体有位置组件和雷达组件。

### 第三步：确定依赖关系

确定这个域需要哪些其他域的信息：

- 这个域的判定依赖什么？
- 需要查询哪些其他域？

**示例**：探测域依赖空间域（距离计算）和阵营域（敌我判断）。

### 第四步：设计服务接口

确定这个域向其他域提供什么服务：

- 其他域可能需要查询什么信息？
- 如何设计接口使其清晰易用？

**示例**：探测域可能提供"已知探测结果"的查询接口。

### 第五步：设计事件类型

确定这个域会产生什么事件：

- 计算结果如何表达？
- 事件包含哪些信息？

**示例**：探测域产生探测事件，包含观察者、目标、距离、置信度等信息。

### 第六步：实现计算逻辑

实现域的核心计算逻辑：

- 如何遍历实体？
- 如何进行判定？
- 如何发出事件？

## 设计建议

### 单一职责

每个域应该只负责一个领域：

- 不要把不相关的逻辑放在同一个域中
- 如果域变得复杂，考虑拆分

### 明确的准入条件

准入条件应该清晰明确：

- 在文档中说明需要哪些组件
- 在准入接口中严格检查
- 对不满足条件的实体给出明确原因

### 合理的依赖

依赖关系应该是必要的：

- 只依赖真正需要的域
- 避免循环依赖
- 考虑执行顺序的影响

### 清晰的接口

服务接口应该清晰易用：

- 接口命名表达意图
- 参数和返回值类型明确
- 复杂的操作提供文档说明

### 适度的事件

事件应该包含必要的信息：

- 不要遗漏重要信息
- 也不要包含过多冗余信息
- 考虑事件的使用者需要什么

## 实现注意事项

### 服务方法签名约定

域的服务方法（供其他域在 `compute` 中查询的只读方法）需要由调用方显式传入 `&EntityStore`：

```rust
// 服务方法的标准签名
pub fn is_hostile(&self, id_a: EntityId, id_b: EntityId, entities: &EntityStore) -> bool
pub fn distance(&self, id_a: EntityId, id_b: EntityId, entities: &EntityStore) -> Option<f64>
```

调用侧写法：

```rust
let faction = ctx.get_domain::<FactionRules>().expect("阵营域未注册");
let hostile = faction.is_hostile(a, b, &ctx.entities);
```

原因：域服务方法只持有 `&self`，不持有 `DomainContext`，因此无法自行访问实体存储。由调用方传入 `&ctx.entities` 是唯一合法的模式——服务方法只负责"怎么算"，不负责"从哪里取数据"。这是当前架构下明确的职责分工，不是框架的额外限制。

### domain_rules_any! 宏

每个 `DomainRules` 实现的末尾都必须包含：

```rust
impl DomainRules for MyRules {
    // ... 其他方法 ...
    domain_rules_any!(MyRules);
}
```

该宏的展开结果是为具体类型实现 `as_any(&self) -> &dyn Any` 和 `as_any_mut(&mut self) -> &mut dyn Any` 两个方法，用于运行时向下转型（`ctx.get_domain::<T>()` 的内部实现依赖此机制）。

Rust 不允许在 trait 定义中为所有实现类提供这两个方法的默认实现（会引入歧义），因此每个实现类必须手动提供，宏消除了这一机械重复。这是每个 `DomainRules` 实现的固定样板行，缺少它会导致编译错误。

### 域的写入边界

在计算阶段，域可以直接修改**自身管辖实体**的组件状态（这是域权威性的直接体现），同时可以向事件通道追加事件。但**不能在 `compute()` 中直接发起生命周期操作（创建/销毁实体）**，生命周期操作通过事件系统在事件处理阶段完成。

这一约束是刻意设计的，背后有两条核心理由：

**理由一：可追溯性**。框架的事件通道是所有跨边界影响的唯一可见记录。若允许 `compute()` 直接 spawn 实体，创建行为就成了无法被外部观察、记录和回放的隐式副作用；通过发出事件，每一次实体创建都在事件流中有对应的记录，支持仿真复盘和调试。

**理由二：未来并行化安全**。`compute()` 阶段按依赖图串行执行，未来有望并行化。若允许域在 compute 中直接创建实体，多个域可能同时操作实体存储，需要额外的同步机制；将生命周期操作集中在事件处理阶段可以保持 compute 阶段的安全边界。

**推荐模式：事件 + `step_with` 回调**

域在 `compute()` 中做出"应创建/销毁实体"的决策后，通过发出自定义事件来传递意图，再由应用层的 `step_with` 回调执行生命周期操作：

```rust
// 1. 定义携带完整初始化数据的事件
pub struct FireEvent {
    pub shooter_id: EntityId,
    pub launch_pos: (f64, f64),
    pub target_id: EntityId,
}

// 2. 域在 compute 中发出事件，而不直接 spawn
fn compute(&mut self, ctx: &mut DomainContext) {
    for shooter_id in ctx.own_entity_ids() {
        if self.should_fire(shooter_id, ctx) {
            ctx.emit(DomainEvent::custom(FireEvent {
                shooter_id,
                launch_pos: self.get_position(shooter_id, ctx),
                target_id: self.current_target[&shooter_id],
            }));
        }
    }
}

// 3. 应用层在 step_with 回调中负责实际创建实体
world.step_with(dt, |event, world| {
    if let Some(fire) = event.downcast::<FireEvent>() {
        world.spawn(
            Entity::new("missile")
                .with_domain("motion")
                .with_domain("collision")
                .with_component(Position::new(fire.launch_pos.0, fire.launch_pos.1))
                .with_component(MissileState::new(fire.target_id, fire.shooter_id)),
        );
    }
});
```

这一模式保证了事件流的完整性：`FireEvent` 可以被日志记录、回放和测试断言，实体创建逻辑也集中在一处，易于追踪。

### compute 中的借用模式

由于 Rust 借用检查器的约束，在同一作用域内不能同时持有同一数据的不可变引用和可变引用。在 `compute` 中读取实体数据后再写回时，需要先将需要的数据提取到局部变量，释放不可变借用后再进行可变访问：

```rust
fn compute(&mut self, ctx: &mut DomainContext) {
    let entity_ids: Vec<EntityId> = ctx.own_entity_ids().collect();

    for entity_id in entity_ids {
        // 先只读借用，提取数据到局部变量
        let (x, y, vx, vy) = {
            let entity = ctx.entities.get(entity_id).unwrap();
            let pos = entity.get_component::<Position>().unwrap();
            let vel = entity.get_component::<Velocity>().unwrap();
            (pos.x, pos.y, vel.vx, vel.vy)
        }; // 只读借用在此释放

        // 再可变借用写回
        if let Some(entity) = ctx.entities.get_mut(entity_id) {
            if let Some(pos) = entity.get_component_mut::<Position>() {
                pos.x = x + vx * ctx.dt;
                pos.y = y + vy * ctx.dt;
            }
        }
    }
}
```

先 `collect` entity_ids 同理——这样做是为了释放对 `ctx.own_entities` 的借用，以便后续可变访问 `ctx.entities`。这是处理 Rust 所有权的标准写法，不是框架的额外限制。

### on_attach / on_detach 生命周期钩子

`DomainRules` 提供两个生命周期钩子，用于在实体加入或离开域时执行初始化/清理工作。这是管理"域内每实体缓存"的标准入口，比在 `compute()` 中做懒初始化更清晰、性能更好。

#### on_attach

**签名**：`fn on_attach(&mut self, entity: &Entity)`

**调用时机**：在 `try_attach()` 返回 `true` 后，由框架**同步调用**一次。此时实体已被插入实体存储（所有组件完整可用），但尚未进入 `Active` 状态——`on_attach` 是实体"激活前的初始化"窗口。

**`entity: &Entity` 参数**：包含 `spawn` 时传入的所有组件数据，可通过 `entity.get_component::<T>()` 读取任意组件初始值。`entity.id` 是已分配好的 `EntityId`。

**合法操作**：
- 在域内初始化"每实体缓存"（如历史轨迹容器、冷却计时器等）
- 读取组件初始值来配置域内状态
- 记录实体 ID（如将地面实体 ID 缓存到 `ground_id` 字段）

**不适合的操作**：
- 依赖其他域的数据——其他域可能尚未对该实体执行 `on_attach`，顺序不确定
- 发出事件——`on_attach` 调用时没有事件通道访问权
- 修改实体组件——参数为 `&Entity`（只读），如需修改请在首帧 `compute()` 中进行

#### on_detach

**签名**：`fn on_detach(&mut self, entity_id: EntityId)`

**调用时机**：实体销毁流程开始时同步调用。此时实体已从实体存储中脱离，只传入 `EntityId`（实体数据不再可访问）。

**职责**：清理域内与该实体关联的所有缓存，防止内存泄漏。这是与 `on_attach` 的对称操作——`on_attach` 中建立的所有数据结构，应在 `on_detach` 中释放。

#### 完整示例：追踪历史缓存

```rust
pub struct TrackingRules {
    // 域内每实体缓存：记录历史轨迹
    histories: HashMap<EntityId, Vec<(f64, f64)>>,
}

impl DomainRules for TrackingRules {
    fn try_attach(&self, entity: &Entity) -> bool {
        entity.has_component::<Position>()
    }

    fn on_attach(&mut self, entity: &Entity) {
        // 读取初始位置来初始化轨迹缓存
        let initial = entity
            .get_component::<Position>()
            .map(|pos| (pos.x, pos.y));
        self.histories
            .insert(entity.id, initial.into_iter().collect());
    }

    fn on_detach(&mut self, entity_id: EntityId) {
        // 清理该实体的缓存，避免内存泄漏
        self.histories.remove(&entity_id);
    }

    fn compute(&mut self, ctx: &mut DomainContext) {
        // 不再需要懒初始化检查——on_attach 已保证缓存存在
        let entity_ids: Vec<EntityId> = ctx.own_entity_ids().collect();
        for entity_id in entity_ids {
            if let Some(entity) = ctx.entities.get(entity_id) {
                if let Some(pos) = entity.get_component::<Position>() {
                    if let Some(history) = self.histories.get_mut(&entity_id) {
                        history.push((pos.x, pos.y));
                    }
                }
            }
        }
    }

    domain_rules_any!(TrackingRules);
}
```

**惯用原则**：`on_attach` 中建立的所有数据 → `on_detach` 中必须清理。如果域内有 `HashMap<EntityId, T>`，在 `on_detach` 中调用 `map.remove(&entity_id)` 是固定模式，不要遗漏。

## 常见问题

### 域可以没有计算逻辑吗

可以。某些域只提供服务接口，不需要每帧计算。这种情况下，计算接口可以为空。

### 域可以依赖不存在的域吗

不建议。如果依赖的域不存在，域注册表会在计算执行顺序时报错。

### 域可以动态添加实体吗

通常实体在创建时加入域。如果需要在运行时动态添加，需要调用域的准入接口。

## 参考实现

> 以下域仅为示例，不是框架的一部分。用户应根据具体仿真场景自行实现。

### 空间域

**职责**：空间关系的计算与查询（距离、视线遮挡、范围查询）。

**准入条件**：实体有位置组件（坐标 + 朝向）。

**依赖**：无（基础域，常被其他域依赖）。

**服务接口**：距离计算、视线检测、范围内实体查询。

**产生事件**：通常不产生，只提供服务。

---

### 阵营域

**职责**：敌我关系的判定与查询。

**准入条件**：实体有阵营组件（阵营标识）。

**依赖**：无（基础域）。

**服务接口**：敌对判断、阵营查询、敌对实体列表。

**配置**：需要预配置阵营关系（敌对/友好/中立）。

**产生事件**：通常不产生，只提供服务。

---

### 运动域

**职责**：根据运动参数每帧更新实体位置和速度。

**准入条件**：实体有位置组件 + 速度组件。

**依赖**：无（基础域，常被碰撞域等依赖）。

**产生事件**：通常不产生——运动域作为权威直接修改自身管辖实体的位置和速度组件，无需绕道事件系统。碰撞等后续判定通过执行顺序保证（碰撞域声明依赖运动域）读取到本帧已更新的结果。

---

### 探测域

**职责**：判断一个实体能否探测到另一个实体，典型的主控域模式示例。

**准入条件**：实体有位置组件 + 雷达组件（探测范围、类型等）。

**依赖**：空间域、阵营域。

**计算流程（主控域模式）**：
1. 遍历有探测能力的实体
2. 对每个实体遍历潜在目标
3. 问阵营域：目标是否敌对？
4. 问空间域：目标是否在探测范围内？
5. 问空间域：是否有视线遮挡？（可选）
6. 综合判断后发出探测事件

**产生事件**：探测事件（观察者、目标、距离、置信度、时间戳）。

**跨域服务调用示例**：

探测域依赖阵营域，在 `compute` 中通过 `ctx.get_domain::<T>()` 获取另一个域的只读引用，直接调用其服务方法：

```rust
fn compute(&mut self, ctx: &mut DomainContext) {
    // 方式一：按实现类型查找（推荐，编译期类型安全，无需 downcast）
    let faction = ctx
        .get_domain::<FactionRules>()
        .expect("探测域依赖阵营域，但阵营域未注册");

    let observer_ids: Vec<EntityId> = ctx.own_entity_ids().collect();
    for observer_id in observer_ids {
        // ... 遍历目标，调用阵营域服务
        // 注意：服务方法需要由调用方传入 &ctx.entities，见下方"服务方法签名约定"
        if faction.is_hostile(observer_id, target_id, &ctx.entities) {
            // 进行探测判定...
        }
    }
}
```

若需按名称动态查找（适用于域名由配置决定的场景）：

```rust
// 方式二：按名称查找，需手动向下转型
if let Some(domain) = ctx.get_domain_by_name("faction") {
    if let Some(faction) = domain.rules.as_any().downcast_ref::<FactionRules>() {
        // 调用服务方法...
    }
}
```

方式一是常规写法，在编译期就能确保类型正确；方式二适合域名由配置决定、编译时不确定具体类型的场景。

**遍历潜在目标**：

探测是跨实体的交叉计算：观察者在本域管辖范围内，目标则可能是任意活跃实体（包括未归属探测域的实体）。

`ctx.entities` 提供了对全部实体的访问能力，全量只读遍历是合法的，不违反权威边界（写入权限仍限于自身管辖实体）：

```rust
// 遍历全部活跃实体作为候选目标
for target in ctx.entities.active_entities() {
    if target.id == observer_id { continue; }
    // 候选目标判定...
}
```

对于范围敏感的场景，更推荐通过空间域的范围查询服务获取候选目标，避免 O(n²) 的全量遍历：

```rust
// 通过空间域的范围查询获取候选目标（性能更优）
let space = ctx.get_domain::<SpaceRules>().expect("探测域依赖空间域");
let candidates = space.query_range(observer_pos, radar_range);
```

两种方式都是合法的，选择取决于场景需求和性能要求。

---

### 战斗域

**职责**：武器交战判定和伤害计算。

**准入条件**：实体有位置组件 + 武器组件（武器类型、射程、射速、伤害等）。

**依赖**：空间域、阵营域。

**产生事件**：命中事件（攻击者、目标、命中位置、伤害值）、拦截事件。

---

### 协调型域示例：综合威胁评估域

不对应具体物理领域，作为多个域的协调者。

**职责**：综合探测、战斗、阵营信息，评估威胁等级。

**依赖**：探测域、战斗域、阵营域。

**产生事件**：威胁评估事件（评估者、目标、威胁等级、评估依据）。
