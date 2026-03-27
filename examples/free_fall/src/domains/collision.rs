//! 碰撞域
//!
//! 负责碰撞检测和响应，直接修改实体的位置和速度，并发出碰撞事件。
//! 体现了框架中"域负责跨边界通知"的设计原则：
//! - 速度修正是域的权威操作，直接写组件
//! - 碰撞发生是对外的事实通告，通过事件发出
//!
//! # 穿越检测
//!
//! 离散时间步进中，若步长较大，球可能在单帧内"穿透"地面（一帧前在上方，一帧后在下方）。
//! 本域通过记录**上一帧位置**（`prev_y`）检测穿越：
//! ```text
//! 穿越条件：prev_y > ground_height AND curr_y <= ground_height
//! ```
//! 检测到穿越时，将位置修正回地面并反转速度（乘弹性系数）。
//!
//! # 磁滞（重复触发防护）
//!
//! 由于弹跳后球会多次在地面附近徘徊，单纯的穿越检测可能在同一次弹跳中反复触发。
//! `prev_y` 映射兼作磁滞机制：
//!
//! - 穿越发生时：`prev_y[id] = curr_y`（锁定当前位置，阻断下一帧再次触发）
//! - 球向上弹起（`vy > 0`）时：清除 `prev_y[id]`，允许下次下落重新检测
//! - 球在地面以上继续下落时：`prev_y[id] = curr_y`，追踪最新下落位置
//!
//! # 地面实体缓存
//!
//! 本示例仅支持**单一静态地面**。地面在 `on_attach` 时通过 `StaticBody` 标记识别，
//! ID 缓存在域内部状态中，每帧直接读取，无需遍历查询。
//!
//! **约束**：若有多个 `StaticBody` 实体，后附加者会覆盖先前的缓存，
//! 只有最后被附加的静态体才有效。更复杂的场景需维护地面列表（超出本示例范围）。
//!
//! # 执行顺序
//!
//! 碰撞域通过 `dependencies()` 声明依赖运动域，框架拓扑排序保证运动域每帧先执行。
//! 碰撞域看到的 Position/Velocity 是本帧 motion 积分后的"新鲜"值。

use duan::{domain_rules_any, DomainContext, DomainEvent, DomainRules, Entity, EntityId};
use std::collections::HashMap;

use crate::components::{Collider, Position, StaticBody, Velocity};

/// 碰撞域规则
pub struct CollisionRules {
    /// 静态碰撞体（地面）的实体 ID，在 on_attach 时识别并缓存
    ///
    /// 只保存最后一个附加的 StaticBody 实体，本示例仅支持单一地面。
    ground_id: Option<EntityId>,
    /// 上一帧 y 坐标记录（EntityId → prev_y）
    ///
    /// 同时承担两个职责：
    /// 1. 穿越检测：比较 prev_y 与 curr_y 判断是否穿越地面
    /// 2. 磁滞防护：通过插入/清除控制碰撞触发窗口
    prev_y: HashMap<EntityId, f64>,
}

impl CollisionRules {
    pub fn new() -> Self {
        Self {
            ground_id: None,
            prev_y: HashMap::new(),
        }
    }
}

