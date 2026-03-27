//! 终端可视化显示模块
//!
//! 使用 crossterm 在终端备用屏幕中回放仿真帧序列。
//! `FreeFallDisplay` 是无状态渲染器，所有帧数据由 `RenderFrame` 携带。
//! 创建时自动进入备用屏幕，Drop 时自动恢复原始终端状态。

use crossterm::{
    cursor, execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
use std::io::{self, Write};

/// 轨道行数（每行对应 max_height / (TRACK_ROWS-1) 米）
const TRACK_ROWS: usize = 20;
/// 轨道内部宽度（字符数）
const TRACK_WIDTH: usize = 42;

/// 单帧渲染数据快照，由仿真阶段填充，回放阶段消费
#[derive(Clone)]
pub struct RenderFrame {
    /// 该帧对应的仿真时间（秒），用于回放定时
    pub sim_time: f64,
    pub y: f64,
    pub vy: f64,
    pub bounce_count: u32,
    /// 截至本帧最近一次碰撞的 (冲击速度, 弹性系数)
    pub last_collision: Option<(f64, f64)>,
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
        // row 0 = max_height，row TRACK_ROWS-1 = 0（贴近地面）
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
                    // 下落时黄色，上升时绿色
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
        queue!(
            out,
            Print(format!("\n  高度  {:8.4} m\n", frame.y)),
            Print(format!("  速度  {}  {:8.4} m/s\n", dir, frame.vy.abs())),
        )?;

        // ── 碰撞状态行 ───────────────────────────────────
        queue!(out, Print("\n  "))?;
        match frame.last_collision {
            Some((impact, restitution)) => queue!(
                out,
                SetForegroundColor(Color::Red),
                Print(format!(
                    "▶ 碰撞  冲击速度 {:6.3} m/s   弹性系数 {:.2}  ",
                    impact, restitution
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
