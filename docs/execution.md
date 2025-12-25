# Execution Model

## Overview

minsql uses a volcano-style execution model with iterator-based operators. Each operator implements a standard interface and produces tuples on demand.

## Execution Pipeline

### Query Flow

```
User Query
    ↓
Lexer → Tokens
    ↓
Parser → AST
    ↓
Semantic Analysis → Intent
    ↓
Logical Planner → Logical Plan
    ↓
Optimizer → Optimized Logical Plan
    ↓
Physical Planner → Physical Plan
    ↓
Execution Engine → Results
```

### Intent Extraction

The semantic analyzer converts the AST into a structured intent representation:

```rust
Intent::Retrieve {
    projection: vec![Column("name"), Column("email")],
    source: Table("users"),
    filter: Some(BinaryOp {
        op: GreaterThan,
        left: Column("age"),
        right: Literal(18)
    }),
    limit: Some(10)
}
```

This intent is what the optimizer operates on.

### Logical Planning

The logical planner creates an operator tree:

```
Limit(10)
  ↓
Filter(age > 18)
  ↓
Project(name, email)
  ↓
Scan(users)
```

### Physical Planning

The physical planner chooses implementations:

```
Limit(10)
  ↓
Filter(age > 18)
  ↓
Project(name, email)
  ↓
IndexScan(users, users_age_idx)
```

Here, the scan was converted to an index scan because an appropriate index exists.

## Operator Interface

### Standard Methods

Every operator implements:

```rust
trait Operator {
    fn open(&mut self) -> Result<()>;
    fn next(&mut self) -> Result<Option<Tuple>>;
    fn close(&mut self) -> Result<()>;
}
```

- `open()`: Initialize operator state
- `next()`: Produce next tuple, or None if exhausted
- `close()`: Release resources

### Tuple Flow

Operators are composed in a tree. Parent operators call `next()` on children to pull tuples.

## Core Operators

### Scan

Reads tuples from storage:

- `SeqScan`: Full table scan
- `IndexScan`: Index-based access
- `BitmapScan`: Bitmap index scan for multiple conditions

### Filter

Evaluates a predicate on each tuple:

```
Filter(age > 18 AND active = true)
```

Only tuples satisfying the predicate are passed to the parent.

### Project

Extracts specific columns:

```
Project(name, email)
```

Produces tuples containing only the projected columns.

### Join

Combines tuples from two inputs:

- `NestedLoopJoin`: Simple nested loop
- `HashJoin`: Hash-based join for equality predicates
- `MergeJoin`: Sort-merge join for sorted inputs

### Aggregate

Computes aggregates:

```
Aggregate(
    group_by: [department],
    aggregates: [Count(*), Avg(salary)]
)
```

Accumulates state and produces one tuple per group.

### Sort

Orders tuples:

```
Sort(order_by: [created_at DESC])
```

Buffers all input tuples, sorts them, then produces sorted output.

### Limit

Restricts output:

```
Limit(10)
```

Stops after producing N tuples.

### Insert

Writes tuples to storage:

```
Insert(table: users, values: [...])
```

### Update

Modifies existing tuples:

```
Update(
    table: users,
    set: [(age, age + 1)],
    filter: Some(active = true)
)
```

### Delete

Removes tuples:

```
Delete(table: users, filter: Some(age < 18))
```

## Expression Evaluation

### Expression Trees

Expressions are evaluated recursively:

```
BinaryOp(
    op: GreaterThan,
    left: Column("age"),
    right: Literal(18)
)
```

Evaluation:
1. Evaluate left operand → retrieve "age" column value
2. Evaluate right operand → constant 18
3. Apply operator → compare values

### Type System

Expressions are strongly typed. Type checking happens during semantic analysis:

```
age > 18       // Valid: integer > integer
age > "foo"    // Invalid: integer > text
age + "5"      // Invalid: integer + text
```

### Null Handling

SQL-style three-valued logic:

```
null = null    → null
null > 18      → null
null AND true  → null
null OR true   → true
```

## Query Sandboxing

### Resource Tracking

Each query runs with tracked resources:

```rust
struct QueryLimits {
    max_cpu_time: Duration,
    max_memory: usize,
    max_wall_time: Duration,
}
```

