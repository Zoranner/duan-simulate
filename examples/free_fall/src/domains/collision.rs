//! 碰撞域
//!
//! 负责碰撞检测和响应，直接修改实体的位置和速度，并发出碰撞事件。
//!
//! # 穿越检测（Crossing Detection）
//!
//! 碰撞域在运动域之后执行，此时实体 vy 已经过本帧 motion 积分。
//! 用上一帧位置（prev_y）检测是否穿越了地面：若 prev_y > ground 且 curr_y <= ground，
//! 说明球在两帧之间穿过了地面，需要触发碰撞响应。
//!
//! # 磁滞（Hysteresis）
//!
//! 用 bounced 映射避免同一碰撞反复触发：
//! - 碰撞发生时在 bounced 中记录 prev_y
//! - 只有当 vy > 0（motion 已将球向上推）才清除记录
//! - 清除后下一帧 vy 可能又变负，但此时球已远离地面，不会立即再撞
//!
//! # 地面实体缓存
//!
//! 地面在 `on_attach` 时识别并缓存 ID，`compute` 直接使用缓存，
//! 避免每帧遍历查询。
//!
//! # 执行顺序
//!
//! 碰撞域声明依赖运动域：`dependencies()` 返回 `["motion"]`。

use duan::{domain_rules_any, DomainContext, DomainEvent, DomainRules, Entity, EntityId};
use std::collections::HashMap;

use crate::components::{Collider, Position, Velocity};

/// 碰撞域规则
pub struct CollisionRules {
    /// 静态碰撞体（地面）的实体 ID，在 try_attach 时识别并缓存
    ground_id: Option<EntityId>,
    /// 上一帧位置记录（EntityId → 上一帧的 y 坐标）
    /// 用于穿越检测：若 prev_y > ground 且 curr_y <= ground，则发生穿越
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
        // 从缓存读取地面参数
        let (ground_name, ground_height, restitution, friction) = {
            let ground_id = match self.ground_id {
                Some(id) => id,
                None => return,
            };
            let entity = match ctx.entities.get(ground_id) {
                Some(e) => e,
                None => return,
            };
            let collider = match entity.get_component::<Collider>() {
                Some(c) => c,
                None => return,
            };
            let pos_y = entity
                .get_component::<Position>()
                .map(|p| p.y)
                .unwrap_or(0.0);
            (
                collider.name.clone(),
                pos_y + collider.offset_y,
                collider.restitution,
                collider.friction,
            )
        };

        // 收集动态实体 ID
        let entity_ids: Vec<EntityId> = ctx.own_entity_ids().collect();

        for entity_id in entity_ids {
            // 读取运动后的状态（motion 已在前面执行，vy 已更新）
            let (curr_y, curr_vy) = {
                let entity = match ctx.entities.get(entity_id) {
                    Some(e) => e,
                    None => continue,
                };
                if entity.get_component::<Velocity>().is_none() {
                    continue; // 跳过静态碰撞体（地面）
                }
                let pos = match entity.get_component::<Position>() {
                    Some(p) => p,
                    None => continue,
                };
                let vel = match entity.get_component::<Velocity>() {
                    Some(v) => v,
                    None => continue,
                };
                (pos.y, vel.vy)
            };

            // 获取上一帧位置（第一次见到此实体时等于当前位置，不误触发）
            let prev_y = *self.prev_y.entry(entity_id).or_insert(curr_y);

            // 穿越检测：从上方穿越了地面
            let crossed = prev_y > ground_height && curr_y <= ground_height;

            // 磁滞清除：正在上升时清除记录
            if curr_vy > 0.0 {
                self.prev_y.remove(&entity_id);
            }

            if crossed {
                self.prev_y.insert(entity_id, curr_y);

                let impact_vy = curr_vy;
                let bounce_vy = -impact_vy * restitution;

                // 修正实体状态：位置修正到地面，速度反向
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

                // 发出碰撞事件
                ctx.emit(DomainEvent::custom(
                    crate::events::GroundCollisionEvent::new(
                        entity_id,
                        ground_name.clone(),
                        impact_vy.abs(),
                        restitution,
                        friction,
                    ),
                ));
            } else if curr_y > ground_height {
                self.prev_y.insert(entity_id, curr_y);
            }
        }
    }

    fn try_attach(&self, entity: &Entity) -> bool {
        let has_pos = entity.has_component::<Position>();
        let has_collider = entity.has_component::<Collider>();
        // 静态碰撞体（地面）：Position + Collider，无 Velocity
        // 动态碰撞体（小球）：Position + Velocity + Collider
        has_pos && has_collider
    }

    fn on_attach(&mut self, entity: &Entity) {
        let has_vel = entity.has_component::<Velocity>();
        let has_pos = entity.has_component::<Position>();
        let has_collider = entity.has_component::<Collider>();

        if has_pos && has_collider && !has_vel {
            // 静态碰撞体识别为地面，缓存 ID
            self.ground_id = Some(entity.id);
        }
    }

    fn on_detach(&mut self, entity_id: EntityId) {
        self.prev_y.remove(&entity_id);
        if self.ground_id == Some(entity_id) {
            self.ground_id = None;
        }
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["motion"]
    }

    domain_rules_any!(CollisionRules);
}
