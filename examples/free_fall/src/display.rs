//! 终端可视化显示模块
//!
//! 使用 crossterm 在终端备用屏幕中回放仿真帧序列。
//!
//! # 设计说明
//!
//! `FreeFallDisplay` 是**无状态渲染器**：它不持有仿真状态，所有渲染所需的数据
//! 都由 `RenderFrame` 携带。这使得 Phase 1（仿真）和 Phase 2（回放）完全解耦。
//!
//! 创建时自动进入备用屏幕（`EnterAlternateScreen`），Drop 时自动恢复原始终端状态，
//! 无论是正常退出还是 panic 都能正确清理。

use crossterm::{
    cursor, execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
use std::io::{self, Write};

/// 轨道行数（每行代表 max_height / (TRACK_ROWS-1) 米高度）
const TRACK_ROWS: usize = 20;
/// 轨道内部宽度（字符数）
const TRACK_WIDTH: usize = 42;
/// 能量条宽度（字符数，势能/动能均使用此宽度）
const BAR_WIDTH: usize = 30;
/// 动能条最大量程（m/s），超过此值截断显示
const SPEEDOMETER_MAX: f64 = 15.0;

/// 最近一次碰撞的快照数据
#[derive(Clone, Copy)]
pub struct CollisionSnapshot {
    /// 碰撞时的冲击速度大小（m/s，始终为正）
    pub impact_velocity: f64,
    /// 本次碰撞采用的弹性系数
    pub restitution: f64,
}

/// 单帧渲染数据快照，由仿真阶段填充，回放阶段消费
#[derive(Clone)]
pub struct RenderFrame {
    /// 该帧对应的世界时间（秒），用于回放定时
    pub time: f64,
    /// 小球当前高度（m）
    pub y: f64,
    /// 小球当前垂直速度（m/s，负值表示下落）
    pub vy: f64,
    /// 截至本帧的累计弹跳次数
    pub bounce_count: u32,
    /// 截至本帧最近一次碰撞的快照；尚未发生碰撞时为 None
    pub last_collision: Option<CollisionSnapshot>,
    /// 碰撞后短暂闪光标志（约 80ms），用于强调弹跳时刻
    pub just_bounced: bool,
}

/// 自由落体仿真的终端显示器（无状态渲染器）
pub struct FreeFallDisplay {
    max_height: f64,
}

impl FreeFallDisplay {
    /// 创建显示器并进入备用屏幕
    pub fn new(max_height: f64) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(Self { max_height })
    }

    /// 渲染一帧
    pub fn render(&self, frame: &RenderFrame) -> io::Result<()> {
        let mut out = io::stdout();

        // 每帧从左上角重绘，避免闪烁
        queue!(out, cursor::MoveTo(0, 0))?;

        // ── 标题栏 ──────────────────────────────────────
        queue!(
            out,
            SetForegroundColor(Color::Cyan),
            Print("  DUAN 自由落体仿真\n"),
            Print("  ──────────────────────────────────────────────────\n"),
            ResetColor,
        )?;

        // ── 全局状态行 ───────────────────────────────────
        queue!(
            out,
            Print(format!(
                "  仿真时间  {:6.2} s      弹跳次数  {}\n\n",
                frame.time, frame.bounce_count
            ))
        )?;

        // ── 速度计算与颜色分级 ────────────────────────────
        let speed = frame.vy.abs();
        let speed_ratio = speed / SPEEDOMETER_MAX;

        // 落下时按速度比例分级：暗→亮→红；上升为绿；碰撞瞬间为白
        let ball_color = if frame.just_bounced {
            Color::White
        } else if frame.vy >= 0.0 {
            Color::Green
        } else if speed_ratio < 0.3 {
            Color::DarkYellow
        } else if speed_ratio < 0.7 {
            Color::Yellow
        } else {
            Color::Red
        };
        let ball_char = "◉";

        // ── 竖向高度轨道 ─────────────────────────────────
        // row 0 对应最大高度，row TRACK_ROWS-1 对应地面（y=0）
        let ball_row = {
            let ratio = 1.0 - frame.y.clamp(0.0, self.max_height) / self.max_height;
            (ratio * (TRACK_ROWS - 1) as f64).round() as usize
        };
        let ball_col = TRACK_WIDTH / 2;

        for row in 0..TRACK_ROWS {
            let h = self.max_height * (1.0 - row as f64 / (TRACK_ROWS - 1) as f64);

            if row < TRACK_ROWS - 1 {
                // 普通行
                queue!(out, Print(format!("  {:5.1} │", h)))?;
                for col in 0..TRACK_WIDTH {
                    if col == ball_col && row == ball_row {
                        queue!(
                            out,
                            SetForegroundColor(ball_color),
                            Print(ball_char),
                            ResetColor
                        )?;
                    } else {
                        queue!(out, Print(" "))?;
                    }
                }
                queue!(out, Print(" \n"))?;
            } else {
                // 地面行：坐标轴零点即地面，消除重复的 0.0 标注
                // 小球在地面时直接坐在 ═ 线上；弹起后 ★ 在原碰撞列短暂残留
                queue!(
                    out,
                    SetForegroundColor(Color::DarkGreen),
                    Print(format!("  {:5.1} ╘", h))
                )?;
                for col in 0..TRACK_WIDTH + 1 {
                    if col == ball_col && ball_row == TRACK_ROWS - 1 {
                        queue!(
                            out,
                            ResetColor,
                            SetForegroundColor(ball_color),
                            Print(ball_char),
                            SetForegroundColor(Color::DarkGreen)
                        )?;
                    } else if col == ball_col && frame.just_bounced {
                        queue!(
                            out,
                            SetForegroundColor(Color::Yellow),
                            Print("★"),
                            SetForegroundColor(Color::DarkGreen)
                        )?;
                    } else {
                        queue!(out, Print("═"))?;
                    }
                }
                queue!(out, ResetColor, Print("╛\n"))?;
            }
        }

        // ── 数值读数 ─────────────────────────────────────
        let dir = if frame.vy >= 0.0 { '↑' } else { '↓' };
        queue!(
            out,
            Print(format!("\n  高度  {:8.4} m\n", frame.y)),
            Print(format!("  速度  {}  {:8.4} m/s\n", dir, speed)),
        )?;

        // ── 势能/动能双条（直观展示能量在 PE⇄KE 间的转换）──
        let height_ratio = (frame.y / self.max_height).min(1.0);
        let pe_filled = (height_ratio * BAR_WIDTH as f64) as usize;
        let pe_empty = BAR_WIDTH - pe_filled;
        queue!(
            out,
            Print("  势能 ["),
            SetForegroundColor(Color::Cyan),
            Print(format!("{:█<pe_filled$}", "", pe_filled = pe_filled)),
            ResetColor,
            Print(format!(
                "{dots:·<pe_empty$}] {y:5.2}/{max:.1} m\n",
                dots = "",
                pe_empty = pe_empty,
                y = frame.y,
                max = self.max_height,
            )),
        )?;

        let ke_filled = (speed_ratio.min(1.0) * BAR_WIDTH as f64) as usize;
        let ke_empty = BAR_WIDTH - ke_filled;
        queue!(
            out,
            Print("  动能 ["),
            SetForegroundColor(ball_color),
            Print(format!("{:█<ke_filled$}", "", ke_filled = ke_filled)),
            ResetColor,
            Print(format!(
                "{dots:·<ke_empty$}] {speed:5.1}/{max:.0} m/s\n",
                dots = "",
                ke_empty = ke_empty,
                speed = speed,
                max = SPEEDOMETER_MAX,
            )),
        )?;

        // ── 碰撞状态行 ───────────────────────────────────
        queue!(out, Print("\n  "))?;
        match frame.last_collision {
            Some(c) => queue!(
                out,
                SetForegroundColor(if frame.just_bounced {
                    Color::Yellow
                } else {
                    Color::Red
                }),
                Print(format!(
                    "▶ 碰撞  冲击速度 {:6.3} m/s   弹性系数 {:.2}  ",
                    c.impact_velocity, c.restitution
                )),
                ResetColor,
                Print("\n"),
            )?,
            None => queue!(out, Print("  等待落地...                              \n"))?,
        }

        out.flush()
    }
}

impl Drop for FreeFallDisplay {
    fn drop(&mut self) {
        // 无论正常退出还是 panic，都恢复终端状态
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
    }
}
