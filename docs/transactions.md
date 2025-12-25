# Transactions and MVCC

## Overview

minsql implements Multi-Version Concurrency Control (MVCC) with snapshot isolation by default. This allows concurrent readers and writers without blocking.

## Transaction Model

### ACID Properties

minsql provides full ACID guarantees:

**Atomicity**: Transactions are all-or-nothing. If any operation fails, all changes are rolled back.

**Consistency**: Transactions move the database from one consistent state to another. Integrity constraints are enforced.

**Isolation**: Concurrent transactions do not interfere with each other. Default isolation level is snapshot isolation.

**Durability**: Committed transactions survive crashes. WAL ensures durability.

## MVCC Implementation

### Transaction IDs

Each transaction receives a unique ID:

```rust
pub struct TransactionId(u64);

impl TransactionId {
    pub fn next() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        TransactionId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}
```

### Tuple Versions

Each tuple has version information:

```c
typedef struct TupleHeader {
    uint32_t t_xmin;     // Transaction that inserted this tuple
    uint32_t t_xmax;     // Transaction that deleted this tuple (0 if not deleted)
    uint16_t t_infomask; // Status flags
    uint8_t t_hoff;      // Header size
} TupleHeader;
```

### Snapshot

A snapshot captures the state of active transactions:

```rust
pub struct Snapshot {
    pub xid: TransactionId,
    pub xmin: TransactionId,
    pub xmax: TransactionId,
    pub active_xids: Vec<TransactionId>,
    pub logical_time: LogicalTime,
}
```

### Visibility Rules

A tuple is visible to a snapshot if:

```rust
fn is_visible(tuple: &Tuple, snapshot: &Snapshot) -> bool {
    let xmin = tuple.xmin();
    let xmax = tuple.xmax();
    
    // Inserted by a transaction that hasn't committed yet
    if xmin >= snapshot.xmax {
        return false;
    }
    
    // Inserted by an active transaction (not us)
    if snapshot.active_xids.contains(&xmin) && xmin != snapshot.xid {
        return false;
    }
    
    // Not deleted
    if xmax == 0 {
        return true;
    }
    
    // Deleted by a future transaction
    if xmax >= snapshot.xmax {
        return true;
    }
    
    // Deleted by an active transaction (not us)
    if snapshot.active_xids.contains(&xmax) && xmax != snapshot.xid {
        return true;
    }
    
    // Deleted by a committed transaction
    false
}
```

## Isolation Levels

### Read Committed

Each statement sees a fresh snapshot:

```
begin transaction isolation level read committed
  retrieve users where age > 18  -- Snapshot 1
  retrieve orders where total > 100  -- Snapshot 2 (potentially different)
commit
```

### Snapshot Isolation (Default)

All statements in a transaction see the same snapshot:

```
begin transaction
  retrieve users where age > 18  -- Snapshot 1
  retrieve orders where total > 100  -- Snapshot 1 (same)
commit
```

### Serializable

Provides full serializability by detecting conflicts:

```
begin transaction isolation level serializable
  retrieve users where age > 18
  update users set age = age + 1
commit
```

If concurrent transactions would violate serializability, one is aborted.

## Transaction States

### Transaction State Machine

```
               START
                 ↓
              ACTIVE
               ↙   ↘
        COMMITTED   ABORTED
```

### State Transitions

```rust
pub enum TransactionState {
    Active,
    Committed,
    Aborted,
}

pub struct Transaction {
    id: TransactionId,
    state: TransactionState,
    snapshot: Snapshot,
    logical_time: LogicalTime,
}
```

## Write Operations

### Insert

```rust
fn insert_tuple(table: &Table, tuple: Tuple, xid: TransactionId) -> Result<()> {
    // Set version info
    tuple.set_xmin(xid);
    tuple.set_xmax(0);
    
    // Write WAL entry
    let wal_entry = WALEntry::Insert {
        table_id: table.id,
        tuple: tuple.clone(),
    };
    wal_append(wal_entry)?;
    
    // Write to storage
    storage_insert(table, tuple)?;
    
    Ok(())
}
```

