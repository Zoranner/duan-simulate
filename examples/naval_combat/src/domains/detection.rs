//! 探测域
//!
//! 遍历有雷达的实体，通过空间域范围查询候选目标，通过阵营域判断敌对关系。
//! 更新内部探测状态，发出 DetectionEvent。
//!
//! 体现了"跨域服务调用"：通过 ctx.get_domain::<T>() 调用 space 和 faction 的服务。

use duan::{domain_rules_any, DomainContext, DomainEvent, DomainRules, Entity, EntityId};
use std::collections::{HashMap, HashSet};

use crate::components::Radar;
use crate::domains::{FactionRules, SpaceRules};
use crate::events::DetectionEvent;

pub struct DetectionRules {
    /// 当前各观察者的探测目标集合
    detected: HashMap<EntityId, HashSet<EntityId>>,
}

impl DetectionRules {
    pub fn new() -> Self {
        Self {
            detected: HashMap::new(),
        }
    }

    /// 获取观察者的当前探测目标集合
    pub fn get_detected(&self, observer_id: EntityId) -> &HashSet<EntityId> {
        static EMPTY: std::sync::OnceLock<HashSet<EntityId>> = std::sync::OnceLock::new();
        self.detected
            .get(&observer_id)
            .unwrap_or_else(|| EMPTY.get_or_init(HashSet::new))
    }
}

impl Default for DetectionRules {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainRules for DetectionRules {
    fn compute(&mut self, ctx: &mut DomainContext) {
        let observer_ids: Vec<EntityId> = ctx.own_entity_ids().collect();

        // 每帧先清空旧的探测结果，防止已销毁目标残留
        self.detected.clear();

        for observer_id in observer_ids {
            let radar_range = match ctx
                .entities
                .get(observer_id)
                .and_then(|e| e.get_component::<Radar>())
                .map(|r| r.range)
            {
                Some(r) => r,
                None => continue,
            };

            // 通过空间域服务获取范围内候选目标（方式一：按类型查找，推荐）
            let candidates = match ctx.get_domain::<SpaceRules>() {
                Some(space) => space.entities_in_range(observer_id, radar_range, ctx.entities),
                None => continue,
            };

            let mut targets = HashSet::new();

            for target_id in candidates {
                if target_id == observer_id {
                    continue;
                }

                // 通过阵营域判断是否敌对
                let is_hostile = match ctx.get_domain::<FactionRules>() {
                    Some(faction) => faction.is_hostile(observer_id, target_id, ctx.entities),
                    None => false,
                };

                if !is_hostile {
                    continue;
                }

                let distance = ctx
                    .get_domain::<SpaceRules>()
                    .and_then(|s| s.distance(observer_id, target_id, ctx.entities))
                    .unwrap_or(f64::MAX);

                targets.insert(target_id);

                ctx.emit(DomainEvent::custom(DetectionEvent::new(
                    observer_id,
                    target_id,
                    distance,
                )));
            }

            if !targets.is_empty() {
                self.detected.insert(observer_id, targets);
            }
        }
    }

    fn try_attach(&self, entity: &Entity) -> bool {
        entity.has_component::<Radar>()
    }

    fn on_detach(&mut self, entity_id: EntityId) {
        self.detected.remove(&entity_id);
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["space", "faction"]
    }

    domain_rules_any!(DetectionRules);
}
