# minsql System Architecture

## High-Level Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Client Applications                       │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ Protocol Layer
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                     minsql Server                            │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Security Layer                           │  │
│  │  • Authentication  • Authorization  • Encryption      │  │
│  │  • Audit Logging   • Row-Level Security              │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Query Processing                         │  │
│  │  ┌─────────┐  ┌──────────┐  ┌────────────────────┐  │  │
│  │  │ Lexer   │→ │ Parser   │→ │ Semantic Analysis  │  │  │
│  │  └─────────┘  └──────────┘  └────────────────────┘  │  │
│  │        ↓               ↓               ↓              │  │
│  │  ┌──────────────────────────────────────────────┐   │  │
│  │  │         Intent Extraction                     │   │  │
│  │  └──────────────────────────────────────────────┘   │  │
│  │        ↓               ↓               ↓              │  │
│  │  ┌─────────┐  ┌──────────┐  ┌────────────────────┐  │  │
│  │  │Logical  │→ │Optimizer │→ │Physical Planner    │  │  │
│  │  │Planner  │  │          │  │                    │  │  │
│  │  └─────────┘  └──────────┘  └────────────────────┘  │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Execution Engine                         │  │
│  │  • Operators  • Expression Eval  • Sandboxing        │  │
│  │  • Vectorized Execution  • Parallel Processing       │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Analytics Layer                          │  │
│  │  • Columnar Storage  • Materialized Views            │  │
│  │  • Query Cache       • Statistics                    │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │          Transaction Management                       │  │
│  │  • MVCC  • Snapshots  • Time-Travel  • Isolation     │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Storage Engine (C/C++)                   │  │
│  │  ┌────────┐  ┌────────┐  ┌──────┐  ┌───────────┐   │  │
│  │  │ Pages  │  │ Buffer │  │ WAL  │  │ Indexes   │   │  │
│  │  │ (C)    │  │ Pool   │  │ (C)  │  │ (C++)     │   │  │
│  │  └────────┘  └────────┘  └──────┘  └───────────┘   │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │          Distributed Coordination                     │  │
│  │  • Raft Consensus  • Sharding  • Replication         │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Monitoring & Observability               │  │
│  │  • Health Checks  • Metrics  • Alerts  • Tracing     │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    File System / Disk                        │
└─────────────────────────────────────────────────────────────┘
```

## Component Architecture

### 1. Protocol Layer

**Location**: `engine/protocol/`

**Components:**
- `handshake.rs` - Protocol negotiation and version checking
- `framing.rs` - Message framing and serialization
- `auth.rs` - Authentication and credential management
- `server.rs` - TCP server and connection handling

**Responsibilities:**
- Client connection acceptance
- Protocol handshake
- Message framing
- Authentication
- Connection pooling

**Data Flow:**
```
Client → TCP → Handshake → Auth → Framed Messages → Query Processing
```

### 2. Security Layer

**Location**: `engine/security/`

**Components:**
- `encryption.rs` - Data encryption at rest and in transit
- `row_level_security.rs` - Row-level access control
- `rbac.rs` - Role-based access control
- `audit_log.rs` - Comprehensive audit trail

**Features:**
- AES-256 encryption
- Policy-based row filtering
- Fine-grained permissions
- Complete audit logging

**Security Model:**
```
User → Authentication → Role Assignment → Permission Check → RLS Policies → Data Access
```

### 3. Query Processing Pipeline

**Location**: `engine/language/`, `engine/planner/`

**Stages:**

#### 3.1 Lexical Analysis
```
Input: "retrieve users where age > 18"
Output: [Token::Retrieve, Token::Identifier("users"), ...]
```

#### 3.2 Parsing
```
Tokens → AST (Abstract Syntax Tree)

AST::RetrieveStatement {
  projection: [...],
  from: TableReference("users"),
  filter: BinaryOp(Column("age"), GreaterThan, Literal(18))
}
```

#### 3.3 Semantic Analysis
```
AST → Intent (Semantic Representation)

