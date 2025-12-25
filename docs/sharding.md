# Sharding

## Overview

minsql implements first-class sharding where the query planner understands data distribution and optimizes for locality. Sharding is transparent to applications.

## Sharding Model

### Keyspace Partitioning

Data is partitioned based on a shard key:

```rust
pub struct ShardKey {
    pub columns: Vec<String>,
}

pub struct Keyspace {
    pub ranges: Vec<KeyRange>,
}

pub struct KeyRange {
    pub start: Vec<u8>,
    pub end: Vec<u8>,
    pub shard_id: ShardId,
}
```

### Hash-Based Sharding

Data is distributed using consistent hashing:

```rust
fn compute_shard(key: &[u8], num_shards: usize) -> ShardId {
    let hash = blake3::hash(key);
    let hash_value = u64::from_le_bytes(hash.as_bytes()[0..8].try_into().unwrap());
    ShardId((hash_value % num_shards as u64) as u32)
}
```

### Range-Based Sharding

Data is partitioned by key ranges:

```
Shard 0: users.id [0, 1000)
Shard 1: users.id [1000, 2000)
Shard 2: users.id [2000, 3000)
...
```

## Shard Routing

### Routing Layer

The router determines which shard(s) to query:

```rust
pub struct Router {
    keyspace: Keyspace,
    shard_map: HashMap<ShardId, ShardInfo>,
}

impl Router {
    pub fn route(&self, query: &Query) -> Vec<ShardId> {
        match extract_shard_key(query) {
            Some(key) => {
                // Single-shard query
                vec![self.keyspace.lookup(key)]
            }
            None => {
                // Multi-shard query (fanout)
                self.shard_map.keys().copied().collect()
            }
        }
    }
}
```

### Key Extraction

Extract shard key from query:

```rust
fn extract_shard_key(query: &Query) -> Option<Vec<u8>> {
    match &query.intent {
        Intent::Retrieve { filter, .. } => {
            // Look for equality predicate on shard key
            if let Some(Expr::Eq { column, value }) = filter {
                if column == "id" {
                    return Some(serialize_value(value));
                }
            }
        }
        _ => {}
    }
    None
}
```

## Query Execution

### Single-Shard Queries

Queries that target a single shard execute locally:

```
retrieve users where id = 100

If id=100 maps to shard 2:
  Execute only on shard 2
```

No cross-shard coordination needed.

### Multi-Shard Queries

Queries that span shards require coordination:

```
retrieve users where age > 18

This requires:
  1. Send query to all shards
  2. Each shard executes locally
  3. Coordinator merges results
```

### Joins

#### Co-Located Joins

If tables are sharded on the same key, joins execute locally:

```
create table users (id bigint primary key, ...) shard by id
create table orders (id bigint, user_id bigint, ...) shard by user_id

retrieve users.name, orders.total
from users
join orders on users.id = orders.user_id
where users.id = 100

Since both tables are sharded by user_id (or id):
  Execute join on single shard
```

#### Non-Co-Located Joins

If tables are sharded differently, data must be shuffled:

```
retrieve users.name, products.name
from users
join orders on users.id = orders.user_id
join products on orders.product_id = products.id

Strategy:
  1. Fetch orders from all shards
  2. Shuffle orders by product_id
  3. Execute join with products locally per shard
  4. Merge results
```

### Aggregation

#### Distributed Aggregation

Aggregates are computed in two phases:

```
retrieve department, count(*), avg(salary)
from employees
where hire_date > '2020-01-01'
group by department

Phase 1 (per shard):
  Compute partial aggregates:
    {Engineering: count=50, sum=5000000}
    {Sales: count=30, sum=2400000}

Phase 2 (coordinator):
  Merge partial aggregates:
    {Engineering: count=150, avg=100000}
    {Sales: count=90, avg=80000}
```

#### Aggregation Pushdown

Optimizer pushes aggregation to shards when possible:

```
retrieve count(*) from users

Instead of:
  1. Fetch all users from all shards
  2. Count at coordinator

Do:
  1. Count on each shard
  2. Sum counts at coordinator
```

### Sorting

#### Distributed Sort

Sorting across shards requires merge:

```
retrieve users
order by created_at desc
limit 10

Strategy:
  1. Each shard returns top 10 by created_at
  2. Coordinator merges K sorted lists
  3. Return top 10 overall
```

## Shard Placement

### Placement Strategy

Shards are placed on nodes to balance load:

```rust
pub struct PlacementStrategy {
    pub shard_to_node: HashMap<ShardId, NodeId>,
}

impl PlacementStrategy {
    pub fn assign_shard(&mut self, shard_id: ShardId) -> NodeId {
        // Find node with fewest shards
        let node = self.find_least_loaded_node();
        self.shard_to_node.insert(shard_id, node);
        node
    }
}
```

### Replication

Each shard is replicated across multiple nodes:

```rust
pub struct ShardReplica {
    pub shard_id: ShardId,
    pub replicas: Vec<NodeId>,
    pub primary: NodeId,
}
```

Writes go to primary, reads can go to any replica.

## Rebalancing

### Shard Splitting

When a shard grows too large, it can be split:

```rust
fn split_shard(shard_id: ShardId) -> (ShardId, ShardId) {
    let old_range = get_shard_range(shard_id);
    let midpoint = (old_range.start + old_range.end) / 2;
    
    let shard_a = create_shard(old_range.start, midpoint);
    let shard_b = create_shard(midpoint, old_range.end);
    
    // Copy data
    for row in read_shard(shard_id) {
        if row.key < midpoint {
            write_shard(shard_a, row);
        } else {
            write_shard(shard_b, row);
        }
    }
    
    // Update metadata
    remove_shard(shard_id);
    
    (shard_a, shard_b)
}
```

