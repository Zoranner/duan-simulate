//! 运动域
//!
//! 负责纯运动学积分，直接修改实体的位置和速度。
//!
//! # 设计原则
//!
//! - **无状态**：运动域本身不维护任何状态，所有状态都在实体的组件中
//! - **确定性**：使用半隐式欧拉积分，稳定性优于标准欧拉
//! - **直接修改**：直接修改实体的 Position 和 Velocity 组件
//!
//! # 执行顺序
//!
//! 运动域无依赖，最先执行。其输出是碰撞域的输入。

use duan::{domain_rules_any, DomainContext, DomainRules, Entity, EntityId};

use crate::components::{Position, Velocity};

/// 运动域规则
///
/// 执行半隐式欧拉积分：
/// - v_new = v_old - g * dt
/// - p_new = p_old + v_new * dt
pub struct MotionRules {
    gravity: f64,
}

impl MotionRules {
    pub fn new(gravity: f64) -> Self {
        Self { gravity }
    }

    pub fn earth() -> Self {
        Self::new(9.8)
    }
}

impl DomainRules for MotionRules {
    fn compute(&mut self, ctx: &mut DomainContext, dt: f64) {
        // 收集实体 ID（释放不可变引用后再可变访问实体）
        let entity_ids: Vec<EntityId> = ctx.own_entity_ids().collect();

        for entity_id in entity_ids {
            // 读取当前位置和速度（只读借用）
            let (x, y, z, vx_old, vy_old, vz_old) = {
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
                (pos.x, pos.y, pos.z, vel.vx, vel.vy, vel.vz)
            };

            // 半隐式欧拉积分
            let new_vy = vy_old - self.gravity * dt;
            let new_x = x + vx_old * dt;
            let new_y = y + new_vy * dt;
            let new_z = z + vz_old * dt;

            // 可变访问：写回结果
            if let Some(entity) = ctx.entities.get_mut(entity_id) {
                if let Some(pos) = entity.get_component_mut::<Position>() {
                    pos.x = new_x;
                    pos.y = new_y;
                    pos.z = new_z;
                }
                if let Some(vel) = entity.get_component_mut::<Velocity>() {
                    vel.vx = vx_old;
                    vel.vy = new_vy;
                    vel.vz = vz_old;
                }
            }
        }
    }

    fn try_attach(&mut self, entity: &Entity) -> bool {
        entity.has_component::<Position>() && entity.has_component::<Velocity>()
    }

    fn on_detach(&mut self, _entity_id: EntityId) {}

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    domain_rules_any!(MotionRules);
}
