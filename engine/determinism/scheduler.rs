use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(pub u64);

pub struct Task {
    pub id: TaskId,
    pub priority: u8,
    pub work: Arc<dyn Fn() + Send + Sync>,
}

pub struct DeterministicScheduler {
    ready_queue: Arc<Mutex<BTreeMap<TaskId, Task>>>,
    next_task_id: Arc<Mutex<u64>>,
}

impl DeterministicScheduler {
    pub fn new() -> Self {
        Self {
            ready_queue: Arc::new(Mutex::new(BTreeMap::new())),
            next_task_id: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn schedule(&self, priority: u8, work: Arc<dyn Fn() + Send + Sync>) -> TaskId {
        let mut next_id = self.next_task_id.lock().await;
        let task_id = TaskId(*next_id);
        *next_id += 1;
        drop(next_id);

        let task = Task {
            id: task_id,
            priority,
            work,
        };

        let mut queue = self.ready_queue.lock().await;
        queue.insert(task_id, task);

        task_id
    }

    pub async fn execute_next(&self) -> Option<TaskId> {
        let mut queue = self.ready_queue.lock().await;
        
        if let Some((task_id, task)) = queue.pop_first() {
            drop(queue);
            (task.work)();
            Some(task_id)
        } else {
            None
        }
    }

    pub async fn is_empty(&self) -> bool {
        self.ready_queue.lock().await.is_empty()
    }
}
