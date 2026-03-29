# DUAN 下一代架构设计

> 临时设计文档，记录对话中形成的终态架构设计思路。
> 尚未进入正式决策流程，所有内容均可修改。

---

## 设计哲学

### 核心原则

```
域是组件类型的权威。组件的存在即表示归属。
```

- **写入独占**：一个组件类型最多被一个域声明独占写入，`build()` 时校验
- **依赖显式**：`type Writes/Reads/After` 三个关联类型，全部编译期验证
- **事件驱动跨边界**：事件可序列化，支持回放与分布式
- **算法可替换**：换 Domain 实现不影响其他代码

### 三个编程原语

框架暴露给用户的编程单元只有三个：

| 概念 | 本质 | 用户做什么 |
|------|------|-----------|
| Component | 纯数据，`#[derive(Component, Clone)]` | 定义数据结构 |
| Entity | 零大小标记类型 | 定义实体行为 + 默认组件 |
| Domain | 组件权威 | 定义跨实体逻辑 |

### 数据流

```
Entity::tick()  →  意图组件  →  Domain::compute()  →  状态组件
（单向不可逆）
```

- Entity 只能写自己的组件，且不能写任何域声明了 Writes 的组件
- Domain 只能写 `type Writes` 声明的组件，编译期保证
- 意图组件是 Entity 和 Domain 之间的唯一通信管道

---

## 核心概念

### EntityId：结构化 64 位 ID

```rust
/// 全局唯一实体标识，分布式就绪
///
/// 内存布局：
///   [63:48] node_id: u16      最多 65536 个分布式节点
///   [47:32] generation: u16   代际版本，检测悬垂引用
///   [31:0]  local_index: u32  单节点支持 40 亿实体
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct EntityId(u64);

impl EntityId {
    pub fn node(&self) -> u16;
    pub fn generation(&self) -> u16;
    pub fn local_index(&self) -> u32;
    pub fn is_local(&self) -> bool;
    pub fn is_remote(&self) -> bool;
}
```

- 本地单节点：`node_id = 0`，行为与普通 u64 ID 一致
- 分布式模式：`node_id` 直接告知管辖节点，无需额外查表
- 代际（generation）：框架检测"用了已销毁实体的旧引用"

---

### Component：纯数据，按类型密集存储

```rust
pub trait Component: Send + Sync + Clone + 'static {}

#[derive(Component, Clone)]
pub struct Position { pub x: f64, pub y: f64, pub z: f64 }

#[derive(Component, Clone)]
pub struct Health { pub value: f64, pub max: f64 }

#[derive(Component, Clone)]
pub struct MovementOrder { pub desired_velocity: Vec3 }
```

**存储设计：**

```
ComponentStorage<Position>:
  dense: Vec<Position>               连续内存，缓存友好
  entity_to_slot: SparseVec<u32>     EntityId.local_index → 槽位
  slot_to_entity: Vec<EntityId>      槽位 → EntityId（反查）
```

- 十万个 Position = 一块 2.4MB 连续内存，L2 缓存全命中
- `#[derive(Component)]` 生成所有必要实现，无样板代码

---

### Entity：零大小标记类型

Entity 没有字段。所有状态都是 Component。Entity 只做两件事：定义行为逻辑、声明默认组件。

```rust
pub trait Entity: 'static {
    /// 每帧调用。静态方法，无 &self。
    /// 读冻结快照做决策，写自身意图组件。
    fn tick(ctx: &mut EntityContext) {}

    /// 此实体类型自带的默认组件
    fn bundle() -> impl ComponentBundle { () }
}
```

**用法：**

```rust
pub struct Soldier;

impl Entity for Soldier {
    fn tick(ctx: &mut EntityContext) {
        let mem   = ctx.get::<SoldierMemory>().unwrap();
        let pos   = ctx.get::<Position>().unwrap();
        let world = ctx.world();

        let threats = world.query_nearby::<Position>(pos, 500.0);
        let new_mem = decide_threats(mem, &threats);

        ctx.set(MovementOrder { desired_velocity: flee_direction(pos, &threats) });
        ctx.set(AttackIntent::target(closest_threat(&threats)));
        ctx.set(new_mem);
    }

    fn bundle() -> impl ComponentBundle {
        (
            Health::new(100),
            SoldierMemory::new(),
            FactionTag::Blue,
        )
    }
}

// "私有状态"作为组件，没有任何域声明 Writes 它
#[derive(Component, Clone)]
pub struct SoldierMemory {
    threats: Vec<ThreatInfo>,
    phase: AIPhase,
}
```

