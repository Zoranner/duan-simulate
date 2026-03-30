# DUAN 下一代架构设计

---

## 设计哲学

### 核心原则

```
域是状态数据的权威。实体是意志的主体。
```

- **三元数据语义**：所有数据按语义归入 `Memory`、`Intent`、`State` 三类，写入权限由类型本身决定，编译期强制
- **写入独占**：一个 `State` 类型最多被一个域声明独占写入，`build()` 时校验
- **依赖显式**：`type Writes/Reads/After` 三个关联类型，全部编译期验证
- **事件驱动跨边界**：事件可序列化，支持回放与分布式
- **算法可替换**：换 Domain 实现不影响其他代码

### 三个编程原语

框架暴露给用户的编程单元只有三个：

| 概念 | 本质 | 用户做什么 |
|------|------|-----------|
| Component | 纯数据，按语义实现 `Memory`/`Intent`/`State` | 定义数据结构 |
| Entity | 零大小标记类型 | 定义实体行为 + 默认数据 |
| Domain | `State` 数据的权威 | 定义跨实体逻辑 |

### 三元数据语义

所有附加在实体上的数据按语义归入三类，写入权限编译期强制：

```
记忆（Memory）：实体的内化认知，绝对私有。
               外部既不能读也不能写，是"我是谁"的载体——
               积累的经验、内部推理的中间结果。

意图（Intent）：实体向世界表达的欲望与意志。
               由实体写入，向 Domain 公开（只读），
               是"我想做什么"的声明。

状态（State）：实体在世界法则下的客观条件。
               由 Domain 写入，反映物理现实。
               实体只能感知（从上帧快照读取），不能主动改变。
```

**访问矩阵（编译期强制）：**

| | Entity（自身） | Entity（经 `world()`） | `Domain::get_mut` | `Domain::get` |
|---|---|---|---|---|
| Memory | 读/写 | 不可见 | 编译错误 | 编译错误 |
| Intent | 读/写 | 只读（快照） | 编译错误 | ✓（在 Reads 中） |
| State | 只读（快照） | 只读（快照） | ✓（在 Writes 中） | ✓（在 Reads 中） |

```rust
// 用户声明数据语义，不需要其他样板代码
#[derive(Clone)] pub struct SoldierMemory { threats: Vec<ThreatInfo>, phase: AIPhase }
impl Memory for SoldierMemory {}

#[derive(Clone)] pub struct MovementOrder { pub desired_velocity: Vec3 }
impl Intent for MovementOrder {}

#[derive(Clone)] pub struct Position { pub x: f64, pub y: f64, pub z: f64 }
impl State for Position {}
```

### 数据流

```
Entity::tick()  →  Intent（意图）  →  Domain::compute()  →  State（状态）
（单向不可逆，编译期保证）
```

- Entity 可写 `Memory`（私有认知）和 `Intent`（表达意志），**不能写 `State`**（编译期阻止）
- Domain 只能写 `type Writes` 声明的 `State` 类型，读 `Intent`/`State`（编译期保证）
- `Memory` 对外完全不可见，Domain 无法访问任何实体的 Memory

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

### Component：三元语义，按类型密集存储

`Component` 是所有实体数据的统一 supertrait（sealed，用户不直接实现），用户只与三个语义 sub-trait 打交道：

```rust
// 统一约束（sealed，用户不直接实现，只通过 Memory/Intent/State 间接获得）
pub trait Component: Send + Sync + Clone + 'static {}

// 用户声明语义的三个公开 trait
pub trait Memory: Component {}   // 实体私有认知
pub trait Intent: Component {}   // 实体意志表达
pub trait State:  Component {}   // 世界客观状态

// EntityWritable：Entity 可写的数据类型（Memory + Intent，不含 State）
// 框架通过 blanket impl 自动提供，用户无需关注
pub trait EntityWritable: Component {}
impl<T: Memory> EntityWritable for T {}
impl<T: Intent> EntityWritable for T {}
```

用户只需声明语义，零样板代码：

```rust
#[derive(Clone)] pub struct SoldierMemory { threats: Vec<ThreatInfo>, phase: AIPhase }
impl Memory for SoldierMemory {}

#[derive(Clone)] pub struct MovementOrder { pub desired_velocity: Vec3 }
impl Intent for MovementOrder {}

#[derive(Clone)] pub struct Position { pub x: f64, pub y: f64, pub z: f64 }
impl State for Position {}

#[derive(Clone)] pub struct Health { pub value: f64, pub max: f64 }
impl State for Health {}
```

