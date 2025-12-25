# Deterministic Execution

## Overview

Deterministic execution guarantees that identical inputs produce identical outputs with identical timing. This is a core differentiator of minsql.

## Why Determinism?

### Reproducible Debugging

When a query fails or produces unexpected results, deterministic execution allows exact reproduction:

```
Transaction ID: 12345
Query: retrieve users where age > 18
Result: 487 rows

Re-execute with same transaction ID:
Result: 487 rows (guaranteed identical)
```

### Consistent Replication

Replicas executing the same operations produce identical state:

```
Primary: Execute query Q with input I → State S
Replica: Replay query Q with input I → State S (guaranteed identical)
```

This eliminates divergence issues common in asynchronous replication.

### Audit Compliance

Financial and regulated industries require reproducible execution:

```
Audit query: "What was account balance at 2024-11-10 12:03:21?"

Execute today: Balance = $1,523.45
Execute next year: Balance = $1,523.45 (guaranteed identical)
```

### Performance Testing

Deterministic timing enables reliable performance regression detection:

```
Version 1.0: Query takes 42.3ms
Version 1.1: Query takes 43.1ms

Regression is real, not measurement noise.
```

## Sources of Nondeterminism

### System Clock

Reading the system clock produces different values:

```c
time_t now = time(NULL);  // Different every call
```

### Thread Scheduling

OS thread scheduling is nondeterministic:

```
Thread A and Thread B both ready
OS may schedule A first, or B first
```

### Hash Tables

Hash table iteration order depends on memory addresses:

```rust
let mut map = HashMap::new();
for (key, value) in map.iter() {
    // Order is nondeterministic
}
```

### Memory Allocation

Pointer addresses are nondeterministic:

```c
void* p1 = malloc(100);
void* p2 = malloc(100);
// Relationship between p1 and p2 is nondeterministic
```

### Random Numbers

Random number generators produce different sequences:

```rust
let x = rand::random::<u64>();  // Different every call
```

### I/O Timing

Disk and network I/O timing varies:

```
Read from disk: May take 5ms or 50ms
```

## Deterministic Mode Design

### Logical Time

Replace system clock with Hybrid Logical Clock (HLC):

```rust
struct HLC {
    logical: u64,
    physical: u64,
}

impl HLC {
    fn now(&mut self) -> LogicalTime {
        self.logical += 1;
        LogicalTime {
            logical: self.logical,
            physical: self.physical,  // Frozen in deterministic mode
        }
    }
}
```

In deterministic mode:
- `physical` component is fixed at transaction start
- Only `logical` component advances
- Multiple calls to `now()` produce deterministic sequence

### Deterministic Scheduling

Use a deterministic scheduler:

```rust
struct DeterministicScheduler {
    ready_queue: BTreeMap<TaskId, Task>,
}

impl DeterministicScheduler {
    fn schedule_next(&mut self) -> Task {
        // Always choose lowest TaskId
        self.ready_queue.pop_first().unwrap()
    }
}
```

Tasks are scheduled in deterministic order based on TaskId, not OS scheduling.

### Deterministic Hash Tables

Use BTreeMap instead of HashMap:

```rust
// Nondeterministic
let map = HashMap::new();

// Deterministic
let map = BTreeMap::new();
```

BTreeMap iteration order is deterministic (sorted by key).

### Deterministic Memory Allocation

Use arena allocators with deterministic addresses:

```rust
struct Arena {
    base: *mut u8,
    offset: usize,
}

impl Arena {
    fn alloc(&mut self, size: usize) -> *mut u8 {
        let ptr = unsafe { self.base.add(self.offset) };
        self.offset += size;
        ptr  // Deterministic address
    }
}
```

Allocations happen at predictable addresses.

### Seeded Randomness

Seed random number generators:

```rust
let mut rng = StdRng::seed_from_u64(transaction_id);
let x = rng.gen::<u64>();  // Deterministic for given seed
```

Same seed produces same sequence.

### Deterministic I/O

Model I/O timing deterministically:

```rust
fn read_page(page_id: PageId) -> Page {
    // Nondeterministic: actual disk I/O
    let page = disk_read(page_id);
    
    // Deterministic: fixed cost model
    logical_clock.advance_by(PAGE_READ_COST);
    
    page
}
```

Logical time advances by fixed cost, not actual I/O time.

## Transaction Isolation

### Snapshot Isolation

Deterministic transactions use snapshot isolation:

```rust
struct DeterministicTransaction {
    id: TransactionId,
    snapshot: Snapshot,
    logical_time: LogicalTime,
}
```

The snapshot is determined by logical time, not wall-clock time.

### Conflict Detection

Conflicting transactions are serialized deterministically:

```
Transaction A: Update row 1
Transaction B: Update row 1

Serialization order: Transaction with lower ID goes first
```

This prevents nondeterministic deadlock resolution.

## WAL Replay

### Idempotent Replay

WAL entries are replayed idempotently:

```rust
struct WALEntry {
    transaction_id: TransactionId,
    logical_time: LogicalTime,
    operation: Operation,
}
```

Replaying the same WAL produces identical state.

### Deterministic Recovery

Crash recovery is deterministic:

