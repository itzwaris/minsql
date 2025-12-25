use std::time::Duration;

pub struct UdfSandbox {
    max_execution_time: Duration,
    max_memory: usize,
}

impl UdfSandbox {
    pub fn new(max_execution_time: Duration, max_memory: usize) -> Self {
        Self {
            max_execution_time,
            max_memory,
        }
    }

    pub fn check_limits(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