### Enforcement

- CPU time tracked via execution instrumentation
- Memory tracked via allocator hooks
- Wall time tracked via deadline checks

If limits are exceeded, execution is aborted:

```
Error: Query exceeded memory limit (100MB)
```

### Priority Scheduling

Queries are assigned priorities:

- `High`: Interactive queries
- `Normal`: Standard queries
- `Low`: Background jobs

Lower priority queries yield CPU to higher priority queries.

## Deterministic Execution

### Deterministic Mode

When enabled:

1. System clock access is forbidden
2. Random number generation is seeded
3. Operator scheduling is deterministic
4. Memory allocation is deterministic

### Logical Clock

Time is tracked via Hybrid Logical Clock (HLC):

```rust
struct LogicalTime {
    logical: u64,
    physical: u64,
}
```

In deterministic mode, `physical` is frozen and only `logical` advances.

### Benefits

- Reproducible debugging
- Consistent replication
- Predictable testing
- Audit compliance

## Transaction Integration

### MVCC Visibility

Operators respect transaction snapshots:

```rust
struct Snapshot {
    xid: TransactionId,
    created_at: LogicalTime,
    active_xids: Vec<TransactionId>,
}
```

Tuples are visible if:
1. Created by committed transaction
2. Created before snapshot time
3. Not created by active transaction

### Time-Travel

Operators can execute against historical snapshots:

```
retrieve users
where age > 18
at timestamp '2024-11-10 12:03:21'
```

The scan operator uses the specified snapshot instead of the current one.

## Performance Characteristics

### Scan Operators

- `SeqScan`: O(n) where n is table size
- `IndexScan`: O(log n + k) where k is result size
- `BitmapScan`: O(m log n + k) where m is number of conditions

### Join Operators

- `NestedLoopJoin`: O(n × m)
- `HashJoin`: O(n + m) average, O(n × m) worst case
- `MergeJoin`: O(n + m) for sorted inputs

### Aggregate

- `Aggregate`: O(n) with hash table
- `SortAggregate`: O(n log n) with sorting

### Sort

- `Sort`: O(n log n)

## Error Handling

### Error Types

- `ExecutionError`: Runtime execution failure
- `ResourceExceeded`: Query limit violated
- `DataCorruption`: Storage integrity issue
- `Deadlock`: Transaction deadlock detected

### Rollback

On error:
1. Abort execution
2. Release resources
3. Rollback transaction if active
4. Return error to client

## Monitoring

### Metrics

Each operator exposes metrics:

```rust
struct OperatorMetrics {
    tuples_produced: u64,
    cpu_time: Duration,
    wall_time: Duration,
    memory_used: usize,
}
```

### Profiling

Query plans can be explained:

```
explain retrieve users where age > 18
```

Output:
```
Limit(10) [cost: 5.2, rows: 10]
  Filter(age > 18) [cost: 104.5, rows: 500]
    SeqScan(users) [cost: 100.0, rows: 1000]
```

### Tracing

Execution is traced for debugging:

```
[TRACE] Scan::open(table=users)
[TRACE] Filter::open()
[TRACE] Limit::open()
[TRACE] Scan::next() → Some(Tuple(id=1, name="Alice", age=30))
[TRACE] Filter::next() → Some(Tuple(id=1, name="Alice", age=30))
[TRACE] Limit::next() → Some(Tuple(id=1, name="Alice", age=30))
```

## Parallelism

### Parallel Scans

Large tables can be scanned in parallel:

```
ParallelSeqScan(workers: 4)
```

Each worker scans a partition of the table.

### Parallel Aggregates

Aggregates can be computed in parallel:

```
FinalizeAggregate
  ParallelAggregate(workers: 4)
```

Workers compute partial aggregates, then a final operator combines them.

### Coordination

Parallel operators coordinate via channels:

```rust
struct ParallelContext {
    workers: Vec<Worker>,
    coordinator: Coordinator,
}
```

## Future Optimizations

### Vectorization

Batch tuple processing for better CPU utilization.

### Code Generation

JIT-compile hot operators for reduced interpretation overhead.

### Adaptive Execution

Re-optimize plans based on runtime statistics.

### Predicate Pushdown

Push filters closer to scans across operator boundaries.
