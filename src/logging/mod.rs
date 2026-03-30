//! 统一日志模块
//!
//! 提供框架级观测与业务级调试的统一接口，默认启用内置终端日志（`Logger`）。
//!
//! # 设计原则
//!
//! - 不依赖 `CustomEvent` 事件流，独立于仿真语义事件。
//! - 通过 `LogSink` trait 抽象后端，用户可自行桥接 `log` / `tracing` 等生态。
//! - 框架在关键阶段自动补齐 `sim_time`、`step_count`、`phase`、`entity_id` 等上下文。
//!
//! # 快速开始
//!
//! ```rust,ignore
//! use duan::logging::{LogSink, LogRecord, LogLevel, LogContext, FramePhase};
//! use std::sync::Arc;
//!
//! struct PrintLogger;
//! impl LogSink for PrintLogger {
//!     fn log(&self, record: &LogRecord) {
//!         println!("[{}][{}] {}", record.level, record.phase, record.message);
//!     }
//! }
//!
//! let world = duan::World::builder()
//!     .with_logger(Arc::new(PrintLogger))
//!     .build();
//! ```

use crate::entity::id::EntityId;
use std::fmt;
use std::sync::Arc;

// ──── LogLevel ────────────────────────────────────────────────────────────

/// 日志级别
///
/// 独立于具体后端（`log` / `tracing`），避免公共 API 直接绑死外部 crate。
/// 与 `log::Level` 语义等价，方便用户在适配器里转换。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum LogLevel {
    /// 精细调试信息，热路径慎用
    Trace,
    /// 调试信息，开发期使用
    Debug,
    /// 一般信息
    Info,
    /// 预期外但可继续运行的状况
    Warn,
    /// 严重错误
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => f.write_str("TRACE"),
            LogLevel::Debug => f.write_str("DEBUG"),
            LogLevel::Info => f.write_str("INFO"),
            LogLevel::Warn => f.write_str("WARN"),
            LogLevel::Error => f.write_str("ERROR"),
        }
    }
}

// ──── FramePhase ──────────────────────────────────────────────────────────

/// 仿真帧阶段
///
/// 标识日志记录发生在哪个仿真循环阶段，便于过滤和分析。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FramePhase {
    /// Phase 1：时间推进（`clock.tick`）
    StepStart,
    /// Phase 2：实体 tick
    EntityTick,
    /// Phase 3：域计算
    DomainCompute,
    /// Phase 4a：定时器触发
    TimerDispatch,
    /// Phase 4b：事件分发
    EventDispatch,
    /// Phase 5：生命周期管理
    StepEnd,
    /// 构建期（`WorldBuilder::build`、`Scheduler::build`）
    Build,
    /// 不属于特定阶段（用户自定义或框架外部调用）
    None,
}

impl fmt::Display for FramePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FramePhase::StepStart => f.write_str("StepStart"),
            FramePhase::EntityTick => f.write_str("EntityTick"),
            FramePhase::DomainCompute => f.write_str("DomainCompute"),
            FramePhase::TimerDispatch => f.write_str("TimerDispatch"),
            FramePhase::EventDispatch => f.write_str("EventDispatch"),
            FramePhase::StepEnd => f.write_str("StepEnd"),
            FramePhase::Build => f.write_str("Build"),
            FramePhase::None => f.write_str("-"),
        }
    }
}

// ──── LogContext ──────────────────────────────────────────────────────────

/// 帧日志上下文
///
/// 将每条日志携带的时间维度字段打包，减少参数个数。
/// 框架在 `EntityContext` / `DomainContext` / `step.rs` 内自动构造此结构，
/// 用户也可以在 `Logger::log` 实现里自由解构。
#[derive(Clone, Copy, Debug)]
pub struct LogContext {
    /// 发生阶段
    pub phase: FramePhase,
    /// 当前仿真时间（秒）
    pub sim_time: f64,
    /// 当前帧时间步长（秒），`Build` 阶段为 0.0
    pub dt: f64,
    /// 已执行步数，`Build` 阶段为 0
    pub step_count: u64,
    /// 关联实体 ID（非实体上下文时为 `None`）
    pub entity_id: Option<EntityId>,
}

impl LogContext {
    /// 构造帧上下文（用于 step.rs 框架打点）
    pub fn new(
        phase: FramePhase,
        sim_time: f64,
        dt: f64,
        step_count: u64,
        entity_id: Option<EntityId>,
    ) -> Self {
        Self {
            phase,
            sim_time,
            dt,
            step_count,
            entity_id,
        }
    }
}