Intent::Retrieve {
  columns: [All],
  source: SourceIntent { primary: "users" },
  filter: FilterIntent::Comparison { ... }
}
```

#### 3.4 Logical Planning
```
Intent → Logical Plan (Operator Tree)

Limit(10)
  ↓
Filter(age > 18)
  ↓
Project(name, email)
  ↓
Scan(users)
```

#### 3.5 Optimization
```
Logical Plan → Optimized Logical Plan

Optimizations:
• Predicate pushdown
• Projection pushdown
• Constant folding
• Join reordering
```

#### 3.6 Physical Planning
```
Optimized Logical Plan → Physical Plan

Limit(10)
  ↓
Filter(age > 18)
  ↓
Project(name, email)
  ↓
IndexScan(users, users_age_idx)  ← Chose index scan
```

#### 3.7 Cost Estimation
```
Physical Plan → Cost Model → Plan Selection

IndexScan Cost: 50.2
SeqScan Cost: 104.5
→ Choose IndexScan
```

### 4. Execution Engine

**Location**: `engine/execution/`

**Architecture:**
```
ExecutionEngine
  ↓
Operators (Volcano-style iterators)
  ↓
Tuple Processing
  ↓
Storage Access
```

**Operator Types:**
- **Scan**: SeqScan, IndexScan, BitmapScan
- **Join**: HashJoin, NestedLoopJoin, MergeJoin
- **Aggregate**: HashAggregate, SortAggregate
- **Sort**: QuickSort, MergeSort
- **Limit**: Top-N extraction
- **Filter**: Predicate evaluation
- **Project**: Column extraction

**Execution Modes:**

#### Standard Execution
```rust
fn next(&mut self) -> Result<Option<Tuple>> {
    // Pull one tuple at a time
}
```

#### Vectorized Execution
```rust
fn next_batch(&mut self) -> Result<VectorBatch> {
    // Pull 1024 tuples at a time
}
```

### 5. Analytics Layer

**Location**: `engine/analytics/`

**Components:**

#### 5.1 Columnar Storage
```
Row-oriented (OLTP):
[id=1, name="Alice", age=30]
[id=2, name="Bob", age=25]

Column-oriented (OLAP):
ids:   [1, 2, ...]
names: ["Alice", "Bob", ...]
ages:  [30, 25, ...]
```

**Benefits:**
- Better compression (10-100x)
- Faster scans (only read needed columns)
- SIMD-friendly

#### 5.2 Materialized Views
```
Create: retrieve department, count(*) from employees group by department
Store: Pre-computed results
Refresh: Manual or automatic
Query: Instant retrieval
```

#### 5.3 Query Cache
```
Key: Query string + parameters
Value: Result set + metadata
Eviction: LRU
Invalidation: On table writes
```

### 6. Transaction Management

**Location**: `engine/transactions/`

**MVCC Implementation:**

```
Tuple Structure:
┌──────────┬──────────┬──────────┬──────┐
│ t_xmin   │ t_xmax   │ t_flags  │ data │
└──────────┴──────────┴──────────┴──────┘

t_xmin: Transaction that created this version
t_xmax: Transaction that deleted this version (0 if active)
```

**Snapshot Isolation:**
```rust
struct Snapshot {
    xid: TransactionId,
    xmin: TransactionId,
    xmax: TransactionId,
    active_xids: Vec<TransactionId>,
    logical_time: LogicalTime,
}
```

**Visibility Rules:**
```rust
fn is_visible(tuple: &Tuple, snapshot: &Snapshot) -> bool {
    // Tuple created by future transaction?
    if tuple.xmin >= snapshot.xmax { return false; }
    
    // Tuple created by active transaction (not us)?
    if snapshot.active_xids.contains(&tuple.xmin) 
       && tuple.xmin != snapshot.xid { return false; }
    
    // Tuple deleted?
    if tuple.xmax != 0 && tuple.xmax < snapshot.xmax { return false; }
    
    true
}
```

### 7. Deterministic Execution

**Location**: `engine/determinism/`

**Components:**

#### 7.1 Hybrid Logical Clock
```
LogicalTime {
    logical: u64,    // Monotonically increasing counter
    physical: u64,   // Frozen in deterministic mode
}
```

#### 7.2 Deterministic Scheduler
```
Tasks ordered by TaskId (not OS scheduling)

