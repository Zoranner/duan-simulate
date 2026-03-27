//! 文本显示模块
//!
//! 简单的 stdout 输出：每 10 帧打印态势，事件发生时即时打印日志行。

use duan::EntityId;

/// 舰船态势快照
#[derive(Clone)]
pub struct ShipStatus {
    pub name: String,
    pub team: u8,
    pub x: f64,
    pub y: f64,
    pub health: f64,
    pub max_health: f64,
}

/// 一帧内收集的战斗日志条目
pub enum LogEntry {
    #[allow(dead_code)]
    Detection {
        observer: String,
        target: String,
        distance: f64,
    },
    Fire {
        shooter: String,
        target: String,
    },
    Hit {
        target: String,
        damage: f64,
        health_after: f64,
    },
    ShipDestroyed {
        name: String,
    },
}

/// 战斗日志收集器
pub struct CombatLog {
    entries: Vec<(f64, LogEntry)>,
    /// EntityId -> 名称，用于事件处理时查找名称
    names: std::collections::HashMap<EntityId, String>,
}

impl CombatLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            names: std::collections::HashMap::new(),
        }
    }

    pub fn register_name(&mut self, id: EntityId, name: impl Into<String>) {
        self.names.insert(id, name.into());
    }

    pub fn get_name(&self, id: EntityId) -> String {
        self.names
            .get(&id)
            .cloned()
            .unwrap_or_else(|| format!("#{}", id.raw()))
    }

    pub fn log(&mut self, sim_time: f64, entry: LogEntry) {
        self.entries.push((sim_time, entry));
    }

    /// 打印并清空收集的日志
    pub fn flush(&mut self) {
        for (t, entry) in self.entries.drain(..) {
            match entry {
                LogEntry::Detection {
                    observer,
                    target,
                    distance,
                } => {
                    println!("[{t:6.1}s] 探测  {observer} 发现 {target}（距离 {distance:.0}m）");
                }
                LogEntry::Fire { shooter, target } => {
                    println!("[{t:6.1}s] 开火  {shooter} → {target}");
                }
                LogEntry::Hit {
                    target,
                    damage,
                    health_after,
                } => {
                    println!("[{t:6.1}s] 命中  {target} 受到 {damage:.0} 点伤害，剩余 HP {health_after:.0}");
                }
                LogEntry::ShipDestroyed { name } => {
                    println!("[{t:6.1}s] 销毁  *** {name} 被击沉 ***");
                }
            }
        }
    }
}

/// 打印当前态势表
pub fn print_status(sim_time: f64, ships: &[ShipStatus]) {
    println!();
    println!("── T={sim_time:.1}s 态势 ──────────────────────────");
    for s in ships {
        let team_str = if s.team == 0 { "红" } else { "蓝" };
        let hp_bar = hp_bar(s.health, s.max_health, 10);
        println!(
            "  [{team_str}] {:<8} ({:5.0},{:5.0})  HP {hp_bar} {:.0}/{:.0}",
            s.name, s.x, s.y, s.health, s.max_health
        );
    }
}

fn hp_bar(current: f64, max: f64, width: usize) -> String {
    let filled = ((current / max).clamp(0.0, 1.0) * width as f64).round() as usize;
    let empty = width - filled;
    format!("[{}{}]", "#".repeat(filled), ".".repeat(empty))
}

/// 打印仿真结果摘要
pub fn print_summary(winner_team: Option<u8>, sim_time: f64, total_missiles: u32, total_hits: u32) {
    println!();
    println!("══════════════════════════════════════");
    match winner_team {
        Some(0) => println!("  胜利方：红方"),
        Some(1) => println!("  胜利方：蓝方"),
        Some(_) => println!("  胜利方：未知"),
        None => println!("  结果：平局或超时"),
    }
    println!("  仿真时长：{sim_time:.1}s");
    println!("  总发射导弹：{total_missiles}");
    println!("  总命中次数：{total_hits}");
    println!("══════════════════════════════════════");
}