impl Default for CollisionRules {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainRules for CollisionRules {
    fn compute(&mut self, ctx: &mut DomainContext) {
        // 读取地面参数；无地面则跳过本帧
        let ground_id = match self.ground_id {
            Some(id) => id,
            None => return,
        };
        let (ground_height, restitution, friction) = {
            let entity = match ctx.entities.get(ground_id) {
                Some(e) => e,
                None => return,
            };
            let collider = match entity.get_component::<Collider>() {
                Some(c) => c,
                None => return,
            };
            let pos_y = entity.get_component::<Position>().map_or(0.0, |p| p.y);
            (
                pos_y + collider.offset_y,
                collider.restitution,
                collider.friction,
            )
        };

        // 收集动态实体 ID（排除地面自身——地面有 StaticBody，不参与碰撞响应）
        let entity_ids: Vec<EntityId> =
            ctx.own_entity_ids().filter(|&id| id != ground_id).collect();

        for entity_id in entity_ids {
            // 读取运动域更新后的状态（motion 已在本帧前执行）
            let (curr_y, curr_vy) = {
                let entity = match ctx.entities.get(entity_id) {
                    Some(e) => e,
                    None => continue,
                };
                let pos = match entity.get_component::<Position>() {
                    Some(p) => p,
                    None => continue,
                };
                // 无 Velocity 的实体是静态体，无需碰撞响应（防御性检查，实际上
                // try_attach 已要求动态实体必须有 Velocity）
                let vel = match entity.get_component::<Velocity>() {
                    Some(v) => v,
                    None => continue,
                };
                (pos.y, vel.vy)
            };

            // 获取上一帧位置：首次见到此实体时用 curr_y 初始化，不会误触发穿越检测
            let prev_y = *self.prev_y.entry(entity_id).or_insert(curr_y);

            // 穿越检测：上一帧在地面以上，本帧已到达或穿过地面
            let crossed = prev_y > ground_height && curr_y <= ground_height;

            // 更新磁滞状态（独立于碰撞响应，每帧都要执行）
            if curr_vy > 0.0 {
                // 球已向上弹起：清除历史位置，允许下次下落重新进入穿越检测窗口
                self.prev_y.remove(&entity_id);
            } else if curr_y > ground_height {
                // 球在地面以上且仍在下落：追踪最新位置
                self.prev_y.insert(entity_id, curr_y);
            }
            // 注意：curr_y <= ground_height 时不更新 prev_y，
            // 保持锁定状态直到球向上弹起（vy > 0）后清除

            if crossed {
                // 防止本帧内重复触发：将 prev_y 锁定在地面位置
                self.prev_y.insert(entity_id, curr_y);

                // 弹跳速度：反向并乘弹性系数（restitution=0 完全非弹性，=1 完全弹性）
                let bounce_vy = -curr_vy * restitution;

                // 修正实体状态（域的权威写入）：
                // - 位置修正到地面，避免球陷入地面以下
                // - 速度反向衰减，同时施加水平摩擦
                if let Some(entity) = ctx.entities.get_mut(entity_id) {
                    if let Some(pos) = entity.get_component_mut::<Position>() {
                        pos.y = ground_height;
                    }
                    if let Some(vel) = entity.get_component_mut::<Velocity>() {
                        vel.vx *= 1.0 - friction;
                        vel.vy = bounce_vy;
                        vel.vz *= 1.0 - friction;
                    }
                }

                // 读取地面名称用于事件数据（仅在碰撞发生时读取，避免每帧 clone）
                let surface_name = ctx
                    .entities
                    .get(ground_id)
                    .and_then(|e| e.get_component::<Collider>())
                    .map(|c| c.name.clone())
                    .unwrap_or_default();

                // 发出碰撞事件（跨边界通知，让外部观察者知道发生了碰撞）
                // 事件包含碰撞参数，让接收方无需再查询域状态即可使用
                ctx.emit(DomainEvent::custom(
                    crate::events::GroundCollisionEvent::new(
                        entity_id,
                        surface_name,
                        curr_vy.abs(), // impact_velocity 始终为正值（碰撞前的速度大小）
                        restitution,
                        friction,
                    ),
                ));
            }
        }
    }

    /// 准入条件：有 Position 和 Collider 即可加入碰撞域
    ///
    /// 静态体（Position + Collider + StaticBody）和动态体（Position + Collider + Velocity）
    /// 都满足此条件，通过 `on_attach` 中的 `StaticBody` 检查区分两者。
    fn try_attach(&self, entity: &Entity) -> bool {
        entity.has_component::<Position>() && entity.has_component::<Collider>()
    }

    /// 附加回调：识别静态体并缓存地面 ID
    fn on_attach(&mut self, entity: &Entity) {
        // 通过 StaticBody 标记显式识别静态表面，而不依赖"缺少 Velocity"的隐式推断。
        // 显式标记的优点：语义清晰，未来添加速度为零的动态物体时也不会误判。
        if entity.has_component::<StaticBody>() {
            self.ground_id = Some(entity.id);
        }
    }

    fn on_detach(&mut self, entity_id: EntityId) {
        self.prev_y.remove(&entity_id);
        if self.ground_id == Some(entity_id) {
            self.ground_id = None;
        }
    }

    /// 依赖声明：依赖运动域
    ///
    /// 框架据此在拓扑排序中将 motion 排在 collision 之前，
    /// 保证本域 compute 执行时能读取到本帧已更新的位置和速度。
    fn dependencies(&self) -> Vec<&'static str> {
        vec!["motion"]
    }

    domain_rules_any!(CollisionRules);
}
