use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub entry_type: LogEntryType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogEntryType {
    Write,
    Config,
    Snapshot,
}

pub struct ReplicationLog {
    entries: Vec<LogEntry>,
    commit_index: u64,
    last_applied: u64,
}

impl ReplicationLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            commit_index: 0,
            last_applied: 0,
        }
    }

    pub fn append(&mut self, entry: LogEntry) {
        self.entries.push(entry);
    }

    pub fn get(&self, index: u64) -> Option<&LogEntry> {
        self.entries.get(index as usize)
    }

    pub fn last_index(&self) -> u64 {
        self.entries.len() as u64
    }

    pub fn last_term(&self) -> u64 {
        self.entries.last().map(|e| e.term).unwrap_or(0)
    }

    pub fn commit(&mut self, index: u64) {
        self.commit_index = index;
    }

    pub fn apply(&mut self, index: u64) {
        self.last_applied = index;
    }

    pub fn truncate(&mut self, from_index: u64) {
        self.entries.truncate(from_index as usize);
    }
}
