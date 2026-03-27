//! 指挥域
//!
//! 职责一（数据链）：汇总同阵营所有舰船的探测结果，构建阵营级目标池。
//!   一舰看到的敌人，全舰队共享，解决各舰"单打独斗"的问题。
//!
//! 职责二（目标分配）：统计各敌舰已被几艘友舰瞄准，均衡分配攻击任务，
//!   避免多舰集火同一目标而忽视其他威胁。
//!
//! 执行顺序：faction → space → motion → detection → **command** → steering → combat → collision

use duan::{domain_rules_any, DomainContext, DomainRules, Entity, EntityId};
use std::collections::{HashMap, HashSet};

use crate::components::{Faction, Weapon};
use crate::domains::DetectionRules;

pub struct CommandRules {
    /// 阵营级目标池（数据链）：team → 该阵营已知的所有敌舰 ID
    fleet_detected: HashMap<u8, HashSet<EntityId>>,
    /// 目标分配表：ship_id → assigned target_id
    assignments: HashMap<EntityId, EntityId>,
}

impl CommandRules {
    pub fn new() -> Self {
        Self {
            fleet_detected: HashMap::new(),
            assignments: HashMap::new(),
        }
    }

    /// 查询阵营级目标池（数据链）
    pub fn get_fleet_detected(&self, team: u8) -> &HashSet<EntityId> {
        static EMPTY: std::sync::OnceLock<HashSet<EntityId>> = std::sync::OnceLock::new();
        self.fleet_detected
            .get(&team)
            .unwrap_or_else(|| EMPTY.get_or_init(HashSet::new))
    }

    /// 查询本舰当前指派目标
    pub fn get_assignment(&self, ship_id: EntityId) -> Option<EntityId> {
        self.assignments.get(&ship_id).copied()
    }
}

impl Default for CommandRules {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainRules for CommandRules {
    fn compute(&mut self, ctx: &mut DomainContext) {
        let ship_ids: Vec<EntityId> = ctx.own_entity_ids().collect();

        // ── 阶段一：数据链聚合 ───────────────────────────────
        self.fleet_detected.clear();

        let ship_teams: Vec<(EntityId, u8)> = ship_ids
            .iter()
            .filter_map(|&id| {
                let team = ctx.entities.get(id)?.get_component::<Faction>()?.team;
                Some((id, team))
            })
            .collect();

        for (ship_id, team) in &ship_teams {
            let detected: Vec<EntityId> = match ctx.get_domain::<DetectionRules>() {
                Some(d) => d.get_detected(*ship_id).iter().copied().collect(),
                None => continue,
            };
            self.fleet_detected
                .entry(*team)
                .or_default()
                .extend(detected);
        }

        // 移除已销毁的实体（防止上帧残留）
        for pool in self.fleet_detected.values_mut() {
            pool.retain(|&id| ctx.entities.get(id).is_some());
        }

        // ── 阶段二：目标分配 ─────────────────────────────────
        // 移除目标已销毁的旧分配
        self.assignments
            .retain(|_, &mut target| ctx.entities.get(target).is_some());

        // 按阵营分组
        let mut teams: HashMap<u8, Vec<EntityId>> = HashMap::new();
        for (ship_id, team) in &ship_teams {
            teams.entry(*team).or_default().push(*ship_id);
        }

        for (team, ships) in &teams {
            let enemies: Vec<EntityId> = self
                .fleet_detected
                .get(team)
                .map(|s| {
                    let mut v: Vec<_> = s.iter().copied().collect();
                    v.sort(); // 确定性排序，避免随机波动
                    v
                })
                .unwrap_or_default();

            if enemies.is_empty() {
                for ship in ships {
                    self.assignments.remove(ship);
                }
                continue;
            }

            // 统计各敌舰当前被分配的友舰数
            let mut coverage: HashMap<EntityId, usize> =
                enemies.iter().map(|&e| (e, 0)).collect();
            for ship in ships.iter() {
                if let Some(&target) = self.assignments.get(ship) {
                    if let Some(cnt) = coverage.get_mut(&target) {
                        *cnt += 1;
                    }
                }
            }

            // 重新分配：保留仍有效的分配，对失效的分配选覆盖最少的目标
            for ship in ships.iter() {
                let still_valid = self
                    .assignments
                    .get(ship)
                    .map(|t| enemies.contains(t))
                    .unwrap_or(false);

                if still_valid {
                    continue;
                }

                // 选当前被攻击数最少的敌舰
                let target = enemies
                    .iter()
                    .min_by_key(|&&e| coverage.get(&e).copied().unwrap_or(0))
                    .copied();

                if let Some(t) = target {
                    *coverage.entry(t).or_insert(0) += 1;
                    self.assignments.insert(*ship, t);
                }
            }
        }
    }

    fn try_attach(&self, entity: &Entity) -> bool {
        entity.has_component::<Faction>() && entity.has_component::<Weapon>()
    }

    fn on_detach(&mut self, entity_id: EntityId) {
        self.assignments.remove(&entity_id);
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["detection", "faction"]
    }

    domain_rules_any!(CommandRules);
}
