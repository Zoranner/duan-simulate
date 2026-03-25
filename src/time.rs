//! 时间管理
//!
//! 时间是仿真的基础维度，是基础设施而非域。
//! 所有域都需要访问时间，时间是仿真的元数据。

use crate::events::TimerCallback;
use crate::EntityId;
use ordered_float::OrderedFloat;
use std::collections::{BTreeMap, HashMap};

/// 仿真时钟
///
/// 管理仿真时间，支持暂停、加速等功能。
#[derive(Clone, Debug)]
pub struct TimeClock {
    /// 当前仿真时间（秒，从场景开始计）
    pub sim_time: f64,
    /// 时间比例（1.0 = 实时，2.0 = 2倍速，0 = 暂停）
    pub time_scale: f64,
    /// 是否运行
    pub running: bool,
    /// 已执行步数
    pub step_count: u64,
}

impl TimeClock {
    /// 创建新的仿真时钟
    pub fn new() -> Self {
        Self {
            sim_time: 0.0,
            time_scale: 1.0,
            running: true,
            step_count: 0,
        }
    }

    /// 创建暂停的时钟
    pub fn paused() -> Self {
        Self {
            sim_time: 0.0,
            time_scale: 1.0,
            running: false,
            step_count: 0,
        }
    }

    /// 创建指定时间比例的时钟
    pub fn with_scale(time_scale: f64) -> Self {
        Self {
            sim_time: 0.0,
            time_scale,
            running: true,
            step_count: 0,
        }
    }

    /// 推进时间
    ///
    /// 根据真实时间流逝和时间比例，推进仿真时间。
    /// 返回实际仿真帧时间。
    pub fn tick(&mut self, real_dt: f64) -> f64 {
        if !self.running {
            return 0.0;
        }
        let sim_dt = real_dt * self.time_scale;
        self.sim_time += sim_dt;
        self.step_count += 1;
        sim_dt
    }

    /// 暂停
    pub fn pause(&mut self) {
        self.running = false;
    }

    /// 恢复
    pub fn resume(&mut self) {
        self.running = true;
    }

    /// 设置时间比例
    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale;
    }

    /// 重置时钟
    pub fn reset(&mut self) {
        self.sim_time = 0.0;
        self.step_count = 0;
    }

    /// 检查是否暂停
    pub fn is_paused(&self) -> bool {
        !self.running
    }

    /// 获取当前仿真时间
    pub fn now(&self) -> f64 {
        self.sim_time
    }
}

impl Default for TimeClock {
    fn default() -> Self {
        Self::new()
    }
}

/// 定时器
#[derive(Clone, Debug)]
pub struct Timer {
    /// 定时器 ID
    pub id: String,
    /// 触发时间点（仿真时间）
    pub trigger_at: f64,
    /// 是否循环
    pub repeating: bool,
    /// 循环间隔（如果 repeating）
    pub interval: Option<f64>,
    /// 回调类型
    pub callback: TimerCallback,
}

impl Timer {
    /// 创建单次定时器
    pub fn once(id: impl Into<String>, trigger_at: f64, callback: TimerCallback) -> Self {
        Self {
            id: id.into(),
            trigger_at,
            repeating: false,
            interval: None,
            callback,
        }
    }

    /// 创建周期定时器
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

    /// 创建自毁定时器
    pub fn self_destruct(trigger_at: f64) -> Self {
        Self::once("self_destruct", trigger_at, TimerCallback::SelfDestruct)
    }
}

/// 定时器事件
#[derive(Clone, Debug)]
pub struct TimerEvent {
    /// 关联的实体
    pub entity_id: EntityId,
    /// 定时器 ID
    pub timer_id: String,
    /// 回调
    pub callback: TimerCallback,
}

/// 定时器管理器
///
/// 管理所有定时器，使用 BTreeMap 按触发时间排序。
/// 支持 O(log n) 插入和 O(1) 获取最早触发的定时器。
pub struct TimerManager {
    /// 按触发时间索引：(trigger_at, entity_id, timer_id) -> callback
    /// 使用 OrderedFloat 包装 f64 以支持 Ord
    pending: BTreeMap<(OrderedFloat<f64>, EntityId, String), TimerCallback>,
    /// 循环定时器记录（用于重新调度）
    repeating: HashMap<(EntityId, String), (f64, TimerCallback)>,
}

