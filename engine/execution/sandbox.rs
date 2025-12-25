use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct QueryLimits {
    pub max_cpu_time: Duration,
    pub max_memory: usize,
    pub max_wall_time: Duration,
}

impl Default for QueryLimits {
    fn default() -> Self {
        Self {
            max_cpu_time: Duration::from_secs(60),
            max_memory: 100 * 1024 * 1024,
            max_wall_time: Duration::from_secs(300),
        }
    }
}

pub struct Sandbox {
    limits: QueryLimits,
    start_time: Instant,
    memory_used: usize,
}

impl Sandbox {
    pub fn new(limits: QueryLimits) -> Self {
        Self {
            limits,
            start_time: Instant::now(),
            memory_used: 0,
        }
    }

    pub fn check(&self) -> anyhow::Result<()> {
        let elapsed = self.start_time.elapsed();
        
        if elapsed > self.limits.max_wall_time {
            anyhow::bail!("Query exceeded wall time limit");
        }

        if self.memory_used > self.limits.max_memory {
            anyhow::bail!("Query exceeded memory limit");
        }

        Ok(())
    }

    pub fn track_memory(&mut self, bytes: usize) {
        self.memory_used += bytes;
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}