// ──── LogRecord ───────────────────────────────────────────────────────────

/// 结构化日志记录
///
/// 由框架或用户代码构造，传递给 [`LogSink::log`]。
#[derive(Clone, Debug)]
pub struct LogRecord<'a> {
    /// 日志级别
    pub level: LogLevel,
    /// 帧上下文（阶段、时间、实体等）
    pub ctx: LogContext,
    /// 日志来源标识，类似 `log::Record::target()`
    pub target: &'a str,
    /// 格式化好的消息
    pub message: &'a str,
}

// ──── 字段委托快捷访问（减少 record.ctx.xxx）────────────────────────────

impl LogRecord<'_> {
    /// 发生阶段
    #[inline]
    pub fn phase(&self) -> FramePhase {
        self.ctx.phase
    }
    /// 当前仿真时间
    #[inline]
    pub fn sim_time(&self) -> f64 {
        self.ctx.sim_time
    }
    /// 帧时间步长
    #[inline]
    pub fn dt(&self) -> f64 {
        self.ctx.dt
    }
    /// 已执行步数
    #[inline]
    pub fn step_count(&self) -> u64 {
        self.ctx.step_count
    }
    /// 关联实体 ID
    #[inline]
    pub fn entity_id(&self) -> Option<EntityId> {
        self.ctx.entity_id
    }
}

// ──── LogSink trait ───────────────────────────────────────────────────────

/// 日志后端 trait
///
/// 实现此 trait 以接入自定义日志后端（终端打印、`log` crate、`tracing` span 等）。
///
/// # 实现要求
///
/// - 实现必须是线程安全的（`Send + Sync`）。
/// - 实现应尽量轻量，不在 `log` 方法内做阻塞 I/O。
/// - 对不感兴趣的级别或阶段应尽早返回，避免多余工作。
///
/// # 示例
///
/// ```rust,ignore
/// use duan::logging::{LogSink, LogRecord, LogLevel};
///
/// struct StderrLogger;
///
/// impl LogSink for StderrLogger {
///     fn log(&self, record: &LogRecord) {
///         if record.level >= LogLevel::Info {
///             eprintln!(
///                 "t={:.3} [{}][{}] {}",
///                 record.sim_time(), record.level, record.phase(), record.message
///             );
///         }
///     }
/// }
/// ```
pub trait LogSink: Send + Sync {
    /// 接收一条结构化日志记录
    fn log(&self, record: &LogRecord<'_>);

    /// 检查给定级别是否会被记录（用于跳过昂贵的消息构造）
    ///
    /// 默认返回 `true`（全量记录），实现可根据过滤规则提前返回 `false`。
    fn enabled(&self, level: LogLevel) -> bool {
        let _ = level;
        true
    }
}

// ──── 时间格式化工具 ──────────────────────────────────────────────────────

/// 将仿真时间（秒）格式化为可读时间字符串。
///
/// - 不足一天：`HH:MM:SS.mmm`（例如 `01:23:45.678`）
/// - 满一天及以上：`Nd+HH:MM:SS.mmm`（例如 `2d+14:30:00.000`）
pub fn fmt_sim_time(secs: f64) -> String {
    let total_ms = (secs * 1000.0).round() as u64;
    let ms = total_ms % 1000;
    let total_secs = total_ms / 1000;
    let s = total_secs % 60;
    let total_mins = total_secs / 60;
    let m = total_mins % 60;
    let total_hours = total_mins / 60;
    let h = total_hours % 24;
    let d = total_hours / 24;

    if d > 0 {
        format!("{d}d+{h:02}:{m:02}:{s:02}.{ms:03}")
    } else {
        format!("{h:02}:{m:02}:{s:02}.{ms:03}")
    }
}

// ──── 内置 Logger（默认）──────────────────────────────────────────────────

/// 框架默认日志后端（终端输出）。
///
/// 默认级别为 `Info`：打印关键流程，不打印热路径 Trace 明细。
pub struct Logger {
    min_level: LogLevel,
}

impl Logger {
    pub fn new(min_level: LogLevel) -> Self {
        Self { min_level }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new(LogLevel::Info)
    }
}

impl LogSink for Logger {
    fn log(&self, record: &LogRecord<'_>) {
        eprintln!(
            "[DUAN][{}][{:>5}][{:<13}][step={}] {}: {}",
            fmt_sim_time(record.sim_time()),
            record.level,
            record.phase(),
            record.step_count(),
            record.target,
            record.message
        );
    }