impl TimerManager {
    /// 创建新的定时器管理器
    pub fn new() -> Self {
        Self {
            pending: BTreeMap::new(),
            repeating: HashMap::new(),
        }
    }

    /// 调度定时器
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

    /// 取消定时器
    pub fn cancel(&mut self, entity_id: EntityId, timer_id: &str) {
        // 移除待触发的定时器
        self.pending
            .retain(|(_, eid, tid), _| *eid != entity_id || tid != timer_id);
        self.repeating.remove(&(entity_id, timer_id.to_string()));
    }

    /// 实体销毁时清理所有定时器
    pub fn remove_entity(&mut self, entity_id: EntityId) {
        self.pending.retain(|(_, eid, _), _| *eid != entity_id);
        self.repeating.retain(|(eid, _), _| *eid != entity_id);
    }

    /// 检查并返回到期的定时器事件
    ///
    /// 使用 BTreeMap 的有序特性，O(k) 获取 k 个到期的定时器。
    pub fn check(&mut self, sim_time: f64) -> Vec<TimerEvent> {
        let mut events = Vec::new();

        // 取出所有到期的定时器
        while let Some((&(OrderedFloat(trigger_at), _entity_id, ref _timer_id), _)) =
            self.pending.first_key_value()
        {
            if trigger_at > sim_time {
                break;
            }

            let ((_, entity_id, timer_id), callback) = self.pending.pop_first().unwrap();

            events.push(TimerEvent {
                entity_id,
                timer_id: timer_id.clone(),
                callback: callback.clone(),
            });

            // 如果是循环定时器，重新调度
            if let Some((interval, cb)) = self.repeating.get(&(entity_id, timer_id.clone())) {
                let next_trigger = sim_time + interval;
                self.pending.insert(
                    (OrderedFloat(next_trigger), entity_id, timer_id.clone()),
                    cb.clone(),
                );
            }
        }

        events
    }

    /// 获取待触发的定时器数量
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// 获取下一个定时器的触发时间
    pub fn next_trigger_time(&self) -> Option<f64> {
        self.pending
            .first_key_value()
            .map(|((OrderedFloat(t), _, _), _)| *t)
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

    #[test]
    fn test_time_clock() {
        let mut clock = TimeClock::new();

        assert_eq!(clock.now(), 0.0);
        assert_eq!(clock.tick(0.1), 0.1);
        assert_eq!(clock.now(), 0.1);
        assert_eq!(clock.step_count, 1);
    }

    #[test]
    fn test_time_scale() {
        let mut clock = TimeClock::with_scale(2.0);
        assert_eq!(clock.tick(0.1), 0.2);
        assert_eq!(clock.now(), 0.2);
    }

    #[test]
    fn test_pause() {
        let mut clock = TimeClock::new();
        clock.pause();

        assert!(clock.is_paused());
        assert_eq!(clock.tick(0.1), 0.0);
        assert_eq!(clock.now(), 0.0);
    }

    #[test]
    fn test_timer_manager() {
        let mut manager = TimerManager::new();
        let entity_id = EntityId::new(1);

        manager.schedule(
            entity_id,
            Timer::once("test", 1.0, TimerCallback::SelfDestruct),
        );

        assert_eq!(manager.len(), 1);

        let events = manager.check(0.5);
        assert!(events.is_empty());

        let events = manager.check(1.5);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].timer_id, "test");
    }

    #[test]
    fn test_repeating_timer() {
        let mut manager = TimerManager::new();
        let entity_id = EntityId::new(1);

        manager.schedule(
            entity_id,
            Timer::repeating("repeat", 1.0, 1.0, TimerCallback::SelfDestruct),
        );

        // 第一次触发
        let events = manager.check(1.0);
        assert_eq!(events.len(), 1);

        // 第二次触发（重新调度）
        let events = manager.check(2.0);
        assert_eq!(events.len(), 1);
    }
}
