//! 碰撞域
//!
//! 仅管辖导弹实体（try_attach 要求 MissileBody）。
//! 每帧检查导弹与所有活跃实体的距离，命中时发出 HitEvent。
//! 通过 ctx.entities.active_entities() 遍历全量实体（ISSUE-009 确认合法）。

use duan::{domain_rules_any, DomainContext, DomainEvent, DomainRules, Entity, EntityId};
use std::collections::HashSet;

use crate::components::{Faction, MissileBody, Position, Seeker};
use crate::events::{HitEvent, MissileExpiredEvent};

/// 命中半径（米）
const HIT_RADIUS: f64 = 8.0;

pub struct CollisionRules {
    /// 已处理命中的导弹（同帧防重复）
    hit_missiles: HashSet<EntityId>,
}

impl CollisionRules {
    pub fn new() -> Self {
        Self {
            hit_missiles: HashSet::new(),
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
        self.hit_missiles.clear();

        let missile_ids: Vec<EntityId> = ctx.own_entity_ids().collect();

        for missile_id in missile_ids {
            if self.hit_missiles.contains(&missile_id) {
                continue;
            }

            // 读取导弹位置和追踪信息
            let (missile_x, missile_y, seeker) = {
                let entity = match ctx.entities.get(missile_id) {
                    Some(e) => e,
                    None => continue,
                };
                let pos = match entity.get_component::<Position>() {
                    Some(p) => (p.x, p.y),
                    None => continue,
                };
                let seeker = entity.get_component::<Seeker>().copied();
                (pos.0, pos.1, seeker)
            };

            // 超射程检查：导弹飞出最大射程后自毁
            if let Some(s) = seeker {
                if s.max_range > 0.0 && s.traveled >= s.max_range {
                    self.hit_missiles.insert(missile_id);
                    ctx.emit(DomainEvent::custom(MissileExpiredEvent { missile_id }));
                    continue;
                }
            }

            // 读取导弹阵营（通过发射者）
            let missile_team = seeker.and_then(|s| {
                ctx.entities
                    .get(s.shooter_id)
                    .and_then(|e| e.get_component::<Faction>())
                    .map(|f| f.team)
            });

            // 遍历全量活跃实体，查找可命中目标（全量只读遍历，合法）
            let targets: Vec<(EntityId, f64)> = ctx
                .entities
                .active_entities()
                .filter(|e| {
                    // 排除导弹自身
                    if e.id == missile_id {
                        return false;
                    }
                    // 只命中舰船（无 MissileBody 的有 Faction 的实体）
                    if e.has_component::<MissileBody>() {
                        return false;
                    }
                    if !e.has_component::<Faction>() {
                        return false;
                    }
                    // 不命中同阵营目标
                    if let (Some(mt), Some(et)) =
                        (missile_team, e.get_component::<Faction>().map(|f| f.team))
                    {
                        if mt == et {
                            return false;
                        }
                    }
                    true
                })
                .filter_map(|e| {
                    let pos = e.get_component::<Position>()?;
                    let dx = pos.x - missile_x;
                    let dy = pos.y - missile_y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist <= HIT_RADIUS {
                        Some((e.id, dist))
                    } else {
                        None
                    }
                })
                .collect();

            if let Some((target_id, _)) = targets.into_iter().next() {
                let (shooter_id, damage) = seeker
                    .map(|s| (s.shooter_id, s.damage))
                    .unwrap_or((missile_id, 0.0));

                self.hit_missiles.insert(missile_id);
                ctx.emit(DomainEvent::custom(HitEvent {
                    missile_id,
                    target_id,
                    shooter_id,
                    damage,
                }));
            }
        }
    }

    fn try_attach(&self, entity: &Entity) -> bool {
        entity.has_component::<MissileBody>()
    }

    fn on_detach(&mut self, entity_id: EntityId) {
        self.hit_missiles.remove(&entity_id);
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["motion"]
    }

    domain_rules_any!(CollisionRules);
}