**存储设计（内部，按类型密集数组）：**

```
ComponentStorage<Position>:
  dense: Vec<Position>               连续内存，缓存友好
  entity_to_slot: SparseVec<u32>     EntityId.local_index → 槽位
  slot_to_entity: Vec<EntityId>      槽位 → EntityId（反查）
```

- 十万个 Position = 一块 2.4MB 连续内存，L2 缓存全命中
- Memory/Intent/State 统一存储结构，Memory 仅在 WorldSnapshot 中不可见

---

### Entity：零大小标记类型

Entity 没有字段。所有数据都是 `Memory`/`Intent`/`State`。Entity 只做两件事：定义行为逻辑、声明默认数据。

```rust
pub trait Entity: 'static {
    /// 每帧调用。静态方法，无 &self。
    /// 读自身 Memory/Intent 做决策，读快照感知 State，写 Memory 更新认知，写 Intent 表达意志。
    fn tick(ctx: &mut EntityContext) {}

    /// 此实体类型自带的默认数据
    fn bundle() -> impl ComponentBundle { () }
}
```

**用法：**

```rust
pub struct Soldier;

impl Entity for Soldier {
    fn tick(ctx: &mut EntityContext) {
        let mem = ctx.get::<SoldierMemory>().unwrap();  // Memory，读当前帧值
        let pos = ctx.get::<Position>().unwrap();        // State，读上帧快照

        // 通过快照感知周围世界（只能看到 Intent 和 State，看不到其他实体的 Memory）
        let threats = ctx.world().query_nearby::<Position>(pos, 500.0);
        let new_mem = decide_threats(mem, &threats);

        // 写 Memory（更新私有认知）和 Intent（表达意志）
        // 写 State（Position、Health 等）→ 编译错误
        ctx.set(new_mem);
        ctx.set(MovementOrder { desired_velocity: flee_direction(pos, &threats) });
        ctx.set(AttackIntent::target(closest_threat(&threats)));
    }

    fn bundle() -> impl ComponentBundle {
        (
            Health::new(100),      // State
            SoldierMemory::new(),  // Memory
            FactionTag::Blue,      // State
        )
    }
}

#[derive(Clone)]
pub struct SoldierMemory { threats: Vec<ThreatInfo>, phase: AIPhase }
impl Memory for SoldierMemory {}
```

**无行为实体：**

```rust
pub struct Obstacle;
impl Entity for Obstacle {
    fn bundle() -> impl ComponentBundle {
        (BlockingRadius(10.0),)  // State
    }
}
```

**生成实体：**

```rust
// bundle() 定义的数据自动附加，只传个性化参数
let id = world.spawn(Soldier)
    .with(Position::new(100.0, 0.0, 50.0))
    .id();
```

**Entity 为什么没有字段：**

| 如果 Entity 有字段 | 如果 Entity 无字段 |
|---|---|
| 框架看不到私有状态 → 快照断裂 | 所有数据都是 Memory/Intent/State → 快照可管理 |
| `Box<dyn Entity>` 存储 → 缓存不友好 | 函数指针存储 → 8 字节/实体 |
| tick 读写同一存储 → 确定性未定义 | 冻结快照 + pending buffer → 天然确定 |
| tick 可并行性差 | embarrassingly parallel |

**私有性由类型系统保证**：`SoldierMemory: Memory` 决定了它对 Domain 和 `WorldSnapshot` 完全不可见，不依赖任何架构约定或 debug 检查。

---

### EntityContext：实体 tick 的访问入口