**无行为实体：**

```rust
pub struct Obstacle;
impl Entity for Obstacle {
    fn bundle() -> impl ComponentBundle {
        (BlockingRadius(10.0),)
    }
}
```

**生成实体：**

```rust
// bundle() 定义的组件自动附加，只传个性化参数
let id = world.spawn(Soldier)
    .with(Position::new(100.0, 0.0, 50.0))
    .id();

let wall = world.spawn(Obstacle)
    .with(Position::new(50.0, 0.0, 0.0))
    .id();
```

**Entity 为什么没有字段：**

| 如果 Entity 有字段 | 如果 Entity 无字段 |
|---|---|
| 框架看不到私有状态 → 快照断裂 | 所有状态都是 Component → 快照完整 |
| `Box<dyn Entity>` 存储 → 缓存不友好 | 函数指针存储 → 8 字节/实体 |
| tick 读写同一存储 → 确定性未定义 | 冻结快照 + pending buffer → 天然确定 |
| tick 可并行性差 | embarrassingly parallel |

"私有性"通过架构约定实现：SoldierMemory 不在任何域的 Writes 声明中，只有 Soldier::tick 会写它。debug 模式下框架可检测违规。

---

### EntityContext：实体 tick 的访问入口

```rust
pub struct EntityContext<'w> {
    entity_id: EntityId,
    own: OwnComponentView<'w>,       // 只能读写自身组件
    snapshot: &'w WorldSnapshot,      // 上一帧末冻结快照（只读）
    events: EventEmitter<'w>,
    pub clock: &'w TimeClock,
    pub dt: f64,
}

impl EntityContext<'_> {
    pub fn id(&self) -> EntityId;

    /// 读取自身组件
    pub fn get<T: Component>(&self) -> Option<&T>;

    /// 写入自身组件（debug 模式：如果 T 在某域的 Writes 中则 panic）
    pub fn set<T: Component>(&mut self, value: T);
    pub fn remove<T: Component>(&mut self);

    /// 上一帧末的冻结快照（只读，所有实体共享同一份）
    pub fn world(&self) -> &WorldSnapshot;

    /// 发出事件（延迟到 Phase 4 处理）
    pub fn emit<E: CustomEvent>(&mut self, event: E);

    /// 生命周期命令（延迟到 Phase 5 执行）
    pub fn spawn(&mut self, entity: impl Entity) -> EntityId;
    pub fn destroy(&mut self, id: EntityId);
}
```

**确定性保证：**
- `ctx.world()` 返回的是 Phase 2 开始前冻结的快照，所有实体看到相同数据
- `ctx.set()` 写入 pending buffer，Phase 2 结束后统一 flush
- Entity tick 顺序不影响结果 → 可用 rayon 并行

**一帧延迟：**
- 实体在 Phase 2 读到的域状态（DetectionResult、SpatialIndex 等）是上一帧末的
- 符合"感知-决策-行动"的物理时序
- 高频仿真（100Hz+）时延迟可忽略

---

### Domain：组件类型的权威

```rust
pub trait Domain: Send + Sync + 'static {
    /// 独占写入的组件类型（此域是这些组件的唯一权威）
    type Writes: ComponentSet = ();

    /// 共享只读的组件类型
    type Reads: ComponentSet = ();

    /// 必须在本域之前完成的域（编译期类型检查）
    type After: DomainSet = ();

    /// 每帧计算
    fn compute(&mut self, ctx: &mut DomainContext<Self>, dt: f64) {}
}
```

**五行定义，三个关联类型，一个方法。**

没有 `on_attach`、没有 `on_detach`、没有 `try_attach`、没有 `as_any`。新实体在 Phase 5 生效，下一帧 compute 自动看到。

**域的语义角色（全部是同一 trait）：**

| 角色 | Writes | 典型示例 |
|------|--------|---------|
| 交互域 | 仲裁结果组件 | `DetectionDomain`、`CombatDomain` |
| 执行域 | 物理状态组件 | `MotionDomain`、`KinematicsDomain` |
| 服务域 | `()` | `SpatialIndex`、`EquipmentDatabase` |

分类是语义指导，不是不同的类型。

---

### DomainContext：域 compute 的访问入口