    fn enabled(&self, level: LogLevel) -> bool {
        level >= self.min_level
    }
}

// ──── LoggerHandle ────────────────────────────────────────────────────────

/// 框架内部持有的 logger 句柄
///
/// 封装 `Arc<dyn LogSink>`，并提供带 [`LogContext`] 的便捷记录方法。
/// 设计为 `Clone`，可随 `World` 的上下文结构廉价共享。
#[derive(Clone)]
pub struct LoggerHandle {
    inner: Arc<dyn LogSink>,
}

impl LoggerHandle {
    /// 从任意 `LogSink` 实现构造句柄
    pub fn new(logger: Arc<dyn LogSink>) -> Self {
        Self { inner: logger }
    }

    /// 构造默认句柄（内置 `Logger`，Info 级别）
    pub fn default_logger() -> Self {
        Self::new(Arc::new(Logger::default()))
    }

    /// 检查给定级别是否被记录
    #[inline]
    pub fn enabled(&self, level: LogLevel) -> bool {
        self.inner.enabled(level)
    }

    /// 记录一条日志（不做级别过滤）
    ///
    /// 注意：该方法会绕过 [`LogSink::enabled`] 检查。
    /// 大多数场景应优先使用 [`LoggerHandle::emit`] / `trace` / `debug` / `info` 等方法。
    #[inline]
    pub fn log(&self, record: &LogRecord<'_>) {
        self.inner.log(record);
    }

    /// 以给定级别记录日志
    ///
    /// 只有 `enabled(level)` 返回 `true` 时才实际调用 `log`，
    /// 避免调用方在外部重复判断。
    #[inline]
    pub fn emit(&self, level: LogLevel, ctx: LogContext, target: &str, message: &str) {
        if self.inner.enabled(level) {
            self.inner.log(&LogRecord {
                level,
                ctx,
                target,
                message,
            });
        }
    }

    // ──── 级别快捷方法 ──────────────────────────────────────────────────

    /// 记录 Trace 级别日志
    #[inline]
    pub fn trace(&self, ctx: LogContext, target: &str, message: &str) {
        self.emit(LogLevel::Trace, ctx, target, message);
    }

    /// 记录 Debug 级别日志
    #[inline]
    pub fn debug(&self, ctx: LogContext, target: &str, message: &str) {
        self.emit(LogLevel::Debug, ctx, target, message);
    }

    /// 记录 Info 级别日志
    #[inline]
    pub fn info(&self, ctx: LogContext, target: &str, message: &str) {
        self.emit(LogLevel::Info, ctx, target, message);
    }

    /// 记录 Warn 级别日志
    #[inline]
    pub fn warn(&self, ctx: LogContext, target: &str, message: &str) {
        self.emit(LogLevel::Warn, ctx, target, message);
    }

    /// 记录 Error 级别日志
    #[inline]
    pub fn error(&self, ctx: LogContext, target: &str, message: &str) {
        self.emit(LogLevel::Error, ctx, target, message);
    }
}

impl fmt::Debug for LoggerHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LoggerHandle")
    }
}

#[cfg(test)]
mod tests {
    use super::fmt_sim_time;

    #[test]
    fn test_fmt_sim_time_seconds() {
        assert_eq!(fmt_sim_time(0.0), "00:00:00.000");
        assert_eq!(fmt_sim_time(1.5), "00:00:01.500");
        assert_eq!(fmt_sim_time(59.999), "00:00:59.999");
    }

    #[test]
    fn test_fmt_sim_time_minutes() {
        assert_eq!(fmt_sim_time(90.0), "00:01:30.000");
        assert_eq!(fmt_sim_time(3599.0), "00:59:59.000");
    }

    #[test]
    fn test_fmt_sim_time_hours() {
        assert_eq!(fmt_sim_time(3600.0), "01:00:00.000");
        assert_eq!(fmt_sim_time(86399.0), "23:59:59.000");
    }

    #[test]
    fn test_fmt_sim_time_days() {
        assert_eq!(fmt_sim_time(86400.0), "1d+00:00:00.000");
        assert_eq!(fmt_sim_time(86400.0 * 2.0 + 3661.5), "2d+01:01:01.500");
    }
}
