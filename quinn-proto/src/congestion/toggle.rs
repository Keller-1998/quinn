use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use super::{Controller, ControllerFactory};

/// A toy congestion controller
#[derive(Debug, Clone)]
pub struct Toggle {
    config: Arc<ToggleConfig>,
    window: u64,
    last_window_change: Instant,
}

impl Toggle {
    /// Construct a state using the given `config` and current time `now`
    pub fn new(config: Arc<ToggleConfig>, now: Instant, _current_mtu: u16) -> Self {
        Self {
            window: config.windows.0,
            last_window_change: now,
            config,
        }
    }
}

impl ToggleConfig {
    /// Limits on the amount of outstanding data in bytes.
    pub fn windows(&mut self, values: (u64, u64)) {
        self.windows = values
    }

    /// Duration to toggle between the two congestion windows.
    pub fn toggle_time(&mut self, value: Duration) {
        self.toggle_time = value
    }
}

impl Controller for Toggle {
    fn on_ack(
        &mut self,
        now: Instant,
        _sent: Instant,
        _bytes: u64,
        app_limited: bool,
        _rtt: &crate::RttEstimator,
    ) {
        if app_limited {
            return;
        }
        if self.last_window_change.elapsed() >= self.config.toggle_time {
            if self.window == self.config.windows.0 {
                self.window = self.config.windows.1
            } else {
                self.window = self.config.windows.0
            }
            self.last_window_change = now
        }
    }

    fn on_congestion_event(
        &mut self,
        _now: Instant,
        _sent: Instant,
        _is_persistent_congestion: bool,
        _lost_bytes: u64,
    ) {
    }

    fn on_mtu_update(&mut self, _new_mtu: u16) {}

    fn window(&self) -> u64 {
        self.window
    }

    fn clone_box(&self) -> Box<dyn Controller> {
        Box::new(self.clone())
    }

    fn initial_window(&self) -> u64 {
        self.config.windows.0
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

/// Configuration for the `Toggle` congestion controller
#[derive(Debug, Clone)]
pub struct ToggleConfig {
    windows: (u64, u64),
    toggle_time: Duration,
}

impl Default for ToggleConfig {
    fn default() -> Self {
        Self {
            windows: (3_000_000, 6_000_000),
            toggle_time: Duration::from_secs(5),
        }
    }
}

impl ControllerFactory for ToggleConfig {
    fn build(self: Arc<Self>, now: Instant, current_mtu: u16) -> Box<dyn Controller> {
        Box::new(Toggle::new(self, now, current_mtu))
    }
}