```rust
pub struct DomainContext<'w, D: Domain> {
    writes: WriteView<'w, D::Writes>,
    reads:  ReadView<'w, D::Reads>,
    events: EventEmitter<'w>,
    pub clock: &'w TimeClock,
    pub dt: f64,
}

impl<D: Domain> DomainContext<'_, D> {
    /// 迭代本域管辖的实体（Writes != () 时：拥有 Writes 全部组件的实体）
    pub fn entities(&self) -> impl Iterator<Item = EntityId>;

    /// 按组件类型迭代（任何域都可用，服务域主要用这个）
    pub fn each<T: InReads<D>>(&self) -> impl Iterator<Item = (EntityId, &T)>;

    /// 写本域管辖实体的组件（编译期：只能写 Writes 中的类型）
    pub fn get_mut<T: InWrites<D>>(&mut self, id: EntityId) -> Option<&mut T>;

    /// 读任意实体的组件（编译期：只能读 Reads 中的类型）
    pub fn get<T: InReads<D>>(&self, id: EntityId) -> Option<&T>;

    /// 跨域查询（只能访问已执行完毕的域）
    pub fn domain<OtherD: Domain>(&self) -> Option<&OtherD>;

    /// 发出事件
    pub fn emit<E: CustomEvent>(&mut self, event: E);

    /// 生命周期命令（延迟到 Phase 5 执行）
    pub fn spawn(&mut self, entity: impl Entity) -> EntityId;
    pub fn destroy(&mut self, id: EntityId);
}
```

---

### 意图组件设计原则

意图组件**按域设计**，不按实体设计。所有想移动的实体输出同一种意图：

```
Soldier::tick()  → 寻路算法     → MovementOrder { desired_velocity }
Missile::tick()  → 制导算法     → MovementOrder { desired_velocity }
Vehicle::tick()  → 编队算法     → MovementOrder { desired_velocity }
                                      ↓
                              MotionDomain::compute()
                                      ↓
                                Position, Velocity
```

```rust
// 通用运动意图——对 MotionDomain 来说所有实体都一样
#[derive(Component, Clone)]
pub struct MovementOrder { pub desired_velocity: Vec3 }

// 通用攻击意图——对 CombatDomain 来说所有实体都一样
#[derive(Component, Clone)]
pub struct AttackIntent { pub target: Option<EntityId>, pub weapon_id: u32 }
```

新增 Entity 类型**不需要修改任何 Domain**。差异在 Entity::tick() 内部消化，意图组件是统一接口。

---

## 域示例

### 执行域：MotionDomain

```rust
pub struct MotionDomain;

impl Domain for MotionDomain {
    type Writes = (Position, Velocity);
    type Reads  = (MovementOrder,);

    fn compute(&mut self, ctx: &mut DomainContext<Self>, dt: f64) {
        for id in ctx.entities() {
            let vel = if let Some(order) = ctx.get::<MovementOrder>(id) {
                order.desired_velocity
            } else {
                *ctx.get::<Velocity>(id).unwrap()
            };

            let pos = ctx.get_mut::<Position>(id).unwrap();
            pos.x += vel.x * dt;
            pos.y += vel.y * dt;
            pos.z += vel.z * dt;

            *ctx.get_mut::<Velocity>(id).unwrap() = vel;
        }
    }
}
```

### 服务域：SpatialIndex

```rust
pub struct SpatialIndex {
    grid: SpatialHash,
}

impl Domain for SpatialIndex {
    type Writes = ();
    type Reads  = (Position,);
    type After  = (MotionDomain,);

    fn compute(&mut self, ctx: &mut DomainContext<Self>, _dt: f64) {
        self.grid.clear();
        for (id, pos) in ctx.each::<Position>() {
            self.grid.insert(id, pos);
        }
    }
}

impl SpatialIndex {
    pub fn query_radius(&self, center: &Position, radius: f64) -> Vec<EntityId> {
        self.grid.query_radius(center, radius)
    }
}
```

**不需要空间索引的仿真不注册，零开销。坐标系由用户决定：**

| 场景 | 服务域 |
|------|-------|
| 不需要空间 | 不注册 |
| 2D 平面 | `QuadTreeIndex` |
| 3D 空战 | `OctreeIndex` |
| 地球表面 | `WGS84GeoIndex` |
| 多坐标系共存 | 注册多个不同类型 |

### 交互域：DetectionDomain

