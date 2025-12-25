use crate::determinism::clock::{HybridLogicalClock, LogicalTime};
use crate::ffi::storage::StorageEngine;
use anyhow::Result;

pub struct ReplayEngine {
    clock: HybridLogicalClock,
}

impl ReplayEngine {
    pub fn new(frozen_time: u64) -> Self {
        Self {
            clock: HybridLogicalClock::new_deterministic(frozen_time),
        }
    }

    pub fn replay_wal(&self, storage: &StorageEngine) -> Result<()> {
        storage.wal_replay()?;
        Ok(())
    }

    pub fn current_time(&self) -> LogicalTime {
        self.clock.now()
    }

    pub fn advance_time(&self) -> LogicalTime {
        self.clock.advance()
    }
}
