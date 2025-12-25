# Query Optimizer

## Overview

minsql's optimizer operates on semantic intent rather than raw syntax. This allows for more aggressive optimization and clearer reasoning about query semantics.

## Optimization Pipeline

### Stages

```
Intent
  ↓
Logical Plan Generation
  ↓
Logical Optimization
  ↓
Physical Planning
  ↓
Physical Optimization
  ↓
Cost-Based Selection
  ↓
Final Plan
```

## Logical Optimization

### Rule-Based Transformations

The optimizer applies transformation rules to improve the logical plan.

#### Predicate Pushdown

Move filters closer to data sources:

```
Before:
  Filter(age > 18)
    Join(users, orders)

After:
  Join(
    Filter(age > 18)
      Scan(users),
    Scan(orders)
  )
```

This reduces the size of the join input.

#### Projection Pushdown

Only retrieve needed columns:

```
Before:
  Project(name)
    Scan(users)  // reads all columns

After:
  Scan(users, columns: [name])  // reads only name
```

#### Join Reordering

Reorder joins to minimize intermediate result size:

```
Before:
  Join(
    Join(large_table, huge_table),
    small_table
  )

After:
  Join(
    Join(small_table, large_table),
    huge_table
  )
```

#### Constant Folding

Evaluate constant expressions at compile time:

```
Before:
  Filter(age > 10 + 8)

After:
  Filter(age > 18)
```

#### Expression Simplification

Simplify logical expressions:

```
Before:
  Filter(age > 18 AND age > 18)

After:
  Filter(age > 18)
```

```
Before:
  Filter(active = true AND true)

After:
  Filter(active = true)
```

#### Subquery Decorrelation

Convert correlated subqueries to joins:

```
Before:
  Filter(
    id IN (
      retrieve user_id from orders
      where orders.user_id = users.id
    )
  )

After:
  SemiJoin(users, orders, on: users.id = orders.user_id)
```

### Equivalence Rules

The optimizer recognizes logically equivalent expressions:

```
age > 18 AND active = true
≡
active = true AND age > 18
```

```
NOT (age > 18 OR active = true)
≡
age <= 18 AND active = false
```

## Physical Planning

### Operator Selection

For each logical operator, choose a physical implementation.

#### Scan Selection

Choose scan strategy based on available access paths:

```
Logical: Scan(users)

Physical options:
  1. SeqScan(users)
  2. IndexScan(users, users_id_idx)
  3. BitmapScan(users, users_age_idx)
```

Selection criteria:
- Filter predicates (can index be used?)
- Column requirements (covering index?)
- Result size estimate
- Index selectivity

#### Join Selection

Choose join algorithm:

```
Logical: Join(users, orders)

Physical options:
  1. NestedLoopJoin
  2. HashJoin
  3. MergeJoin
```

Selection criteria:
- Join predicate type (equality vs inequality)
- Input size estimates
- Memory availability
- Sorted input availability

#### Aggregate Selection

Choose aggregation strategy:

```
Logical: Aggregate(group_by: [dept], agg: [count(*)])

Physical options:
  1. HashAggregate
  2. SortAggregate
```

Selection criteria:
- Number of groups
- Memory availability
- Sort requirement for downstream operators

## Cost Model

### Cost Estimation

Each operator has an estimated cost:

```rust
struct Cost {
    cpu: f64,
    io: f64,
    memory: f64,
    network: f64,
}
```

Total cost = `cpu + io + memory + network` (weighted).

### Cardinality Estimation

Estimating result sizes is critical for cost calculation.

#### Base Table Statistics

```rust
struct TableStats {
    row_count: u64,
    page_count: u64,
    avg_row_size: usize,
}
```

#### Column Statistics

```rust
struct ColumnStats {
    distinct_count: u64,
    null_fraction: f64,
    min_value: Value,
    max_value: Value,
    histogram: Option<Histogram>,
}
```

#### Selectivity Estimation

Filter selectivity determines fraction of rows passing:

```
age > 18

If age range is [0, 100]:
  selectivity ≈ (100 - 18) / 100 = 0.82
```

```
active = true

If 70% of rows have active = true:
  selectivity = 0.70
```

#### Join Cardinality

```
Join(users, orders)

If users has 1000 rows and orders has 5000 rows:
  
  Without predicate:
    cardinality = 1000 × 5000 = 5,000,000
  
  With users.id = orders.user_id (foreign key):
    cardinality ≈ 5000  (number of orders)
```

### Operator Costs

#### Sequential Scan

```
cost = page_count × PAGE_SCAN_COST
     + row_count × CPU_TUPLE_COST
```

#### Index Scan

```
cost = index_depth × PAGE_SCAN_COST
     + selectivity × row_count × (INDEX_TUPLE_COST + CPU_TUPLE_COST)
```

#### Nested Loop Join

```
cost = outer_cost
     + outer_rows × (inner_cost + inner_rows × CPU_JOIN_COST)
```

#### Hash Join

```
cost = outer_cost + inner_cost
     + (outer_rows + inner_rows) × CPU_HASH_COST
     + outer_rows × inner_rows × CPU_JOIN_COST
```

#### Sort

```
cost = row_count × log2(row_count) × CPU_SORT_COST
```

