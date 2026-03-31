//! 时间管理
//!
//! 时间是仿真的基础维度，是基础设施而非域。
//! 所有域和实体都需要访问时间，时间是仿真的元数据。

use crate::entity::id::EntityId;
use ordered_float::OrderedFloat;
use std::collections::{BTreeMap, HashMap};

// ──── TimeClock ──────────────────────────────────────────────────────────

/// 仿真时钟
///
/// 管理仿真时间，支持暂停、加速等功能。
#[derive(Clone, Debug)]
pub struct TimeClock {
    /// 当前仿真时间（秒）
    pub sim_time: f64,
    /// 时间比例（1.0 = 实时，2.0 = 2倍速）
    pub time_scale: f64,
    /// 是否运行
    pub running: bool,
    /// 已执行步数
    pub step_count: u64,
    /// 当前帧的仿真步长（由 `step()` 在每帧开始时写入）
    pub current_dt: f64,
}

impl TimeClock {
    pub fn new() -> Self {
        Self {
            sim_time: 0.0,
            time_scale: 1.0,
            running: true,
            step_count: 0,
            current_dt: 0.0,
        }
    }

    pub fn paused() -> Self {
        Self {
            sim_time: 0.0,
            time_scale: 1.0,
            running: false,
            step_count: 0,
            current_dt: 0.0,
        }
    }

    pub fn with_scale(time_scale: f64) -> Self {
        Self {
            sim_time: 0.0,
            time_scale,
            running: true,
            step_count: 0,
            current_dt: 0.0,
        }
    }

    /// 推进时间，返回实际仿真帧时间（暂停时返回 0）
    pub fn tick(&mut self, real_dt: f64) -> f64 {
        if !self.running {
            return 0.0;
        }
        let sim_dt = real_dt * self.time_scale;
        self.sim_time += sim_dt;
        self.step_count += 1;
        sim_dt
    }

    pub fn pause(&mut self) {
        self.running = false;
    }

    pub fn resume(&mut self) {
        self.running = true;
    }

    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale;
    }

    pub fn reset(&mut self) {
        self.sim_time = 0.0;
        self.step_count = 0;
    }

    pub fn is_paused(&self) -> bool {
        !self.running
    }

    pub fn now(&self) -> f64 {
        self.sim_time
    }
}

impl Default for TimeClock {
    fn default() -> Self {
        Self::new()
    }
}

// ──── Timer ──────────────────────────────────────────────────────────────

/// 定时器回调
///
/// 当前唯一支持的行为是让实体在定时器触发时自毁。
/// 若需在特定时间发出事件，推荐在域的 `compute()` 中检查 `ctx.sim_time()` 并主动 `emit`。
#[derive(Clone, Debug)]
pub enum TimerCallback {
    /// 使实体在定时器触发时进入已销毁状态（自毁定时器）
    SelfDestruct,
}

/// 定时器
#[derive(Clone, Debug)]
pub struct Timer {
    pub id: String,
    pub trigger_at: f64,
    pub repeating: bool,
    pub interval: Option<f64>,
    pub callback: TimerCallback,
}

impl Timer {
    pub fn once(id: impl Into<String>, trigger_at: f64, callback: TimerCallback) -> Self {
        Self {
            id: id.into(),
            trigger_at,
            repeating: false,
            interval: None,
            callback,
        }
    }

    pub fn repeating(
        id: impl Into<String>,
        first_trigger: f64,
        interval: f64,
        callback: TimerCallback,
    ) -> Self {
        Self {
            id: id.into(),
            trigger_at: first_trigger,
            repeating: true,
            interval: Some(interval),
            callback,
        }
    }

    pub fn self_destruct(trigger_at: f64) -> Self {
        Self::once("self_destruct", trigger_at, TimerCallback::SelfDestruct)
    }
}

// ──── TimerEvent ─────────────────────────────────────────────────────────

/// 定时器触发事件
#[derive(Clone, Debug)]
pub struct TimerEvent {
    pub entity_id: EntityId,
    pub timer_id: String,
    pub callback: TimerCallback,
}

// ──── TimerManager ───────────────────────────────────────────────────────

/// 定时器管理器
pub struct TimerManager {
    pending: BTreeMap<(OrderedFloat<f64>, EntityId, String), TimerCallback>,
    repeating: HashMap<(EntityId, String), (f64, TimerCallback)>,
}

impl TimerManager {
    pub fn new() -> Self {
        Self {
            pending: BTreeMap::new(),
            repeating: HashMap::new(),
        }
    }

    pub fn schedule(&mut self, entity_id: EntityId, timer: Timer) {
        let key = (OrderedFloat(timer.trigger_at), entity_id, timer.id.clone());
        self.pending.insert(key, timer.callback.clone());

        if timer.repeating {
            self.repeating.insert(
                (entity_id, timer.id),
                (timer.interval.unwrap_or(1.0), timer.callback),
            );
        }
    }

    pub fn cancel(&mut self, entity_id: EntityId, timer_id: &str) {
        self.pending
            .retain(|(_, eid, tid), _| *eid != entity_id || tid != timer_id);
        self.repeating.remove(&(entity_id, timer_id.to_string()));
    }

    pub fn remove_entity(&mut self, entity_id: EntityId) {
        self.pending.retain(|(_, eid, _), _| *eid != entity_id);
        self.repeating.retain(|(eid, _), _| *eid != entity_id);
    }

    pub fn check(&mut self, sim_time: f64) -> Vec<TimerEvent> {
        let mut events = Vec::new();

        while let Some((&(OrderedFloat(trigger_at), _, _), _)) = self.pending.first_key_value() {
            if trigger_at > sim_time {
                break;
            }

            let ((_, entity_id, timer_id), callback) = self.pending.pop_first().unwrap();

            events.push(TimerEvent {
                entity_id,
                timer_id: timer_id.clone(),
                callback: callback.clone(),
            });

            if let Some((interval, cb)) = self.repeating.get(&(entity_id, timer_id.clone())) {
                let next_trigger = sim_time + interval;
                self.pending.insert(
                    (OrderedFloat(next_trigger), entity_id, timer_id),
                    cb.clone(),
                );
            }
        }

        events
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::id::EntityId;

    fn make_id(idx: u32) -> EntityId {
        EntityId::new(0, 0, idx)
    }

    #[test]
    fn test_time_clock() {
        let mut clock = TimeClock::new();
        assert_eq!(clock.tick(0.1), 0.1);
        assert_eq!(clock.now(), 0.1);
        assert_eq!(clock.step_count, 1);
    }

    #[test]
    fn test_pause() {
        let mut clock = TimeClock::new();
        clock.pause();
        assert_eq!(clock.tick(0.1), 0.0);
        assert_eq!(clock.now(), 0.0);
    }

    #[test]
    fn test_timer_manager() {
        let mut mgr = TimerManager::new();
        let id = make_id(1);
        mgr.schedule(id, Timer::once("t", 1.0, TimerCallback::SelfDestruct));

        assert!(mgr.check(0.5).is_empty());
        let evts = mgr.check(1.5);
        assert_eq!(evts.len(), 1);
        assert_eq!(evts[0].timer_id, "t");
    }
}
