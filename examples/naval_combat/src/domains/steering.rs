//! 舵控域
//!
//! 依赖探测域获取当前帧的探测结果，将舰船速度方向逐步转向最近敌舰。
//! 转向受 `Helm` 组件的 `turn_rate`（弧度/秒）限制，保持速率不变。
//!
//! 执行顺序位于 motion 之后、combat 之前；速度修改在下一仿真步的 motion 中生效。

use duan::{domain_rules_any, DomainContext, DomainRules, Entity, EntityId};

use crate::components::{Helm, Position, Velocity, Weapon};
use crate::domains::{CommandRules, DetectionRules};

pub struct SteeringRules;

impl SteeringRules {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SteeringRules {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainRules for SteeringRules {
    fn compute(&mut self, ctx: &mut DomainContext) {
        let dt = ctx.dt;
        let entity_ids: Vec<EntityId> = ctx.own_entity_ids().collect();

        for entity_id in entity_ids {
            // 只读阶段：提取自身位置、速度、转向速率和武器射程
            let (px, py, vx, vy, turn_rate, weapon_range) = {
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
                let turn_rate = match entity.get_component::<Helm>() {
                    Some(h) => h.turn_rate,
                    None => continue,
                };
                let weapon_range = entity
                    .get_component::<Weapon>()
                    .map(|w| w.range)
                    .unwrap_or(200.0);
                (pos.0, pos.1, vel.0, vel.1, turn_rate, weapon_range)
            };

            let speed = (vx * vx + vy * vy).sqrt();
            if speed < 1e-6 {
                continue;
            }

            // 从探测域获取本帧已探测到的敌舰 ID（已清零旧帧残留）
            let detected: Vec<EntityId> = match ctx.get_domain::<DetectionRules>() {
                Some(d) => d.get_detected(entity_id).iter().copied().collect(),
                None => continue,
            };

            // 优先使用指挥域的指派目标，其次找最近的探测目标
            let assigned_pos = ctx
                .get_domain::<CommandRules>()
                .and_then(|c| c.get_assignment(entity_id))
                .and_then(|tid| ctx.entities.get(tid))
                .and_then(|e| e.get_component::<Position>())
                .map(|p| (p.x, p.y));

            let nearest_detected = detected
                .iter()
                .filter_map(|&tid| {
                    ctx.entities
                        .get(tid)
                        .and_then(|e| e.get_component::<Position>())
                        .map(|p| {
                            let dx = p.x - px;
                            let dy = p.y - py;
                            (p.x, p.y, dx * dx + dy * dy)
                        })
                })
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(x, y, _)| (x, y));

            // 无探测目标时朝战场中央推进，避免舰船漂向边缘
            let (tx, ty) = match assigned_pos.or(nearest_detected) {
                Some((x, y)) => (x, y),
                None => {
                    let team = ctx
                        .entities
                        .get(entity_id)
                        .and_then(|e| e.get_component::<crate::components::Faction>())
                        .map(|f| f.team)
                        .unwrap_or(0);
                    let center_x = 500.0_f64;
                    let target_y = if team == 0 { py + 200.0 } else { py - 200.0 };
                    (center_x, target_y)
                }
            };

            // 根据与最近敌舰的距离选择战术
            //   > engage_range : 接敌（向敌逼近）
            //   < retreat_range: 后撤（远离敌舰）
            //   中间区间       : 侧移绕行，保持交战距离
            let dx = tx - px;
            let dy = ty - py;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 1.0 {
                continue;
            }

            // 接敌距离：当超出武器射程时主动靠近
            let engage_range = weapon_range * 1.1;
            // 后撤距离：太近时拉开距离
            let retreat_range = weapon_range * 0.45;

            let (desired_vx, desired_vy) = if dist > engage_range {
                // 超出射程：直接朝敌方接近
                (dx / dist, dy / dist)
            } else if dist < retreat_range {
                // 距离过近：反向拉开
                (-dx / dist, -dy / dist)
            } else {
                // 处于交战区间：侧移绕行
                // 用 entity_id 奇偶决定顺/逆时针，不同舰形成包夹
                let side = if entity_id.0 % 2 == 0 { 1.0_f64 } else { -1.0_f64 };
                // 将朝向敌方的单位向量旋转 90°（侧移）+ 轻微内倾（保持射程）
                let strafe_vx = -dy / dist * side;
                let strafe_vy = dx / dist * side;
                // 小幅向敌方偏移，防止绕行时脱离射程
                let bias = 0.15;
                let bx = strafe_vx + dx / dist * bias;
                let by = strafe_vy + dy / dist * bias;
                let blen = (bx * bx + by * by).sqrt();
                (bx / blen, by / blen)
            };

            // 当前方向单位向量
            let curr_vx = vx / speed;
            let curr_vy = vy / speed;

            // 用叉积符号 + 点积反余弦计算偏转角（带方向）
            let cross = curr_vx * desired_vy - curr_vy * desired_vx;
            let dot = (curr_vx * desired_vx + curr_vy * desired_vy).clamp(-1.0, 1.0);
            let angle_to_target = cross.signum() * dot.acos();

            // 本步最多偏转 turn_rate * dt 弧度
            let rotate = angle_to_target.clamp(-turn_rate * dt, turn_rate * dt);

            let cos_a = rotate.cos();
            let sin_a = rotate.sin();
            let new_vx = (curr_vx * cos_a - curr_vy * sin_a) * speed;
            let new_vy = (curr_vx * sin_a + curr_vy * cos_a) * speed;

            // 写回新速度
            if let Some(entity) = ctx.entities.get_mut(entity_id) {
                if let Some(vel) = entity.get_component_mut::<Velocity>() {
                    vel.vx = new_vx;
                    vel.vy = new_vy;
                }
            }
        }
    }

    fn try_attach(&self, entity: &Entity) -> bool {
        entity.has_component::<Helm>()
    }

    fn on_detach(&mut self, _entity_id: EntityId) {}

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["command", "detection"]
    }

    domain_rules_any!(SteeringRules);
}