### Cost Parameters

Default cost weights:

```rust
const PAGE_SCAN_COST: f64 = 1.0;
const CPU_TUPLE_COST: f64 = 0.01;
const CPU_OPERATOR_COST: f64 = 0.0025;
const CPU_HASH_COST: f64 = 0.005;
const CPU_JOIN_COST: f64 = 0.02;
const CPU_SORT_COST: f64 = 0.03;
```

These can be tuned based on workload characteristics.

## Sharding Awareness

### Locality-Based Optimization

The optimizer understands data distribution across shards.

#### Local vs Remote

```
retrieve users where id = 100

If key 100 maps to local shard:
  cost = LOCAL_SCAN_COST

If key 100 maps to remote shard:
  cost = NETWORK_COST + REMOTE_SCAN_COST
```

#### Cross-Shard Joins

```
retrieve users.name, orders.total
from users
join orders on users.id = orders.user_id

If users and orders are co-located by user_id:
  Execute join locally per shard
  cost = LOCAL_JOIN_COST

If not co-located:
  Shuffle data across shards
  cost = SHUFFLE_COST + REMOTE_JOIN_COST
```

#### Aggregation Pushdown

```
retrieve count(*) from users where age > 18

Strategy 1: Fetch all data, aggregate locally
  cost = NETWORK_TRANSFER_COST(all_data) + AGG_COST

Strategy 2: Aggregate on each shard, combine results
  cost = NUM_SHARDS × REMOTE_AGG_COST + COMBINE_COST
```

The optimizer chooses based on estimated data volume.

## Statistics Collection

### Automatic Statistics

Statistics are collected automatically:

```
ANALYZE users;
```

This:
1. Samples table rows
2. Computes column statistics
3. Builds histograms
4. Stores in system tables

### Statistics Staleness

Statistics have timestamps. If statistics are stale, the optimizer may re-analyze:

```rust
if last_analyze_time < last_modification_time - STALENESS_THRESHOLD {
    trigger_auto_analyze(table);
}
```

## Plan Caching

### Prepared Statements

Plans for prepared statements are cached:

```
prepare user_lookup as
  retrieve users where id = $1

execute user_lookup(100)
execute user_lookup(200)
```

The plan is generated once and reused.

### Plan Invalidation

Cached plans are invalidated when:
- Schema changes
- Statistics are updated
- Indexes are added/removed

## Optimizer Hints

### Intent-Based Hints

Hints guide the optimizer without dictating specific plans:

```
retrieve users
where age > 18
with hint prefer_index
```

Available hints:
- `prefer_index`: Prefer index scans
- `prefer_seq`: Prefer sequential scans
- `prefer_hash_join`: Prefer hash joins
- `prefer_merge_join`: Prefer merge joins

### Why Intent-Based?

Traditional SQL hints (`USE INDEX`, `FORCE INDEX`) are brittle:
- Break when schema changes
- Bypass cost model entirely
- Hard to maintain

Intent-based hints guide rather than dictate, allowing the optimizer to adapt.

## Deterministic Planning

### Plan Stability

In deterministic mode, the optimizer produces identical plans for identical inputs:

- Statistics are frozen
- Random tie-breaking is disabled
- Cost model is deterministic

This ensures:
- Reproducible performance
- Consistent query behavior
- Reliable testing

## Debugging

### EXPLAIN

View the chosen plan:

```
explain retrieve users where age > 18
```

Output:
```
Filter(age > 18) [cost: 104.5, rows: 500]
  SeqScan(users) [cost: 100.0, rows: 1000]

Estimated cost: 104.5
Estimated rows: 500
```

### EXPLAIN ANALYZE

Execute and show actual statistics:

```
explain analyze retrieve users where age > 18
```

Output:
```
Filter(age > 18)
  Estimated: 500 rows, cost 104.5
  Actual: 487 rows, time 12.3ms
  
  SeqScan(users)
    Estimated: 1000 rows, cost 100.0
    Actual: 1000 rows, time 8.7ms
```

### Optimizer Trace

Enable detailed optimizer logging:

```
set optimizer_trace = true;
retrieve users where age > 18;
```

This shows:
- Rules applied
- Cost calculations
- Plan alternatives considered
- Final selection rationale

## Limitations

### Current Constraints

- No multi-column statistics (correlations ignored)
- Histogram resolution is fixed
- Join ordering is heuristic for >5 tables
- No adaptive re-optimization during execution

### Future Work

- Machine learning for cardinality estimation
- Adaptive execution with mid-query replanning
- Multi-column statistics
- Query feedback loop for statistics

## Best Practices

### Schema Design

Use appropriate indexes:
```
create index users_age_idx on users (age)
where active = true
```

Partial indexes reduce overhead.

### Statistics Maintenance

Keep statistics current:
```
ANALYZE users;
```

Run after bulk modifications.

### Query Structure

Write queries that expose optimization opportunities:

Good:
```
retrieve users
where age > 18 and active = true
```

Less good:
```
retrieve users
where age + 0 > 18
```

The second form prevents index usage.

### Monitoring

Watch for plan regressions:
- Track query execution time
- Monitor plan cache hit rate
- Alert on plan changes
