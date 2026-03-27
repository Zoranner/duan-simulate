//! 终端可视化显示模块
//!
//! 使用 crossterm 在终端备用屏幕中回放仿真帧序列。
//!
//! # 设计说明
//!
//! 采用与 free_fall 相同的两阶段设计：无状态渲染器 `NavalDisplay` 仅负责
//! 将 `RenderFrame` 绘制到屏幕，不持有仿真状态。Phase 1（仿真）全速推进并
//! 缓存帧序列，Phase 2（回放）按 sim_time 时间戳以真实时钟定时渲染。

use crossterm::{
    cursor, execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
use duan::EntityId;
use std::collections::{HashMap, VecDeque};
use std::io::{self, Write};

// ── 地图视口常量 ─────────────────────────────────────────

/// 战场 x 轴视口范围（米）
const X_MIN: f64 = -50.0;
const X_MAX: f64 = 1050.0;
/// 战场 y 轴视口范围（米，y 大 = 屏幕上方）
const Y_MIN: f64 = -50.0;
const Y_MAX: f64 = 1050.0;
/// 地图内宽（字符数）
const MAP_W: usize = 52;
/// 地图内高（行数）
const MAP_H: usize = 26;
/// 血条宽度（字符数）
const HP_BAR_W: usize = 10;
/// 滚动事件日志显示行数
const LOG_LINES: usize = 8;

fn to_map_cell(x: f64, y: f64) -> (usize, usize) {
    let col = ((x - X_MIN) / (X_MAX - X_MIN) * MAP_W as f64).round() as isize;
    let row = ((Y_MAX - y) / (Y_MAX - Y_MIN) * MAP_H as f64).round() as isize;
    (
        col.clamp(0, MAP_W as isize - 1) as usize,
        row.clamp(0, MAP_H as isize - 1) as usize,
    )
}

// ── 渲染帧数据结构 ───────────────────────────────────────

/// 单舰船的渲染快照
pub struct ShipFrame {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub health: f64,
    pub max_health: f64,
    pub team: u8,
    pub alive: bool,
}

/// 单导弹的渲染快照
pub struct MissileDot {
    pub x: f64,
    pub y: f64,
    pub team: u8,
}

/// 单帧渲染数据快照，由仿真阶段填充，回放阶段消费
pub struct RenderFrame {
    pub sim_time: f64,
    pub ships: Vec<ShipFrame>,
    pub missiles: Vec<MissileDot>,
    pub recent_log: Vec<String>,
    pub active_missile_count: usize,
    pub total_missiles: u32,
    pub total_hits: u32,
}

// ── 战斗日志收集器 ───────────────────────────────────────

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

pub struct CombatLog {
    pending: Vec<(f64, LogEntry)>,
    recent: VecDeque<String>,
    names: HashMap<EntityId, String>,
}

impl CombatLog {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            recent: VecDeque::new(),
            names: HashMap::new(),
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
        self.pending.push((sim_time, entry));
    }

    /// 将本帧待处理条目格式化后推入滚动日志窗口
    pub fn drain_to_recent(&mut self) {
        for (t, entry) in self.pending.drain(..) {
            let s = match entry {
                LogEntry::Detection {
                    observer,
                    target,
                    distance,
                } => {
                    format!("[{t:6.1}s] 探测  {observer} 发现 {target}（{distance:.0}m）")
                }
                LogEntry::Fire { shooter, target } => {
                    format!("[{t:6.1}s] 开火  {shooter} → {target}")
                }
                LogEntry::Hit {
                    target,
                    damage,
                    health_after,
                } => {
                    format!("[{t:6.1}s] 命中  {target}  -{damage:.0}HP  剩余 {health_after:.0}")
                }
                LogEntry::ShipDestroyed { name } => {
                    format!("[{t:6.1}s] 击沉  *** {name} 被击沉 ***")
                }
            };
            if self.recent.len() >= LOG_LINES {
                self.recent.pop_front();
            }
            self.recent.push_back(s);
        }
    }

    pub fn recent_log(&self) -> Vec<String> {
        self.recent.iter().cloned().collect()
    }
}

// ── 终端显示器 ───────────────────────────────────────────

/// 海战仿真终端显示器（无状态渲染器）
///
/// 创建时自动进入备用屏幕，Drop 时自动恢复原始终端状态。
pub struct NavalDisplay;