### Update

Update creates a new tuple version:

```rust
fn update_tuple(table: &Table, old_tuple: Tuple, new_tuple: Tuple, xid: TransactionId) -> Result<()> {
    // Mark old tuple as deleted
    old_tuple.set_xmax(xid);
    
    // Insert new tuple version
    new_tuple.set_xmin(xid);
    new_tuple.set_xmax(0);
    
    // Write WAL
    let wal_entry = WALEntry::Update {
        table_id: table.id,
        old_tuple: old_tuple.clone(),
        new_tuple: new_tuple.clone(),
    };
    wal_append(wal_entry)?;
    
    // Write to storage
    storage_update(table, old_tuple, new_tuple)?;
    
    Ok(())
}
```

### Delete

Delete marks a tuple as deleted:

```rust
fn delete_tuple(table: &Table, tuple: Tuple, xid: TransactionId) -> Result<()> {
    // Mark as deleted
    tuple.set_xmax(xid);
    
    // Write WAL
    let wal_entry = WALEntry::Delete {
        table_id: table.id,
        tuple: tuple.clone(),
    };
    wal_append(wal_entry)?;
    
    // Write to storage
    storage_delete(table, tuple)?;
    
    Ok(())
}
```

## Commit and Abort

### Commit

```rust
fn commit_transaction(tx: &Transaction) -> Result<()> {
    // Write commit record to WAL
    let wal_entry = WALEntry::Commit {
        xid: tx.id,
        logical_time: tx.logical_time,
    };
    wal_append(wal_entry)?;
    wal_flush()?;
    
    // Mark transaction as committed
    tx.state = TransactionState::Committed;
    
    Ok(())
}
```

### Abort

```rust
fn abort_transaction(tx: &Transaction) -> Result<()> {
    // Write abort record to WAL
    let wal_entry = WALEntry::Abort {
        xid: tx.id,
    };
    wal_append(wal_entry)?;
    
    // Mark transaction as aborted
    tx.state = TransactionState::Aborted;
    
    // Cleanup is handled by MVCC garbage collection
    Ok(())
}
```

## Deadlock Detection

### Wait-For Graph

Track transaction dependencies:

```rust
pub struct WaitForGraph {
    edges: HashMap<TransactionId, Vec<TransactionId>>,
}

impl WaitForGraph {
    pub fn add_edge(&mut self, waiter: TransactionId, holder: TransactionId) {
        self.edges.entry(waiter).or_default().push(holder);
    }
    
    pub fn has_cycle(&self) -> Option<Vec<TransactionId>> {
        // DFS-based cycle detection
        for start in self.edges.keys() {
            if let Some(cycle) = self.find_cycle_from(*start) {
                return Some(cycle);
            }
        }
        None
    }
}
```

### Deadlock Resolution

When a deadlock is detected, abort the youngest transaction:

```rust
fn resolve_deadlock(cycle: Vec<TransactionId>) -> TransactionId {
    // Abort transaction with highest ID (youngest)
    *cycle.iter().max().unwrap()
}
```

## Garbage Collection

### Vacuum

Remove old tuple versions that are no longer visible:

```rust
fn vacuum_table(table: &Table) -> Result<()> {
    let oldest_active_xid = get_oldest_active_transaction();
    
    for page in table.pages() {
        for tuple in page.tuples() {
            // Tuple deleted by old transaction
            if tuple.xmax() > 0 && tuple.xmax() < oldest_active_xid {
                // No transaction can see this tuple anymore
                remove_tuple(page, tuple)?;
            }
        }
    }
    
    Ok(())
}
```

### Autovacuum

Automatic vacuum based on dead tuple ratio:

```rust
fn autovacuum_daemon() {
    loop {
        for table in get_tables() {
            let stats = get_table_stats(table);
            
            if stats.dead_tuple_ratio() > AUTOVACUUM_THRESHOLD {
                vacuum_table(table)?;
            }
        }
        
        sleep(AUTOVACUUM_INTERVAL);
    }
}
```

## Time-Travel Queries

### Historical Snapshots