```rust
pub struct EntityContext<'w> {
    entity_id: EntityId,
    own: OwnComponentView<'w>,    // 当前帧的自身数据（Memory/Intent 可读写，State 只读快照）
    snapshot: &'w WorldSnapshot,  // 上一帧末冻结快照（只暴露 Intent 和 State，Memory 不可见）
    events: EventEmitter<'w>,
    pub clock: &'w TimeClock,
    pub dt: f64,
}

impl EntityContext<'_> {
    pub fn id(&self) -> EntityId;

    /// 读取自身数据
    /// - Memory/Intent：读当前帧值
    /// - State：读上帧快照值（一帧延迟，符合"感知-决策-行动"时序）
    pub fn get<T: Component>(&self) -> Option<&T>;

    /// 写入自身数据（编译期：T 必须是 Memory 或 Intent，State 写入无法通过编译）
    pub fn set<T: EntityWritable>(&mut self, value: T);
    pub fn remove<T: EntityWritable>(&mut self);

    /// 上一帧末的冻结快照（只读，仅暴露 Intent 和 State，Memory 不可见）
    pub fn world(&self) -> &WorldSnapshot;

    /// 发出事件（延迟到 Phase 4 处理）
    pub fn emit<E: CustomEvent>(&mut self, event: E);

    /// 生命周期命令（延迟到 Phase 5 执行）
    pub fn spawn(&mut self, entity: impl Entity) -> EntityId;
    pub fn destroy(&mut self, id: EntityId);
}
```

**编译期安全保证：**

- `ctx.set(Position { ... })` → 编译错误（`Position: State`，不是 `EntityWritable`）
- `ctx.world().get::<SoldierMemory>(other_id)` → 编译错误（`Memory` 不在 `WorldSnapshot` 的可见范围）

**确定性保证：**

- `ctx.world()` 返回的是 Phase 2 开始前冻结的快照，所有实体看到相同数据
- `ctx.set()` 写入 pending buffer，Phase 2 结束后统一 flush
- Entity tick 顺序不影响结果 → 可用 rayon 并行

---

### WorldSnapshot：上帧冻结快照

```rust
pub struct WorldSnapshot { /* 内部：Intent + State 的只读视图，不含 Memory */ }

impl WorldSnapshot {
    /// 读取任意实体的 Intent 或 State（Memory 类型不可查询，编译期阻止）
    pub fn get<T: Component>(&self, id: EntityId) -> Option<&T>
        where T: Intent + State;  // 仅 Intent 和 State

    /// 空间查询（需注册 SpatialIndex 服务域）
    pub fn query_nearby<T: State>(&self, center: &Position, radius: f64) -> Vec<EntityId>;

    /// 查询所有持有某 Intent 或 State 的实体
    pub fn each<T: Component>(&self) -> impl Iterator<Item = (EntityId, &T)>;
}
```

- Phase 2 开始前由框架冻结，所有实体的 tick 共享同一份不可变快照
- Memory 在存储层存在，但 `WorldSnapshot` 的类型接口从不暴露 `Memory` 类型

---

### Domain：State 数据的权威

```rust
pub trait Domain: Send + Sync + 'static {
    /// 独占写入的 State 类型（此域是这些状态的唯一权威）
    type Writes: StateSet = ();

    /// 共享只读的数据类型（Intent 或 State，Memory 无法放入）
    type Reads: SharedSet = ();

    /// 必须在本域之前完成的域（编译期类型检查）
    type After: DomainSet = ();

    /// 每帧计算
    fn compute(&mut self, ctx: &mut DomainContext<Self>, dt: f64) {}
}
```

**五行定义，三个关联类型，一个方法。**

没有 `on_attach`、没有 `on_detach`、没有 `try_attach`、没有 `as_any`。新实体在 Phase 5 生效，下一帧 compute 自动看到。

编译期约束：

- `type Writes` 只接受 `State` 类型（`StateSet` 是 State 元组的约束）
- `type Reads` 接受 `Intent` 或 `State` 类型（`SharedSet`），Memory 无法放入
- 若两个域声明相同 Writes 类型 → `build()` 时 panic

**域的语义角色（全部是同一 trait）：**

| 角色 | Writes | 典型示例 |
|------|--------|---------|
| 交互域 | 仲裁结果（State） | `DetectionDomain`、`CombatDomain` |
| 执行域 | 物理状态（State） | `MotionDomain`、`KinematicsDomain` |
| 服务域 | `()` | `SpatialIndex`、`EquipmentDatabase` |

分类是语义指导，不是不同的类型。

---

### DomainContext：域 compute 的访问入口

