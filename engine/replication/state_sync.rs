use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub last_included_index: u64,
    pub last_included_term: u64,
    pub data: Vec<u8>,
}

pub struct StateSync;

impl StateSync {
    pub fn new() -> Self {
        Self
    }

    pub fn create_snapshot(&self, last_index: u64, last_term: u64, state: Vec<u8>) -> Snapshot {
        Snapshot {
            last_included_index: last_index,
            last_included_term: last_term,
            data: state,
        }
    }

    pub fn install_snapshot(&self, _snapshot: Snapshot) -> Result<()> {
        Ok(())
    }
}