```rust
pub struct DetectionDomain;

impl Domain for DetectionDomain {
    type Writes = (DetectionResult,);
    type Reads  = (Position, FactionTag, SensorRange);
    type After  = (SpatialIndex,);

    fn compute(&mut self, ctx: &mut DomainContext<Self>, _dt: f64) {
        let index = ctx.domain::<SpatialIndex>().unwrap();
        for id in ctx.entities() {
            let pos   = ctx.get::<Position>(id).unwrap();
            let range = ctx.get::<SensorRange>(id).unwrap();

            let detected: Vec<EntityId> = index.query_radius(pos, range.0)
                .into_iter()
                .filter(|&other| is_enemy(ctx, id, other))
                .collect();

            *ctx.get_mut::<DetectionResult>(id).unwrap() = DetectionResult { targets: detected };
        }
    }
}
```

### 静态服务域：EquipmentDatabase

```rust
pub struct EquipmentDatabase {
    specs: HashMap<String, EquipmentSpec>,
}

impl Domain for EquipmentDatabase {
    // Writes、Reads、After 全部默认为空
    // compute() 默认为空，调度器识别后跳过
}

impl EquipmentDatabase {
    pub fn get_spec(&self, name: &str) -> Option<&EquipmentSpec> {
        self.specs.get(name)
    }
}
```

---

## Scheduler：静态分析，插件化执行

调度计划在 `world.build()` 时**一次性构建**：

```
输入：所有域的 Writes/Reads/After 声明
输出：执行批次（DAG 拓扑层分组）

示例：
  Level 0（可完全并行）: [MotionDomain, WeatherDomain]
  Level 1（依赖 Level 0）: [SpatialIndex]
  Level 2（依赖 Level 1）: [DetectionDomain, CollisionDomain]
  Level 3（依赖 Level 2）: [CombatDomain]

冲突检测（build 时）：
  两个域声明相同 Writes → panic
  循环依赖 → panic
```

**三种内置调度策略：**

```rust
pub enum SchedulerStrategy {
    /// 单线程顺序执行，完全确定性，适合调试和回放验证
    Sequential,
    /// rayon 线程池并行，同 Level 内并行，Level 间有 barrier
    Parallel { threads: usize },
    /// 跨节点分布式调度
    Distributed(DistributedConfig),
}
```

---

## Event：跨边界通信

```rust
pub trait CustomEvent: Send + Sync + 'static {
    fn event_name(&self) -> &str;
}

pub trait DistributedEvent: CustomEvent
    + serde::Serialize
    + for<'de> serde::Deserialize<'de> {}

#[derive(Clone)]
pub enum FrameworkEvent {
    EntityDestroyed { entity_id: EntityId, cause: DestroyCause },
    Timer { entity_id: EntityId, timer_id: String },
    Custom(Arc<dyn CustomEvent>),
}
```

---

## 仿真循环（5 阶段）

```
Phase 1  时间推进
         推进仿真时钟

Phase 2  实体 Tick
         冻结组件快照（只读，所有实体共享）
         遍历所有实体，调用 Entity::tick()
         写入进 pending buffer → Phase 2 结束后 flush
         可用 rayon 并行（实体间无数据竞争）

Phase 3  域计算
         按预构建 DAG 计划执行
         同 Level 内并行，Level 间有 barrier
         读 Reads 组件 → 计算 → 写 Writes 组件

Phase 4  事件处理
         处理 Phase 2/3 产生的所有事件（含定时器）
         用户 step_with 闭包处理自定义事件

Phase 5  生命周期
         批量执行 spawn/destroy 命令
         新实体下一帧生效
         销毁实体立即移除
```

**帧内一致性保证：**

- Phase 2 所有实体读同一份冻结快照，tick 顺序不影响结果
- Phase 3 同 Level 域读同一状态，DAG 保证无写冲突
- Phase 2/3 的事件延迟到 Phase 4 处理
- 本帧 spawn 的实体下帧才参与 Phase 2 和 Phase 3

---

## World API

```rust
// 构建
let world = World::builder()
    .with_domain(MotionDomain)
    .with_domain(DetectionDomain)
    .with_domain(CombatDomain)
    .with_domain(SpatialIndex::new(100.0))
    .with_domain(EquipmentDatabase::load("data/equipment.json"))
    .with_scheduler(SchedulerStrategy::Parallel { threads: 8 })
    .with_time_scale(1.0)
    .build();       // 静态分析 Writes/Reads/After，构建 DAG，检测冲突

// 生成实体
let id = world.spawn(Soldier)
    .with(Position::new(100.0, 0.0, 50.0))
    .id();

// 仿真步进
world.step(dt);
world.step_with(dt, |event, world| { ... });
let events = world.step_collect(dt);

// 快照与回放（所有状态在 ComponentStorage 中，完整捕获）
let snap = world.snapshot();
world.restore(snap);

// 查询
world.sim_time() -> f64
world.entity_count() -> usize
```

