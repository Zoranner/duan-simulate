use duan::{Entity, EntityContext};

use crate::components::{Faction, Health, Helm, Position, Radar, Velocity};

/// 舰船实体
///
/// `tick()` 是舰船 AI 的主体：每帧从快照扫描敌方位置，
/// 将期望航向写入**意图**组件 `Helm`（`Intent`），由 `MotionDomain` 执行转向。
///
/// **认知 / 意图 / 状态**闭环（本实体侧重意图 → 状态）：
///
/// | 阶段 | 操作                                     |
/// |------|------------------------------------------|
/// | 感知 | 读取快照中的意图与状态（位置、阵营、雷达等） |
/// | 决策 | `tick` 内部逻辑：寻找最近存活敌方         |
/// | 意志 | 写入**意图** `Helm`（期望航向）            |
/// | 执行 | MotionDomain 读意图，写**状态**（速度等）  |
pub struct Ship;

impl Entity for Ship {
    fn tick(ctx: &mut EntityContext) {
        let my_id = ctx.id();
        let snap = ctx.snapshot();

        // 感知：读取自身位置、阵营、雷达范围（从上帧快照）
        let Some(my_pos) = snap.get::<Position>(my_id) else {
            return;
        };
        let Some(my_faction) = snap.get::<Faction>(my_id) else {
            return;
        };
        let Some(my_radar) = snap.get::<Radar>(my_id) else {
            return;
        };
        let (my_x, my_y) = (my_pos.x, my_pos.y);
        let my_team = my_faction.team;
        let radar_range = my_radar.range;

        // 决策：在雷达范围内寻找最近的存活敌方
        let mut best_dir: Option<(f64, f64)> = None;
        let mut best_dist = f64::MAX;

        for (id, pos) in snap.iter::<Position>() {
            if id == my_id {
                continue;
            }
            // 只追踪有阵营的实体（舰船），跳过导弹
            let Some(faction) = snap.get::<Faction>(id) else {
                continue;
            };
            if faction.team == my_team {
                continue;
            }
            // 跳过已死亡的目标
            if snap.get::<Health>(id).map(|h| h.is_dead()).unwrap_or(true) {
                continue;
            }
            let dx = pos.x - my_x;
            let dy = pos.y - my_y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < radar_range && dist < best_dist {
                best_dist = dist;
                best_dir = Some((dx, dy));
            }
        }

        // 意志：将期望航向写入意图（Helm / Intent），MotionDomain 将据此转向
        //
        // 无目标时保持当前速度方向，避免舰船因 Helm.heading 初始值为 0 而偏转至正右方。
        let desired_heading = if let Some((dx, dy)) = best_dir {
            dy.atan2(dx)
        } else {
            snap.get::<Velocity>(my_id)
                .map(|v| v.vy.atan2(v.vx))
                .unwrap_or_else(|| ctx.get::<Helm>().map(|h| h.heading).unwrap_or(0.0))
        };
        let turn_rate = ctx
            .get::<Helm>()
            .map(|h| h.turn_rate)
            .unwrap_or(std::f64::consts::FRAC_PI_4);
        ctx.set(Helm {
            heading: desired_heading,
            turn_rate,
        });
    }
}
