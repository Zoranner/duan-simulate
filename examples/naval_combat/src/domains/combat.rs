//! 战斗域
//!
//! 遍历有武器的舰船，查询探测域的探测结果，对射程内的目标发出 FireEvent。
//! 检查 Health 为零时发出 ShipDestroyedEvent。

use duan::{domain_rules_any, DomainContext, DomainEvent, DomainRules, Entity, EntityId};
use std::collections::HashMap;

use crate::components::{Health, Weapon};
use crate::domains::{DetectionRules, SpaceRules};
use crate::events::{FireEvent, ShipDestroyedEvent};

pub struct CombatRules {
    /// 各实体的开火冷却剩余时间（秒）
    fire_cooldowns: HashMap<EntityId, f64>,
}

impl CombatRules {
    pub fn new() -> Self {
        Self {
            fire_cooldowns: HashMap::new(),
        }
    }
}

impl Default for CombatRules {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainRules for CombatRules {
    fn compute(&mut self, ctx: &mut DomainContext) {
        let dt = ctx.dt;
        let entity_ids: Vec<EntityId> = ctx.own_entity_ids().collect();

        // 更新冷却计时
        for cd in self.fire_cooldowns.values_mut() {
            *cd = (*cd - dt).max(0.0);
        }

        for entity_id in entity_ids {
            // 检查 Health 是否归零（仅处理尚未触发销毁的实体）
            let is_dead = ctx
                .entities
                .get(entity_id)
                .and_then(|e| e.get_component::<Health>())
                .map(|h| h.is_dead())
                .unwrap_or(false);

            if is_dead && !ctx.entities.is_destroying(entity_id) {
                // 暂时没有 killer 信息，使用 entity_id 自身占位（HitEvent 已记录 killer）
                ctx.emit(DomainEvent::custom(ShipDestroyedEvent {
                    ship_id: entity_id,
                    killer_id: entity_id,
                }));
                continue;
            }

            if is_dead {
                continue;
            }

            // 读取武器参数
            let (weapon_range, weapon_damage, weapon_cooldown, missile_speed) = {
                let entity = match ctx.entities.get(entity_id) {
                    Some(e) => e,
                    None => continue,
                };
                match entity.get_component::<Weapon>() {
                    Some(w) => (w.range, w.damage, w.fire_cooldown, w.missile_speed),
                    None => continue,
                }
            };

            // 检查冷却
            let cooldown = self.fire_cooldowns.entry(entity_id).or_insert(0.0);
            if *cooldown > 0.0 {
                continue;
            }

            // 查询探测域的探测目标（方式一：按类型查找）
            let detected_targets: Vec<EntityId> = match ctx.get_domain::<DetectionRules>() {
                Some(detection) => detection.get_detected(entity_id).iter().copied().collect(),
                None => continue,
            };

            // 对第一个在射程内的目标开火
            for target_id in detected_targets {
                let distance = match ctx.get_domain::<SpaceRules>() {
                    Some(space) => space.distance(entity_id, target_id, ctx.entities),
                    None => None,
                };

                let distance = match distance {
                    Some(d) => d,
                    None => continue,
                };

                if distance > weapon_range {
                    continue;
                }

                // 计算朝目标的方向向量
                let (dir_x, dir_y) = {
                    let shooter_pos = ctx
                        .entities
                        .get(entity_id)
                        .and_then(|e| e.get_component::<crate::components::Position>())
                        .map(|p| (p.x, p.y));
                    let target_pos = ctx
                        .entities
                        .get(target_id)
                        .and_then(|e| e.get_component::<crate::components::Position>())
                        .map(|p| (p.x, p.y));

                    match (shooter_pos, target_pos) {
                        (Some((sx, sy)), Some((tx, ty))) => (tx - sx, ty - sy),
                        _ => continue,
                    }
                };

                let (launch_x, launch_y) = ctx
                    .entities
                    .get(entity_id)
                    .and_then(|e| e.get_component::<crate::components::Position>())
                    .map(|p| (p.x, p.y))
                    .unwrap_or((0.0, 0.0));

                ctx.emit(DomainEvent::custom(FireEvent {
                    shooter_id: entity_id,
                    target_id,
                    launch_x,
                    launch_y,
                    dir_x,
                    dir_y,
                    missile_speed,
                    damage: weapon_damage,
                }));

                // 重置冷却，每帧只对一个目标开火
                *self.fire_cooldowns.entry(entity_id).or_insert(0.0) = weapon_cooldown;
                break;
            }
        }
    }

    fn try_attach(&self, entity: &Entity) -> bool {
        entity.has_component::<Weapon>() && entity.has_component::<Health>()
    }

    fn on_detach(&mut self, entity_id: EntityId) {
        self.fire_cooldowns.remove(&entity_id);
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["detection", "space"]
    }

    domain_rules_any!(CombatRules);
}
