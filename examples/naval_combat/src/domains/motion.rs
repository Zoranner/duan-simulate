//! 运动域
//!
//! 负责积分更新位置和速度。
//! 若实体持有 Seeker 组件，先计算朝目标的转向再积分（保持速度大小不变）。

use duan::{domain_rules_any, DomainContext, DomainRules, Entity, EntityId};

use crate::components::{Position, Seeker, Velocity};

pub struct MotionRules;

impl MotionRules {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MotionRules {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainRules for MotionRules {
    fn compute(&mut self, ctx: &mut DomainContext) {
        let dt = ctx.dt;
        let entity_ids: Vec<EntityId> = ctx.own_entity_ids().collect();

        for entity_id in entity_ids {
            // 只读阶段：提取当前位置、速度、可选的追踪目标
            let state = {
                let entity = match ctx.entities.get(entity_id) {
                    Some(e) => e,
                    None => continue,
                };
                let pos = match entity.get_component::<Position>() {
                    Some(p) => (p.x, p.y),
                    None => continue,
                };
                let vel = match entity.get_component::<Velocity>() {
                    Some(v) => (v.vx, v.vy),
                    None => continue,
                };
                let seeker = entity.get_component::<Seeker>().map(|s| s.target_id);
                (pos, vel, seeker)
            };

            let ((px, py), (vx, vy), seeker_target) = state;

            // 若有追踪目标，重新计算速度方向（保持速率）
            let (new_vx, new_vy) = if let Some(target_id) = seeker_target {
                let target_pos = ctx
                    .entities
                    .get(target_id)
                    .and_then(|e| e.get_component::<Position>())
                    .map(|p| (p.x, p.y));

                if let Some((tx, ty)) = target_pos {
                    let dx = tx - px;
                    let dy = ty - py;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist > 1e-6 {
                        let speed = (vx * vx + vy * vy).sqrt();
                        (dx / dist * speed, dy / dist * speed)
                    } else {
                        (vx, vy)
                    }
                } else {
                    (vx, vy)
                }
            } else {
                (vx, vy)
            };

            // 可变阶段：写回积分结果
            if let Some(entity) = ctx.entities.get_mut(entity_id) {
                if let Some(pos) = entity.get_component_mut::<Position>() {
                    pos.x = px + new_vx * dt;
                    pos.y = py + new_vy * dt;
                }
                if let Some(vel) = entity.get_component_mut::<Velocity>() {
                    vel.vx = new_vx;
                    vel.vy = new_vy;
                }
            }
        }
    }

    fn try_attach(&self, entity: &Entity) -> bool {
        entity.has_component::<Position>() && entity.has_component::<Velocity>()
    }

    fn on_detach(&mut self, _entity_id: EntityId) {}

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    domain_rules_any!(MotionRules);
}
