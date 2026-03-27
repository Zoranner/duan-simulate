//! 阵营域
//!
//! 纯服务型域，提供 is_hostile 查询。compute 为空操作。
//! 阵营关系在构造时配置，仿真过程中不变。

use duan::{domain_rules_any, DomainContext, DomainRules, Entity, EntityId};
use std::collections::HashMap;

use crate::components::Faction;

pub struct FactionRules {
    /// 敌对关系表：(team_a, team_b) -> is_hostile
    hostile: HashMap<(u8, u8), bool>,
}

impl FactionRules {
    /// 创建红蓝双方互为敌对的配置
    pub fn red_vs_blue() -> Self {
        let mut hostile = HashMap::new();
        hostile.insert((0, 1), true);
        hostile.insert((1, 0), true);
        hostile.insert((0, 0), false);
        hostile.insert((1, 1), false);
        Self { hostile }
    }

    /// 查询两个阵营是否敌对
    pub fn is_hostile_teams(&self, team_a: u8, team_b: u8) -> bool {
        *self.hostile.get(&(team_a, team_b)).unwrap_or(&false)
    }

    /// 查询两个实体是否敌对（通过实体存储查询阵营组件）
    pub fn is_hostile(&self, id_a: EntityId, id_b: EntityId, entities: &duan::EntityStore) -> bool {
        let team_a = entities
            .get(id_a)
            .and_then(|e| e.get_component::<Faction>())
            .map(|f| f.team);
        let team_b = entities
            .get(id_b)
            .and_then(|e| e.get_component::<Faction>())
            .map(|f| f.team);

        match (team_a, team_b) {
            (Some(a), Some(b)) => self.is_hostile_teams(a, b),
            _ => false,
        }
    }
}

impl DomainRules for FactionRules {
    fn compute(&mut self, _ctx: &mut DomainContext) {
        // 纯服务型域，compute 为空操作
    }

    fn try_attach(&self, entity: &Entity) -> bool {
        entity.has_component::<Faction>()
    }

    fn on_detach(&mut self, _entity_id: EntityId) {}

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    domain_rules_any!(FactionRules);
}
