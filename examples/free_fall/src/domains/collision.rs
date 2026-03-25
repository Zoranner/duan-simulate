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
//! # 执行顺序
//!
//! 碰撞域声明依赖运动域：`dependencies()` 返回 `["motion"]`。

use duan::{
    DomainContext, DomainEvent, DomainRules, Entity, EntityId,
};
use std::any::Any;
use std::collections::HashMap;

use crate::components::{Collider, Position, Velocity};

/// 碰撞域规则
pub struct CollisionRules {
    /// 上一帧位置记录（EntityId → 上一帧的 y 坐标）
    /// 用于穿越检测：若 prev_y > ground 且 curr_y <= ground，则发生穿越
    prev_y: HashMap<EntityId, f64>,
}

impl CollisionRules {
    pub fn new() -> Self {
        Self {
            prev_y: HashMap::new(),
        }
    }

    /// 从域上下文中查找地面（静态碰撞体：Collider 且无 Velocity）
    fn find_ground_info(ctx: &DomainContext) -> Option<(String, f64, f64, f64)> {
        let domain_name = {
            let domain = ctx.get_domain_by_name("collision")?;
            domain.name.clone()
        };
        let entity_ids: Vec<EntityId> = {
            let domain = ctx.get_domain_by_name(&domain_name)?;
            domain.entity_ids().collect()
        };
        for entity_id in entity_ids {
            let entity = ctx.entities.get(entity_id)?;
            let collider = entity.get_component::<Collider>()?;
            if entity.get_component::<Velocity>().is_some() {
                continue; // 跳过动态碰撞体
            }
            let pos_y = entity.get_component::<Position>().map(|p| p.y).unwrap_or(0.0);
            return Some((
                collider.name.clone(),
                pos_y + collider.offset_y,
                collider.restitution,
                collider.friction,
            ));
        }
        None
    }
}

impl Default for CollisionRules {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainRules for CollisionRules {
    fn compute(&mut self, ctx: &mut DomainContext, _dt: f64) {
        // 查找地面参数
        let (ground_name, ground_height, restitution, friction) =
            match Self::find_ground_info(ctx) {
                Some((name, h, r, f)) => (name, h, r, f),
                None => return,
            };

        // 收集动态实体 ID
        let entity_ids: Vec<EntityId> = {
            let domain = ctx.get_domain_by_name("collision").expect("collision domain must exist");
            domain.entity_ids().collect()
        };

        for entity_id in &entity_ids {
            // 读取运动后的状态（motion 已在前面执行，vy 已更新）
            let curr_y: f64;
            let curr_vy: f64;
            {
                let entity = match ctx.entities.get(*entity_id) {
                    Some(e) => e,
                    None => continue,
                };
                if entity.get_component::<Velocity>().is_none() {
                    continue; // 跳过静态碰撞体
                }
                let pos = match entity.get_component::<Position>() {
                    Some(p) => p,
                    None => continue,
                };
                let vel = match entity.get_component::<Velocity>() {
                    Some(v) => v,
                    None => continue,
                };
                curr_y = pos.y;
                curr_vy = vel.vy;
            }

            // 获取上一帧的位置（用于穿越检测）
            // 第一次见到此实体时，prev_y 等于当前位置（不会误触发穿越）
            let prev_y = *self.prev_y.entry(*entity_id).or_insert(curr_y);

            // 穿越检测：若 prev_y > ground 且 curr_y <= ground，说明从上方穿越了地面
            let crossed = prev_y > ground_height && curr_y <= ground_height;

            // 磁滞清除：若 vy > 0（正在上升），清除 prev_y 记录
            // 此时 curr_y > ground_height，碰撞不会触发，直到下一次下落穿越
            if curr_vy > 0.0 {
                self.prev_y.remove(entity_id);
            }

            if crossed {
                // 更新记录（用于下次判断）
                self.prev_y.insert(*entity_id, curr_y);

                // 冲击速度 = motion 更新后的 vy（向下为负）
                let impact_vy = curr_vy;
                // 反弹速度向上（正），大小为冲击速度乘以弹性系数
                let bounce_vy = -impact_vy * restitution;

                // 修正实体状态：位置修正到地面，速度反向
                if let Some(entity) = ctx.entities.get_mut(*entity_id) {
                    if let Some(pos) = entity.components.get_mut::<Position>() {
                        pos.y = ground_height;
                    }
                    if let Some(vel) = entity.components.get_mut::<Velocity>() {
                        vel.vx *= 1.0 - friction;
                        vel.vy = bounce_vy;
                        vel.vz *= 1.0 - friction;
                    }
                }

                // 发出碰撞事件
                ctx.emit(DomainEvent::Custom(Box::new(crate::events::GroundCollisionEvent::new(
                    *entity_id,
                    ground_name.clone(),
                    impact_vy.abs(),
                    restitution,
                    friction,
                ))));
            } else if curr_y > ground_height {
                // 未穿越时，也更新 prev_y（用于下次判断）
                self.prev_y.insert(*entity_id, curr_y);
            }
        }
    }

    fn try_attach(&mut self, entity: &Entity) -> bool {
        let has_pos = entity.has_component::<Position>();
        let has_collider = entity.has_component::<Collider>();
        let has_vel = entity.get_component::<Velocity>();
        // 动态碰撞体（pos + vel + collider）或静态碰撞体（pos + collider）
        (has_pos && has_vel.is_some() && has_collider)
            || (has_pos && has_collider && has_vel.is_none())
    }

    fn on_detach(&mut self, entity_id: EntityId) {
        self.prev_y.remove(&entity_id);
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["motion"]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