Query data as it existed at a specific time:

```
retrieve users
where age > 18
at timestamp '2024-11-10 12:03:21'
```

Implementation:

```rust
fn create_historical_snapshot(timestamp: LogicalTime) -> Snapshot {
    Snapshot {
        xid: TransactionId(0),
        xmin: TransactionId(0),
        xmax: find_max_xid_before(timestamp),
        active_xids: vec![],
        logical_time: timestamp,
    }
}
```

All transactions that committed before the specified time are visible.

### Time Range Queries

Query changes across a time range:

```
retrieve users
where age > 18
at timestamp '2024-11-10 12:03:21'
until timestamp '2024-11-15 14:30:00'
```

Returns all versions of matching tuples within the time range.

## Savepoints

### Creating Savepoints

```
begin transaction
  update users set age = 30 where id = 1
  savepoint s1
  update users set age = 31 where id = 1
  rollback to savepoint s1
commit
```

Implementation:

```rust
pub struct Savepoint {
    name: String,
    wal_position: u64,
    snapshot: Snapshot,
}

fn create_savepoint(tx: &mut Transaction, name: String) -> Savepoint {
    Savepoint {
        name,
        wal_position: get_wal_position(),
        snapshot: tx.snapshot.clone(),
    }
}

fn rollback_to_savepoint(tx: &mut Transaction, savepoint: &Savepoint) {
    // Restore snapshot
    tx.snapshot = savepoint.snapshot.clone();
    
    // Undo WAL entries after savepoint
    undo_wal_entries(savepoint.wal_position, get_wal_position());
}
```

## Distributed Transactions

### Two-Phase Commit

For transactions spanning multiple shards:

```rust
pub enum TransactionPhase {
    Prepare,
    Commit,
    Abort,
}

fn distributed_commit(participants: Vec<ShardId>) -> Result<()> {
    // Phase 1: Prepare
    for shard in &participants {
        let response = send_prepare(shard)?;
        if response != PrepareOk {
            // Abort all participants
            for s in &participants {
                send_abort(s)?;
            }
            return Err(TransactionAborted);
        }
    }
    
    // Phase 2: Commit
    for shard in &participants {
        send_commit(shard)?;
    }
    
    Ok(())
}
```

## Performance Considerations

### Hot Tuples

Frequently updated tuples create many versions:

- Use HOT (Heap Only Tuple) updates when possible
- Increase autovacuum frequency
- Consider partitioning

### Long-Running Transactions

Long transactions prevent garbage collection:

- Set statement timeouts
- Monitor transaction duration
- Alert on long transactions

### Snapshot Overhead

Snapshot creation has overhead:

- Read committed: Higher overhead (snapshot per statement)
- Snapshot isolation: Lower overhead (snapshot per transaction)

## Monitoring

### Transaction Metrics

```rust
pub struct TransactionMetrics {
    pub active_count: u64,
    pub committed_count: u64,
    pub aborted_count: u64,
    pub deadlocks_detected: u64,
    pub avg_duration: Duration,
}
```

### Dead Tuple Tracking

```rust
pub struct VacuumMetrics {
    pub dead_tuples: u64,
    pub live_tuples: u64,
    pub dead_tuple_ratio: f64,
    pub last_vacuum: Timestamp,
}
```

## Best Practices

### Keep Transactions Short

```
-- Good
begin
  update accounts set balance = balance - 100 where id = 1
commit

-- Bad
begin
  update accounts set balance = balance - 100 where id = 1
  sleep(60000)
commit
```

### Use Appropriate Isolation

```
-- Interactive queries: Read committed
begin transaction isolation level read committed
  retrieve users where active = true
commit

-- Financial operations: Serializable
begin transaction isolation level serializable
  retrieve balance from accounts where id = 1
  update accounts set balance = balance - 100 where id = 1
commit
```

### Index Foreign Keys

Foreign key columns should be indexed:

```
create table orders (
  id bigint primary key,
  user_id bigint references users(id)
)

create index orders_user_id_idx on orders (user_id)
```

This reduces lock contention on referenced rows.