```rust
pub struct DomainContext<'w, D: Domain> {
    writes: WriteView<'w, D::Writes>,  // 对 State 的独占写视图
    reads:  ReadView<'w, D::Reads>,    // 对 Intent/State 的共享读视图
    events: EventEmitter<'w>,
    pub clock: &'w TimeClock,
    pub dt: f64,
}

impl<D: Domain> DomainContext<'_, D> {
    /// 迭代本域管辖的实体（拥有 Writes 中全部 State 类型的实体）
    pub fn entities(&self) -> impl Iterator<Item = EntityId>;

    /// 按数据类型迭代（编译期：T 必须在 Reads 中）
    pub fn each<T: InReads<D>>(&self) -> impl Iterator<Item = (EntityId, &T)>;

    /// 写本域管辖实体的 State（编译期：T 必须在 Writes 中）
    pub fn get_mut<T: InWrites<D>>(&mut self, id: EntityId) -> Option<&mut T>;

    /// 读任意实体的 Intent 或 State（编译期：T 必须在 Reads 中）
    pub fn get<T: InReads<D>>(&self, id: EntityId) -> Option<&T>;

    /// 跨域查询（编译期：OtherD 必须在 D::After 中，即已执行完毕）
    pub fn domain<OtherD: Domain>(&self) -> Option<&OtherD>
        where D::After: ContainsDomain<OtherD>;

    /// 发出事件（延迟到 Phase 4 处理）
    pub fn emit<E: CustomEvent>(&mut self, event: E);

    /// 生命周期命令（延迟到 Phase 5 执行）
    pub fn spawn(&mut self, entity: impl Entity) -> EntityId;
    pub fn destroy(&mut self, id: EntityId);
}
```

---

### Intent 设计原则

Intent **按域设计**，不按实体设计。所有想移动的实体输出同一种意图：

```
Soldier::tick()  → 寻路算法  → MovementOrder { desired_velocity }
Missile::tick()  → 制导算法  → MovementOrder { desired_velocity }
Vehicle::tick()  → 编队算法  → MovementOrder { desired_velocity }
                                    ↓
                            MotionDomain::compute()（读 Intent，写 State）
                                    ↓
                              Position, Velocity（State）
```

```rust
#[derive(Clone)]
pub struct MovementOrder { pub desired_velocity: Vec3 }
impl Intent for MovementOrder {}

#[derive(Clone)]
pub struct AttackIntent { pub target: Option<EntityId>, pub weapon_id: u32 }
impl Intent for AttackIntent {}
```

新增 Entity 类型**不需要修改任何 Domain**。差异在 Entity::tick() 内部消化，Intent 是统一接口。

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
            let desired = ctx.get::<MovementOrder>(id).map(|o| o.desired_velocity);

            let vel = ctx.get_mut::<Velocity>(id).unwrap();
            if let Some(v) = desired { vel.0 = v; }
            let new_vel = vel.0;

            let pos = ctx.get_mut::<Position>(id).unwrap();
            pos.x += new_vel.x * dt;
            pos.y += new_vel.y * dt;
            pos.z += new_vel.z * dt;
        }
    }
}
```

### 服务域：SpatialIndex

```rust
pub struct SpatialIndex { grid: SpatialHash }

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
pub struct EquipmentDatabase { specs: HashMap<String, EquipmentSpec> }

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
         冻结数据快照（Intent + State，所有实体共享同一份只读视图）
         遍历所有实体，调用 Entity::tick()
         写入进 pending buffer → Phase 2 结束后统一 flush
         可用 rayon 并行（实体间无数据竞争）

Phase 3  域计算
         按预构建 DAG 计划执行
         同 Level 内并行，Level 间有 barrier
         读 Reads 数据 → 计算 → 写 Writes State

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
- Phase 3 同 Level 域读同一数据，DAG 保证无写冲突
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
    .build();  // 静态分析 Writes/Reads/After，构建 DAG，检测冲突

// 生成实体
let id = world.spawn(Soldier)
    .with(Position::new(100.0, 0.0, 50.0))
    .id();

// 仿真步进
world.step(dt);
world.step_with(dt, |event, world| { ... });
let events = world.step_collect(dt);

// 快照与回放（State + Intent 完整捕获）
let snap = world.snapshot();
world.restore(snap);

// 查询
world.sim_time() -> f64
world.entity_count() -> usize
```

---

## 团队开发模式

```
src/
  entities/           ← 每人独立开发，互不影响
    soldier.rs        Alice: Soldier + SoldierMemory（Memory）
    tank.rs           Bob:   Tank + TankMemory（Memory）
    missile.rs        Carol: Missile + GuidanceMemory（Memory）
    obstacle.rs       Dave:  Obstacle（纯数据，无 tick）
  domains/            ← 架构师/框架组维护
    motion.rs         MotionDomain（执行域）
    detection.rs      DetectionDomain（交互域）
    combat.rs         CombatDomain（交互域）
    spatial.rs        SpatialIndex（服务域）
    equipment.rs      EquipmentDatabase（服务域）
  components/         ← 共享组件定义
    intents.rs        MovementOrder, AttackIntent, ...（impl Intent）
    states.rs         Position, Velocity, Health, ...（impl State）
  main.rs             组装 World
