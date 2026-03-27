//! 空间域
//!
//! 纯服务型域，提供距离计算和范围查询。compute 为空操作。
//! 其他域通过 ctx.get_domain::<SpaceRules>() 调用服务接口。

use duan::{domain_rules_any, DomainContext, DomainRules, Entity, EntityId, EntityStore};

use crate::components::Position;

pub struct SpaceRules;

impl SpaceRules {
    pub fn new() -> Self {
        Self
    }

    /// 计算两个实体之间的距离
    pub fn distance(&self, id_a: EntityId, id_b: EntityId, entities: &EntityStore) -> Option<f64> {
        let pos_a = entities.get(id_a)?.get_component::<Position>()?;
        let pos_b = entities.get(id_b)?.get_component::<Position>()?;
        let dx = pos_b.x - pos_a.x;
        let dy = pos_b.y - pos_a.y;
        Some((dx * dx + dy * dy).sqrt())
    }

    /// 返回以 center_id 为中心、radius 范围内的所有活跃实体（含自身）
    pub fn entities_in_range(
        &self,
        center_id: EntityId,
        radius: f64,
        entities: &EntityStore,
    ) -> Vec<EntityId> {
        let center_pos = match entities.get(center_id).and_then(|e| e.get_component::<Position>())
        {
            Some(p) => (p.x, p.y),
            None => return vec![],
        };

        entities
            .active_entities()
            .filter(|e| {
                if let Some(pos) = e.get_component::<Position>() {
                    let dx = pos.x - center_pos.0;
                    let dy = pos.y - center_pos.1;
                    (dx * dx + dy * dy).sqrt() <= radius
                } else {
                    false
                }
            })
            .map(|e| e.id)
            .collect()
    }
}

impl Default for SpaceRules {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainRules for SpaceRules {
    fn compute(&mut self, _ctx: &mut DomainContext) {
        // 纯服务型域，compute 为空操作
    }

    fn try_attach(&self, entity: &Entity) -> bool {
        entity.has_component::<Position>()
    }

    fn on_detach(&mut self, _entity_id: EntityId) {}

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    domain_rules_any!(SpaceRules);
}
