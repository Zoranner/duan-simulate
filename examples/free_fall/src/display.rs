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
/// 速度计宽度（字符数，用于显示速度大小比例条）
const SPEEDOMETER_WIDTH: usize = 30;
/// 速度计最大量程（m/s），超过此值截断显示
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
    /// 该帧对应的仿真时间（秒），用于回放定时
    pub sim_time: f64,
    /// 小球当前高度（m）
    pub y: f64,
    /// 小球当前垂直速度（m/s，负值表示下落）
    pub vy: f64,
    /// 截至本帧的累计弹跳次数
    pub bounce_count: u32,
    /// 截至本帧最近一次碰撞的快照；尚未发生碰撞时为 None
    pub last_collision: Option<CollisionSnapshot>,
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
                frame.sim_time, frame.bounce_count
            ))
        )?;

        // ── 竖向高度轨道 ─────────────────────────────────
        // row 0 对应最大高度，row TRACK_ROWS-1 对应地面（y=0）
        let ball_row = {
            let ratio = 1.0 - frame.y.clamp(0.0, self.max_height) / self.max_height;
            (ratio * (TRACK_ROWS - 1) as f64).round() as usize
        };
        let ball_col = TRACK_WIDTH / 2;

        for row in 0..TRACK_ROWS {
            let h = self.max_height * (1.0 - row as f64 / (TRACK_ROWS - 1) as f64);
            queue!(out, Print(format!("  {:5.1} │", h)))?;

            for col in 0..TRACK_WIDTH {
                if col == ball_col && row == ball_row {
                    // 下落时黄色（●），弹起时绿色（●）
                    let color = if frame.vy <= 0.0 {
                        Color::Yellow
                    } else {
                        Color::Green
                    };
                    queue!(out, SetForegroundColor(color), Print("●"), ResetColor)?;
                } else {
                    queue!(out, Print(" "))?;
                }
            }
            queue!(out, Print(" \n"))?;
        }

        // 地面线
        queue!(
            out,
            SetForegroundColor(Color::DarkGreen),
            Print(format!(
                "    0.0 ╘{:═<width$}╛\n",
                "",
                width = TRACK_WIDTH + 1
            )),
            ResetColor,
        )?;

        // ── 数值读数 ─────────────────────────────────────
        let dir = if frame.vy >= 0.0 { '↑' } else { '↓' };
        let speed = frame.vy.abs();
        queue!(
            out,
            Print(format!("\n  高度  {:8.4} m\n", frame.y)),
            Print(format!("  速度  {}  {:8.4} m/s\n", dir, speed)),
        )?;

        // 速度大小比例条（直观显示弹跳衰减趋势）
        let filled = ((speed / SPEEDOMETER_MAX).min(1.0) * SPEEDOMETER_WIDTH as f64) as usize;
        let empty = SPEEDOMETER_WIDTH - filled;
        let bar_color = if frame.vy <= 0.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        queue!(
            out,
            Print("  速度计 ["),
            SetForegroundColor(bar_color),
            Print(format!("{:█<filled$}", "", filled = filled)),
            ResetColor,
            Print(format!(
                "{dots:·<empty$}] {speed:.0}/{max:.0} m/s\n",
                dots = "",
                empty = empty,
                speed = speed,
                max = SPEEDOMETER_MAX
            )),
        )?;

        // ── 碰撞状态行 ───────────────────────────────────
        queue!(out, Print("\n  "))?;
        match frame.last_collision {
            Some(c) => queue!(
                out,
                SetForegroundColor(Color::Red),
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