Ready Queue: BTreeMap<TaskId, Task>
→ Always execute lowest TaskId first
→ Deterministic scheduling order
```

#### 7.3 Replay Engine
```
WAL Entry → Logical Time → Deterministic Replay

Same input + same logical time = same output + same timing
```

### 8. Storage Engine

**Location**: `storage/` (C/C++)

**Architecture:**

#### 8.1 Page Manager (C)
```c
Page Layout (8KB):
┌────────────────────────┐
│  Header (24 bytes)     │
├────────────────────────┤
│  Line Pointers (var)   │
├────────────────────────┤
│  Free Space            │
├────────────────────────┤
│  Tuples (var)          │
└────────────────────────┘
```

#### 8.2 Buffer Pool (C)
```c
struct BufferPool {
    Page* pages;           // Cached pages
    size_t capacity;       // Max pages in cache
    HashTable* page_table; // Page ID → Buffer slot
    LRUCache* lru;         // Eviction policy
}
```

#### 8.3 WAL (C)
```c
WAL Entry:
┌────────┬─────────┬──────────┬──────┬────────┬──────┐
│  LSN   │  XID    │ LogTime  │ Type │ Length │ Data │
└────────┴─────────┴──────────┴──────┴────────┴──────┘
```

**Write-Ahead Logging:**
```
1. Write to WAL buffer
2. Flush WAL to disk (fsync)
3. Apply to data pages
4. Checkpoint periodically
```

#### 8.4 Indexes (C++)

**B-Tree:**
```cpp
template<typename Key, typename Value>
class BTree {
    Node* root;
    static const size_t ORDER = 128;
    
    // Operations: O(log n)
    void insert(Key, Value);
    bool search(Key, Value&);
    void remove(Key);
}
```

**Hash Index:**
```cpp
template<typename Key, typename Value>
class HashIndex {
    Bucket* buckets;
    size_t num_buckets;
    
    // Operations: O(1) average
    void insert(Key, Value);
    bool search(Key, Value&);
}
```

**Bloom Filter:**
```cpp
class BloomFilter {
    vector<uint8_t> bits;
    size_t num_hashes;
    
    // Quick membership test
    bool might_contain(Key);  // False positives possible
}
```

### 9. Distributed Coordination

**Location**: `engine/sharding/`, `engine/replication/`

**Architecture:**

#### 9.1 Sharding
```
Keyspace → Hash Function → Shard Assignment

Example:
hash("user_123") % 16 = 5 → Shard 5

Co-located tables:
users (shard by user_id)
orders (shard by user_id)
→ Join executes on single shard
```

#### 9.2 Raft Consensus
```
Leader Election:
┌───────┐  ┌───────┐  ┌───────┐
│Node 1 │  │Node 2 │  │Node 3 │
│Leader │→ │Follwr │  │Follwr │
└───────┘  └───────┘  └───────┘

Write Replication:
Client → Leader → Log Entry
Leader → Followers (parallel)
Wait for Quorum (2/3)
Commit → Apply to State Machine
```

### 10. Monitoring System

**Location**: `engine/monitoring/`, `engine/telemetry/`

**Components:**

#### 10.1 Health Checks
```
Periodic Checks:
• CPU usage < 90%
• Memory usage < 90%
• Disk space > 10%
• Raft leader present
• Storage engine operational

Status: Healthy | Degraded | Unhealthy
```

#### 10.2 Performance Monitoring
```
Tracked Metrics:
• Query execution time (P50, P95, P99)
• Queries per second
• Rows scanned
• Cache hit rate
• Disk I/O
• Network I/O
```

#### 10.3 Alert System
```
Alert Conditions:
• CPU > 90% → Warning
• Memory > 95% → Critical
• Disk < 10% → Critical
• Replication lag > 10s → Warning
• Query timeout → Info