---

## 团队开发模式

```
项目结构：
  src/
    entities/           ← 每人独立开发，互不影响
      soldier.rs        Alice: Soldier + SoldierMemory
      tank.rs           Bob: Tank + TankState
      missile.rs        Carol: Missile + GuidanceState
      obstacle.rs       Dave: Obstacle（纯数据，无 tick）
    domains/            ← 架构师/框架组维护
      motion.rs         MotionDomain（执行域）
      detection.rs      DetectionDomain（交互域）
      combat.rs         CombatDomain（交互域）
      spatial.rs        SpatialIndex（服务域）
      equipment.rs      EquipmentDatabase（服务域）
    components/         ← 共享数据定义
      intents.rs        MovementOrder, AttackIntent, ...
      state.rs          Position, Velocity, Health, ...
    main.rs             组装 World
```

**新增 Entity 类型不修改任何 Domain。新增 Domain 不修改任何 Entity。**

---

## 分布式扩展

**核心约束：** 每个组件类型在任意时刻只有一个节点是它的权威。

**Ghost 实体机制：**

```
Node A（西北战区，entities 1–50000）
  拥有：Position、Velocity（本地 MotionDomain 管辖）
  读取：Node B 的边界实体位置快照（只读 Ghost）

Node B（东南战区，entities 50001–100000）
  拥有：Position、Velocity（本地 MotionDomain 管辖）
  读取：Node A 的边界实体位置快照（只读 Ghost）

边界同步：每帧同步 Ghost 实体的组件快照
跨节点事件：通过 DistributedEvent 序列化路由
```

EntityId 的 `node_id` 字段直接告知管辖节点。

---

## 与当前版本对比

| 维度 | 当前版本 | 终态版本 |
|------|---------|---------|
| 核心哲学 | 域管辖一组实体 | 域管辖一类组件 |
| 编程原语 | Entity + Component + DomainRules | Entity + Component + Domain |
| Entity 本质 | 数据容器（HashMap 存组件） | 零大小标记类型（tick 函数 + bundle） |
| Entity 状态 | struct 字段 + 组件混合 | 全部是 Component，无 struct 字段 |
| Domain trait | 7 个方法 + as_any | 1 个方法 + 3 个关联类型 |
| 服务概念 | 无 | `Writes = ()` 的域 |
| EntityId | 本地自增 u64 | 结构化 64 位（node+generation+index） |
| 组件存储 | 每实体 HashMap + Box | 按类型密集数组 |
| 写入边界 | 运行时检查 | 编译期类型保证 |
| 确定性 | 依赖执行顺序 | 冻结快照 + DAG，顺序无关 |
| 并行执行 | 单线程 | Phase 2 实体并行 + Phase 3 域并行 |
| unsafe | 1 处 | 零 |
| 仿真循环 | 5 阶段 | 5 阶段（语义更清晰） |
| 快照/回放 | 无 | 完整支持（所有状态在 ComponentStorage） |
| 分布式 | 不支持 | 结构化 EntityId + Ghost 实体 |

---

## 迁移路径

| 变化点 | 现有代码 | 改变后 |
|--------|---------|--------|
| 组件定义 | `impl_component!(Pos, "pos")` | `#[derive(Component, Clone)]` |
| 域 trait | `impl DomainRules` | `impl Domain` |
| 域样板 | `try_attach()` + `as_any()` + `on_attach()` | 全部删除 |
| 域声明 | `dependencies()` 返回字符串 | `type Writes/Reads/After` |
| compute 签名 | `fn compute(&mut self, ctx: &mut DomainContext)` | `fn compute(&mut self, ctx: &mut DomainContext<Self>, dt: f64)` |
| 组件读写 | `ctx.entities.get_mut(id)` | `ctx.get_mut::<T>(id)` |
| 服务注册 | 无 | `.with_domain(MyService::new())` |
| 实体定义 | 纯数据 | `impl Entity for T`（可选 tick + bundle） |
| World 配置 | `builder().with_domain("name", rules)` | `builder().with_domain(MyDomain)` |
| 事件处理 | `step_with(dt, \|event, world\| {})` | 不变 |

---

*最后更新：2026-03-30*
*状态：设计草案，待正式评审*