```

**新增 Entity 类型不修改任何 Domain。新增 Domain 不修改任何 Entity。**

---

## 分布式扩展

**核心约束：** 每个 State 类型在任意时刻只有一个节点是它的权威。

**Ghost 实体机制：**

```
Node A（西北战区，entities 1–50000）
  拥有：Position、Velocity（本地 MotionDomain 管辖）
  读取：Node B 的边界实体 State 快照（只读 Ghost）

Node B（东南战区，entities 50001–100000）
  拥有：Position、Velocity（本地 MotionDomain 管辖）
  读取：Node A 的边界实体 State 快照（只读 Ghost）

边界同步：每帧同步 Ghost 实体的 State/Intent 快照
跨节点事件：通过 DistributedEvent 序列化路由
```

EntityId 的 `node_id` 字段直接告知管辖节点。

---

## 与当前版本对比

| 维度 | 当前版本 | 终态版本 |
|------|---------|---------|
| 核心哲学 | 域管辖一组实体 | 域管辖 State 数据，Entity 管辖 Memory+Intent |
| 编程原语 | Entity + Component + DomainRules | Entity + Memory/Intent/State + Domain |
| 数据语义 | 无区分，统一为 Component | Memory（私有）/ Intent（意志）/ State（客观） |
| Entity 本质 | 数据容器（HashMap 存组件） | 零大小标记类型（tick 函数 + bundle） |
| Entity 数据 | struct 字段 + 组件混合 | 全部是 Memory/Intent/State，无 struct 字段 |
| Entity 写入边界 | 无检查（可写任意组件） | 编译期：只能写 Memory 和 Intent，State 写入编译失败 |
| Domain trait | 7 个方法 + as_any | 1 个方法 + 3 个关联类型（Writes: StateSet） |
| 服务概念 | 无 | `Writes = ()` 的域 |
| EntityId | 本地自增 u64 | 结构化 64 位（node+generation+index） |
| 数据存储 | 每实体 HashMap + Box | 按类型密集数组（Memory/Intent/State 统一存储） |
| 写入边界 | 运行时检查 | 编译期类型保证（三元语义 + InWrites/InReads） |
| 确定性 | 依赖执行顺序 | 冻结快照 + DAG，顺序无关 |
| 并行执行 | 单线程 | Phase 2 实体并行 + Phase 3 域并行 |
| unsafe | 1 处 | 零 |
| 仿真循环 | 5 阶段（无 Entity tick） | 5 阶段（Phase 2 新增 Entity tick） |
| 快照/回放 | 无 | 完整支持（State + Intent 完整捕获） |
| 分布式 | 不支持 | 结构化 EntityId + Ghost 实体 |

---

## 迁移路径

| 变化点 | 现有代码 | 改变后 |
|--------|---------|--------|
| 数据定义 | `impl_component!(Pos, "pos")` | `#[derive(Clone)] struct Pos; impl State for Pos {}` |
| 数据语义 | 无区分 | 明确声明 `Memory` / `Intent` / `State` |
| 域 trait | `impl DomainRules` | `impl Domain` |
| 域样板 | `try_attach()` + `as_any()` + `on_attach()` | 全部删除 |
| 域声明 | `dependencies()` 返回字符串 | `type Writes: StateSet`（只能填 State 类型） |
| compute 签名 | `fn compute(&mut self, ctx: &mut DomainContext)` | `fn compute(&mut self, ctx: &mut DomainContext<Self>, dt: f64)` |
| 数据读写 | `ctx.entities.get_mut(id)` | `ctx.get_mut::<T>(id)` |
| 服务注册 | 无 | `.with_domain(MyService::new())` |
| 实体定义 | 纯数据 struct | `impl Entity for T`（可选 tick + bundle） |
| World 配置 | `builder().with_domain("name", rules)` | `builder().with_domain(MyDomain)` |
| 事件处理 | `step_with(dt, \|event, world\| {})` | 不变 |

---

*最后更新：2026-03-30*
