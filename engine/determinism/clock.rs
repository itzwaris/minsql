use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalTime {
    pub logical: u64,
    pub physical: u64,
}

impl LogicalTime {
    pub fn zero() -> Self {
        Self {
            logical: 0,
            physical: 0,
        }
    }

    pub fn new(logical: u64, physical: u64) -> Self {
        Self { logical, physical }
    }
}

pub enum ClockMode {
    Realtime,
    Deterministic { frozen_physical: u64 },
}

pub struct HybridLogicalClock {
    mode: ClockMode,
    logical_counter: AtomicU64,
}

impl HybridLogicalClock {
    pub fn new_realtime() -> Self {
        Self {
            mode: ClockMode::Realtime,
            logical_counter: AtomicU64::new(0),
        }
    }

    pub fn new_deterministic(frozen_physical: u64) -> Self {
        Self {
            mode: ClockMode::Deterministic { frozen_physical },
            logical_counter: AtomicU64::new(0),
        }
    }

    pub fn now(&self) -> LogicalTime {
        let physical = match &self.mode {
            ClockMode::Realtime => {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as u64
            }
            ClockMode::Deterministic { frozen_physical } => *frozen_physical,
        };

        let logical = self.logical_counter.fetch_add(1, Ordering::SeqCst);

        LogicalTime { logical, physical }
    }

    pub fn advance(&self) -> LogicalTime {
        self.now()
    }

    pub fn advance_by(&self, delta: u64) {
        self.logical_counter.fetch_add(delta, Ordering::SeqCst);
    }
}