1. Replay WAL from checkpoint
2. Apply operations in logical time order
3. Reconstruct identical state

No matter how many times recovery runs, final state is identical.

## API

### Enabling Deterministic Mode

```
begin deterministic transaction
  retrieve users order by id
commit
```

This transaction executes deterministically.

### Deterministic Queries

```
retrieve users
where age > 18
with deterministic = true
```

Single query executes deterministically.

### Timestamp Pinning

```
begin deterministic transaction at timestamp '2024-11-10 12:03:21'
  retrieve users where age > 18
commit
```

Logical time is pinned to specified value.

## Implementation Details

### Clock Management

```rust
pub struct Clock {
    mode: ClockMode,
    hlc: HLC,
}

pub enum ClockMode {
    Realtime,
    Deterministic { frozen_physical: u64 },
}

impl Clock {
    pub fn now(&mut self) -> LogicalTime {
        match self.mode {
            ClockMode::Realtime => {
                let physical = system_clock_micros();
                self.hlc.now(physical)
            }
            ClockMode::Deterministic { frozen_physical } => {
                self.hlc.now(frozen_physical)
            }
        }
    }
}
```

### Task Scheduling

```rust
pub struct Task {
    id: TaskId,
    priority: Priority,
    work: Box<dyn FnOnce()>,
}

pub struct Scheduler {
    ready: BTreeMap<TaskId, Task>,
    blocked: HashMap<TaskId, Task>,
}

impl Scheduler {
    pub fn schedule(&mut self) -> Option<Task> {
        self.ready.pop_first().map(|(_, task)| task)
    }
}
```

### Expression Evaluation

Expressions evaluate deterministically:

```rust
fn eval_expression(expr: &Expr, tuple: &Tuple, clock: &mut Clock) -> Value {
    match expr {
        Expr::Now => Value::Timestamp(clock.now()),
        Expr::Random => {
            let seed = clock.logical_time().logical;
            let mut rng = StdRng::seed_from_u64(seed);
            Value::Float(rng.gen())
        }
        _ => { /* Standard evaluation */ }
    }
}
```

## Limitations

### Performance Cost

Deterministic mode has overhead:

- Logical clock management
- Deterministic scheduling
- BTreeMap instead of HashMap
- Arena allocation

Typical overhead: 10-20% for deterministic mode.

### I/O Modeling

Deterministic I/O timing uses fixed cost models, which may not reflect actual system behavior.

### External Systems

Interactions with external systems (file system, network) cannot be made fully deterministic.

## Testing

### Determinism Validation

Test that identical inputs produce identical outputs:

```rust
#[test]
fn test_deterministic_query() {
    let query = "retrieve users where age > 18";
    let snapshot = create_snapshot();
    
    let result1 = execute_deterministic(query, snapshot.clone());
    let result2 = execute_deterministic(query, snapshot.clone());
    
    assert_eq!(result1, result2);
}
```

### Timing Validation

Test that timing is identical:

```rust
#[test]
fn test_deterministic_timing() {
    let query = "retrieve users where age > 18";
    let snapshot = create_snapshot();
    
    let (result1, time1) = execute_with_timing(query, snapshot.clone());
    let (result2, time2) = execute_with_timing(query, snapshot.clone());
    
    assert_eq!(time1, time2);
}
```

### Replay Validation

Test WAL replay:

```rust
#[test]
fn test_wal_replay_determinism() {
    let wal = generate_wal_entries();
    
    let state1 = replay_wal(wal.clone());
    let state2 = replay_wal(wal.clone());
    
    assert_eq!(state1, state2);
}
```

## Monitoring

### Determinism Metrics

Track determinism-related metrics:

```rust
struct DeterminismMetrics {
    logical_time: u64,
    tasks_scheduled: u64,
    clock_advances: u64,
}
```

### Audit Logging

Log all operations in deterministic mode:

```
[DETERMINISTIC] Transaction 12345 started at logical_time=1000
[DETERMINISTIC] Query executed: retrieve users where age > 18
[DETERMINISTIC] Logical time advanced to 1042
[DETERMINISTIC] Transaction committed at logical_time=1042
```

## Best Practices

### When to Use Deterministic Mode

Use deterministic mode for:
- Audit-critical operations
- Debugging production issues
- Performance testing
- Compliance requirements

### When Not to Use

Avoid deterministic mode for:
- High-throughput workloads (overhead may be prohibitive)
- Real-time applications (fixed I/O costs may not match reality)
- Exploratory analysis (determinism not required)

### Combining with Time Travel

Deterministic mode + time travel enables powerful debugging:

```
begin deterministic transaction at timestamp '2024-11-10 12:03:21'
  retrieve users where age > 18
commit
```

This query is:
- Deterministic (same results every time)
- Historical (queries past state)
- Reproducible (can be re-executed exactly)

## Future Enhancements

### Adaptive Determinism

Automatically enable determinism when needed:
- On query failure
- During debugging
- For compliance queries

### Determinism Verification

Automatically verify determinism:
- Execute query twice
- Compare results
- Alert on divergence

### Distributed Determinism

Extend determinism to distributed queries:
- Deterministic cross-shard coordination
- Deterministic network timing
- Deterministic failure handling