Actions:
• Log alert
• Send notification
• Auto-remediation (future)
```

## Data Flow Examples

### Example 1: Simple Query

```
Query: retrieve users where age > 18

1. Protocol Layer
   Client → TCP → Parse request

2. Security Layer
   Authenticate → Check permissions → Apply RLS

3. Query Processing
   Lex → Parse → Semantic → Intent
   
4. Planning
   Logical Plan:
     Filter(age > 18)
       Scan(users)
   
   Optimization:
     • Check for index on age
     • Estimate selectivity
   
   Physical Plan:
     Filter(age > 18)
       IndexScan(users, users_age_idx)

5. Execution
   IndexScan.open()
   loop:
     tuple = IndexScan.next()
     if Filter.evaluate(tuple):
       yield tuple

6. Storage Access
   IndexScan → Buffer Pool → Check cache
   If miss: Disk → Read page → Cache → Return

7. Result
   Tuples → Serialize → Send to client
```

### Example 2: Deterministic Transaction

```
Query: begin deterministic transaction
       retrieve users order by id
       commit

1. Transaction Start
   Create snapshot with frozen logical time
   logical_time = LogicalTime { logical: 1000, physical: FROZEN }

2. Query Execution
   Use deterministic scheduler (BTreeMap-based)
   All operations use logical_time (no system clock)

3. Storage Access
   Read with snapshot visibility rules
   All timing based on logical clock

4. Commit
   Write to WAL with logical_time
   Replay will be deterministic

Result: Same query → Same results → Same timing
```

### Example 3: Distributed Query

```
Query: retrieve users where department = 'Engineering'

1. Routing
   No shard key in filter → Fanout to all shards

2. Shard Execution (parallel)
   Shard 1: Find matching users
   Shard 2: Find matching users
   ...
   Shard 16: Find matching users

3. Result Aggregation
   Coordinator merges results from all shards

4. Return
   Merged results → Client
```

## Performance Characteristics

### Throughput
- **OLTP Writes**: 50K+/sec per shard
- **OLTP Reads**: 100K+/sec per shard
- **OLAP Scans**: 1GB/sec (columnar)
- **Aggregations**: 10M rows/sec (vectorized)

### Latency
- **Simple queries**: P99 < 10ms
- **Complex queries**: P99 < 100ms
- **Cross-shard queries**: P99 < 500ms

### Scalability
- **Horizontal**: Linear scaling up to 100+ nodes
- **Vertical**: Efficient use of multi-core systems
- **Storage**: Petabyte-scale capability

## Deployment Models

### Single Node
```
┌──────────────┐
│  minsql Node │
│  • All layers│
│  • All data  │
└──────────────┘
```

### Replicated Cluster
```
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Node 1       │  │ Node 2       │  │ Node 3       │
│ (Leader)     │→ │ (Follower)   │  │ (Follower)   │
└──────────────┘  └──────────────┘  └──────────────┘
     Raft Consensus (Strong Consistency)
```

### Sharded Cluster
```
┌─────────────────────────────────────────────┐
│           Coordinator / Router              │
└─────────────────────────────────────────────┘
         ↓              ↓              ↓
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ Shard 0-5    │ │ Shard 6-10   │ │ Shard 11-15  │
│ (3 replicas) │ │ (3 replicas) │ │ (3 replicas) │
└──────────────┘ └──────────────┘ └──────────────┘
```

## Future Architecture Enhancements

### Planned Improvements
- Vectorized expression evaluation (SIMD)
- JIT compilation for hot paths
- Adaptive query execution
- Automatic index tuning
- Machine learning for cardinality estimation
- Distributed transactions with 2PC
- Streaming replication
- Change data capture (CDC)

---

This architecture enables minsql to be:
- **Fast**: Optimized at every layer
- **Reliable**: Strong consistency guarantees
- **Scalable**: Horizontal and vertical scaling
- **Observable**: Comprehensive monitoring
- **Secure**: Multiple security layers
- **Maintainable**: Clean separation of concerns