### Shard Migration

Shards can be moved between nodes:

```rust
fn migrate_shard(shard_id: ShardId, from_node: NodeId, to_node: NodeId) -> Result<()> {
    // 1. Start copying data
    let snapshot = create_snapshot(shard_id);
    copy_snapshot(from_node, to_node, snapshot)?;
    
    // 2. Catch up with incremental changes
    let changes = get_changes_since(snapshot);
    apply_changes(to_node, changes)?;
    
    // 3. Pause writes
    pause_writes(shard_id)?;
    
    // 4. Final sync
    let final_changes = get_changes_since(snapshot);
    apply_changes(to_node, final_changes)?;
    
    // 5. Update routing
    update_routing(shard_id, to_node)?;
    
    // 6. Resume writes
    resume_writes(shard_id)?;
    
    Ok(())
}
```

## Schema Design

### Choosing a Shard Key

Good shard keys:

- High cardinality (many distinct values)
- Uniform distribution (no hot shards)
- Used in common queries (enables single-shard execution)

```
-- Good: user_id
create table orders (
  id bigint,
  user_id bigint,
  total decimal
) shard by user_id

-- Bad: status (only a few values)
create table orders (
  id bigint,
  status varchar(20)
) shard by status
```

### Co-Location

Related tables should be sharded on the same key:

```
create table users (
  id bigint primary key
) shard by id

create table orders (
  id bigint,
  user_id bigint
) shard by user_id

create table order_items (
  id bigint,
  order_id bigint,
  user_id bigint
) shard by user_id
```

This enables local joins without shuffling.

## Transaction Coordination

### Single-Shard Transactions

Transactions that touch only one shard execute locally:

```
begin transaction
  update users set balance = balance - 100 where id = 1
  insert into transactions (user_id, amount) values (1, -100)
commit

If both operations target the same shard:
  Execute as local transaction
```

### Multi-Shard Transactions

Transactions spanning shards use 2PC:

```
begin transaction
  update users set balance = balance - 100 where id = 1
  update users set balance = balance + 100 where id = 2
commit

If id=1 and id=2 are on different shards:
  Use two-phase commit
```

## Monitoring

### Shard Metrics

```rust
pub struct ShardMetrics {
    pub size_bytes: u64,
    pub row_count: u64,
    pub query_rate: f64,
    pub hot_key_count: u64,
}
```

### Load Balancing

Track load per shard:

```rust
pub struct LoadMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub network_io: f64,
    pub disk_io: f64,
}
```

Trigger rebalancing if load is skewed.

## Failure Handling

### Shard Failure

If a shard becomes unavailable:

```rust
fn handle_shard_failure(shard_id: ShardId) -> Result<()> {
    // 1. Mark shard as unavailable
    mark_shard_unavailable(shard_id)?;
    
    // 2. Promote a replica to primary
    let new_primary = select_replica(shard_id)?;
    promote_to_primary(new_primary)?;
    
    // 3. Update routing
    update_routing(shard_id, new_primary)?;
    
    Ok(())
}
```

### Network Partition

During network partition, use quorum reads/writes:

```rust
fn quorum_write(shard_id: ShardId, data: &[u8]) -> Result<()> {
    let replicas = get_replicas(shard_id);
    let mut successes = 0;
    
    for replica in replicas {
        if write_to_replica(replica, data).is_ok() {
            successes += 1;
        }
    }
    
    if successes >= quorum_size() {
        Ok(())
    } else {
        Err(QuorumNotReached)
    }
}
```

## Query Planning

### Shard-Aware Optimization

The optimizer considers shard boundaries:

```
retrieve users
where id in (1, 2, 3, 100, 200, 300)

If keys 1,2,3 are on shard A and 100,200,300 are on shard B:
  Plan: Parallel query to shards A and B
  Cost: 2 × SHARD_QUERY_COST
```

### Locality Cost Model

Remote shard access has higher cost:

```rust
fn compute_scan_cost(table: &Table, shard_id: ShardId) -> f64 {
    if is_local_shard(shard_id) {
        LOCAL_SCAN_COST
    } else {
        REMOTE_SCAN_COST + NETWORK_COST
    }
}
```

## Best Practices

### Design for Single-Shard Queries

Most queries should target a single shard:

```
-- Good (single shard)
retrieve users where id = 100

-- Less optimal (all shards)
retrieve users where age > 18
```

### Use Co-Location

Related data should be co-located:

```
-- Co-locate users and their orders
shard both by user_id
```

### Avoid Hot Shards

Distribute load evenly:

```
-- Bad: celebrity users create hot shard
shard by user_id

-- Better: add randomness for celebrities
shard by hash(user_id || random_suffix)
```

### Monitor Shard Health

Track shard metrics:

```
-- Size
-- Query rate
-- Hot keys
-- Replication lag
```

Rebalance proactively before problems occur.

### Plan for Growth

Choose shard key that supports splitting:

```
-- Good: can split at any point
shard by hash(id)

-- Bad: hard to split
shard by (country, region, city)
```

## Future Enhancements

### Auto-Sharding

Automatically split shards based on size/load:

```rust
if shard_size > SPLIT_THRESHOLD {
    split_shard(shard_id);
}
```

### Hot Key Detection

Identify and mitigate hot keys:

```rust
if access_rate(key) > HOT_KEY_THRESHOLD {
    replicate_key_to_multiple_shards(key);
}
```

### Cross-Shard Transactions

Optimize multi-shard transactions:

- Parallel prepare phase
- Early abort on conflicts
- Transaction state caching