impl NavalDisplay {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(Self)
    }

    pub fn render(&self, frame: &RenderFrame) -> io::Result<()> {
        let mut out = io::stdout();
        queue!(out, cursor::MoveTo(0, 0))?;

        // ── 标题行 ───────────────────────────────────────
        queue!(
            out,
            SetForegroundColor(Color::Cyan),
            Print(format!(
                "  DUAN 海战仿真{:>56}\n",
                format!("t = {:6.1}s", frame.sim_time)
            )),
            Print(
                "  ──────────────────────────────────────────────────────────────────────\n"
            ),
            ResetColor,
        )?;

        // ── 构建地图网格 ─────────────────────────────────
        // None = 空格；Some((team, is_ship)) = 实体
        let mut grid: Vec<Vec<Option<(u8, bool)>>> = vec![vec![None; MAP_W]; MAP_H];

        // 导弹先绘（低优先级，被舰船覆盖）
        for m in &frame.missiles {
            let (c, r) = to_map_cell(m.x, m.y);
            if grid[r][c].is_none() {
                grid[r][c] = Some((m.team, false));
            }
        }
        // 舰船后绘（高优先级）
        for s in &frame.ships {
            if s.alive {
                let (c, r) = to_map_cell(s.x, s.y);
                grid[r][c] = Some((s.team, true));
            }
        }

        // ── 地图 + 右侧状态栏并排渲染 ───────────────────
        // 收集各队舰船在 frame.ships 中的下标，支持任意舰队规模
        let red_idxs: Vec<usize> = frame
            .ships
            .iter()
            .enumerate()
            .filter(|(_, s)| s.team == 0)
            .map(|(i, _)| i)
            .collect();
        let blue_idxs: Vec<usize> = frame
            .ships
            .iter()
            .enumerate()
            .filter(|(_, s)| s.team == 1)
            .map(|(i, _)| i)
            .collect();
        // 右侧面板行布局：红方标题 → 红方舰船 → 空行 → 蓝方标题 → 蓝方舰船 → 空行 → 导弹统计
        let blue_hdr_row = red_idxs.len() + 2;
        let stats_row = blue_hdr_row + blue_idxs.len() + 2;

        // 地图顶部边框
        queue!(
            out,
            SetForegroundColor(Color::DarkGrey),
            Print(format!("  ┌{}┐\n", "─".repeat(MAP_W))),
            ResetColor,
        )?;

        for (row, grid_row) in grid.iter().enumerate() {
            // 左边框
            queue!(out, SetForegroundColor(Color::DarkGrey), Print("  │"), ResetColor)?;

            // 地图内容
            for cell in grid_row.iter() {
                match cell {
                    None => queue!(out, Print(" "))?,
                    Some((team, is_ship)) => {
                        let color = if *team == 0 { Color::Red } else { Color::Cyan };
                        queue!(out, SetForegroundColor(color))?;
                        let ch = if *is_ship {
                            if *team == 0 { "▲" } else { "▼" }
                        } else {
                            "·"
                        };
                        queue!(out, Print(ch), ResetColor)?;
                    }
                }
            }

            // 右边框
            queue!(out, SetForegroundColor(Color::DarkGrey), Print("│  "), ResetColor)?;

            // 右侧状态列（动态，支持任意舰队规模）
            if row == 0 {
                queue!(out, SetForegroundColor(Color::Red), Print("红方舰队"), ResetColor)?;
            } else if row >= 1 && row < 1 + red_idxs.len() {
                render_ship_line(&mut out, frame.ships.get(red_idxs[row - 1]))?;
            } else if row == blue_hdr_row {
                queue!(out, SetForegroundColor(Color::Cyan), Print("蓝方舰队"), ResetColor)?;
            } else if row > blue_hdr_row && row <= blue_hdr_row + blue_idxs.len() {
                render_ship_line(
                    &mut out,
                    frame.ships.get(blue_idxs[row - blue_hdr_row - 1]),
                )?;
            } else if row == stats_row {
                queue!(
                    out,
                    Print(format!(
                        "飞行: {:2}   总发: {:3}   命中: {:3}",
                        frame.active_missile_count, frame.total_missiles, frame.total_hits
                    ))
                )?;
            }

            queue!(out, Print("\n"))?;
        }

        // 地图底部边框
        queue!(
            out,
            SetForegroundColor(Color::DarkGrey),
            Print(format!("  └{}┘\n", "─".repeat(MAP_W))),
            ResetColor,
        )?;

        // ── 事件日志 ─────────────────────────────────────
        queue!(
            out,
            SetForegroundColor(Color::DarkGrey),
            Print(
                "  ──────────────────────────────────────────────────────────────────────\n"
            ),
            ResetColor,
            Print("  最近事件:\n"),
        )?;

        for i in 0..LOG_LINES {
            if let Some(line) = frame.recent_log.get(i) {
                let color = if line.contains("击沉") {
                    Color::Red
                } else if line.contains("命中") {
                    Color::Yellow
                } else if line.contains("开火") {
                    Color::Green
                } else {
                    Color::White
                };
                queue!(
                    out,
                    SetForegroundColor(color),
                    Print(format!("  {line}\n")),
                    ResetColor
                )?;
            } else {
                // 用空行覆盖上一帧可能残留的内容
                queue!(out, Print("  \n"))?;
            }
        }

        out.flush()
    }
}

impl Drop for NavalDisplay {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
    }
}

fn render_ship_line(out: &mut io::Stdout, ship: Option<&ShipFrame>) -> io::Result<()> {
    let Some(s) = ship else {
        return Ok(());
    };
    if !s.alive {
        return queue!(
            out,
            SetForegroundColor(Color::DarkGrey),
            Print(format!("✕ {} [已击沉]", s.name)),
            ResetColor,
        );
    }
    let color = if s.team == 0 { Color::Red } else { Color::Cyan };
    let symbol = if s.team == 0 { "▲" } else { "▼" };
    let hp_ratio = (s.health / s.max_health).clamp(0.0, 1.0);
    let filled = (hp_ratio * HP_BAR_W as f64) as usize;
    let empty = HP_BAR_W - filled;
    let bar_color = if hp_ratio > 0.5 {
        Color::Green
    } else if hp_ratio > 0.25 {
        Color::Yellow
    } else {
        Color::Red
    };
    queue!(
        out,
        SetForegroundColor(color),
        Print(format!("{symbol} {} ", s.name)),
        ResetColor,
        Print("["),
        SetForegroundColor(bar_color),
        Print("█".repeat(filled)),
        SetForegroundColor(Color::DarkGrey),
        Print("·".repeat(empty)),
        ResetColor,
        Print(format!("] {:5.0}/{:.0}", s.health, s.max_health)),
    )
}
