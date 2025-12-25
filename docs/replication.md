# Replication

## Overview

minsql uses Raft consensus for replication. Each shard has a Raft group with one leader and multiple followers. Writes go through Raft to ensure consistency.

## Raft Consensus

### Raft Basics

Raft provides:

- Leader election
- Log replication
- Safety (committed entries survive failures)

### Raft Roles

```rust
pub enum RaftRole {
    Leader,
    Follower,
    Candidate,
}
```

**Leader**: Accepts writes, replicates to followers
**Follower**: Receives log entries from leader
**Candidate**: Competing to become leader during election

## Replication Architecture

### Raft Group per Shard

```rust
pub struct RaftGroup {
    pub shard_id: ShardId,
    pub members: Vec<NodeId>,
    pub leader: Option<NodeId>,
    pub term: u64,
}
```

Each shard has its own Raft group with independent leader election.

### Log Structure

```rust
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub entry_type: LogEntryType,
    pub data: Vec<u8>,
}

pub enum LogEntryType {
    Write,
    Config,
    Snapshot,
}
```

## Write Path

### Write Flow

```
Client
  ↓
Leader Node
  ↓
Append to local log
  ↓
Replicate to followers (parallel)
  ↓
Wait for quorum
  ↓
Apply to state machine
  ↓
Respond to client
```

### Replication Protocol

```rust
async fn replicate_entry(
    leader: &Leader,
    entry: LogEntry,
) -> Result<()> {
    // Append to local log
    leader.log.append(entry.clone());
    
    // Send to all followers
    let mut acks = 0;
    for follower in &leader.followers {
        let response = send_append_entries(follower, vec![entry.clone()]).await?;
        if response.success {
            acks += 1;
        }
    }
    
    // Wait for quorum
    let quorum = (leader.members.len() / 2) + 1;
    if acks + 1 >= quorum {
        // Leader counts as ack
        leader.commit_index = entry.index;
        apply_to_state_machine(entry)?;
        Ok(())
    } else {
        Err(QuorumNotReached)
    }
}
```

### Quorum Writes

A write is committed when replicated to a majority:

```
3 nodes: quorum = 2 (leader + 1 follower)
5 nodes: quorum = 3 (leader + 2 followers)
```

## Read Path

### Linearizable Reads

By default, reads go to the leader for linearizability:

```rust
async fn read(leader: &Leader, query: Query) -> Result<Vec<Row>> {
    // Ensure we're still the leader
    leader.heartbeat().await?;
    
    // Execute query
    let result = execute_query(query)?;
    Ok(result)
}
```

### Follower Reads

Stale reads can go to followers:

```
retrieve users
where id = 100
with consistency = eventual
```

This allows higher read throughput but may return stale data.

### Read Index Protocol

For consistent follower reads:

```rust
async fn follower_read(follower: &Follower, query: Query) -> Result<Vec<Row>> {
    // Ask leader for safe read index
    let read_index = follower.request_read_index().await?;
    
    // Wait until we've applied up to read_index
    follower.wait_for_apply(read_index).await?;
    
    // Execute query
    let result = execute_query(query)?;
    Ok(result)
}
```

## Leader Election

### Election Trigger

Elections happen when:

- Follower doesn't receive heartbeat (timeout)
- Node starts up
- Leader explicitly steps down

### Election Process

```rust
async fn start_election(node: &mut Node) -> Result<()> {
    // Increment term and become candidate
    node.term += 1;
    node.role = RaftRole::Candidate;
    node.voted_for = Some(node.id);
    
    // Request votes from all peers
    let mut votes = 1; // Vote for self
    for peer in &node.peers {
        let response = send_request_vote(peer, node.term, node.last_log_index()).await?;
        if response.vote_granted {
            votes += 1;
        }
    }
    
    // Check if we won
    let quorum = (node.peers.len() / 2) + 1;
    if votes >= quorum {
        node.role = RaftRole::Leader;
        node.leader = Some(node.id);
        start_heartbeat_timer(node);
    }
    
    Ok(())
}
```

### Split Brain Prevention

Raft prevents split brain through terms:

```
Term 1: Node A is leader
Term 2: Network partition, Node B wins election
Term 3: Partition heals, Node A sees higher term, steps down
```

Only one leader per term is possible.

## Log Compaction

### Snapshots

Periodically compact the log into snapshots:

```rust
pub struct Snapshot {
    pub last_included_index: u64,
    pub last_included_term: u64,
    pub data: Vec<u8>,
}

fn create_snapshot(node: &Node) -> Snapshot {
    let state = serialize_state_machine(&node.state);
    Snapshot {
        last_included_index: node.commit_index,
        last_included_term: node.log.entry(node.commit_index).term,
        data: state,
    }
}
```

### Snapshot Transfer

New nodes or lagging followers receive snapshots:

```rust
async fn install_snapshot(
    follower: &mut Follower,
    snapshot: Snapshot,
) -> Result<()> {
    // Replace state machine
    follower.state = deserialize_state_machine(&snapshot.data)?;
    
    // Truncate log
    follower.log.truncate_before(snapshot.last_included_index);
    
    // Update indices
    follower.last_applied = snapshot.last_included_index;
    follower.commit_index = snapshot.last_included_index;
    
    Ok(())
}
```

## Membership Changes

### Configuration Changes

Add or remove nodes dynamically:

```rust
pub struct ConfigChange {
    pub change_type: ConfigChangeType,
    pub node_id: NodeId,
}

pub enum ConfigChangeType {
    AddNode,
    RemoveNode,
}

async fn apply_config_change(
    leader: &mut Leader,
    change: ConfigChange,
) -> Result<()> {
    // Append config change to log
    let entry = LogEntry {
        term: leader.term,
        index: leader.next_index(),
        entry_type: LogEntryType::Config,
        data: serialize(change),
    };
    
    // Replicate
    replicate_entry(leader, entry).await?;
    
    // Apply change
    match change.change_type {
        ConfigChangeType::AddNode => {
            leader.members.push(change.node_id);
        }
        ConfigChangeType::RemoveNode => {
            leader.members.retain(|&id| id != change.node_id);
        }
    }
    
    Ok(())
}
```

### Joint Consensus

Use Raft's joint consensus for safe membership changes:

```
Old config: [A, B, C]
New config: [A, B, D]

Step 1: Joint consensus with [A, B, C] and [A, B, D]
  Requires quorum in both configs
Step 2: Transition to [A, B, D] only
```

## Integration with WAL

### WAL + Raft

Raft log entries correspond to WAL entries:

```rust
pub struct RaftWALEntry {
    pub raft_index: u64,
    pub raft_term: u64,
    pub wal_entry: WALEntry,
}
```

### State Machine

The state machine is the storage engine:

```rust
fn apply_to_state_machine(entry: LogEntry) -> Result<()> {
    let wal_entry: WALEntry = deserialize(&entry.data)?;
    
    match wal_entry.operation {
        Operation::Insert { table, tuple } => {
            storage_insert(table, tuple)?;
        }
        Operation::Update { table, old, new } => {
            storage_update(table, old, new)?;
        }
        Operation::Delete { table, tuple } => {
            storage_delete(table, tuple)?;
        }
    }
    
    Ok(())
}
```

### Deterministic Replay

In deterministic mode, Raft replay is deterministic:

```rust
fn apply_entry_deterministic(entry: LogEntry) -> Result<()> {
    // Use logical time from entry
    let logical_time = entry.logical_time;
    set_logical_clock(logical_time);
    
    // Apply operation
    apply_to_state_machine(entry)?;
    
    Ok(())
}
```

## Failure Scenarios

### Leader Failure

```
1. Followers detect missing heartbeats
2. Followers start elections
3. New leader elected
4. New leader catches up followers
5. System operational
```

Downtime: ~election timeout (typically 150-300ms)

### Follower Failure

```
1. Leader detects failed follower
2. Leader continues with remaining quorum
3. Failed follower eventually recovers
4. Leader replays missing entries
5. Follower catches up
```

No service disruption if quorum maintained.

### Network Partition

```
Partition: [Leader + 1 follower] | [2 followers]

Majority partition:
  - Continues to serve writes
  - Has quorum

Minority partition:
  - Cannot elect new leader (no quorum)
  - Serves stale reads only

When partition heals:
  - Minority followers catch up from leader
  - System returns to normal
```

## Performance

### Replication Lag

Monitor replication lag:

```rust
pub struct ReplicationMetrics {
    pub leader_commit_index: u64,
    pub follower_commit_index: u64,
    pub lag: u64, // leader_commit_index - follower_commit_index
}
```

### Throughput

Replication throughput depends on:

- Network bandwidth
- Disk I/O (for log persistence)
- Number of followers

Typical: 10k-50k writes/sec per shard

### Batching

Batch multiple writes:

```rust
async fn batch_replicate(
    leader: &Leader,
    entries: Vec<LogEntry>,
) -> Result<()> {
    // Append all entries to log
    for entry in &entries {
        leader.log.append(entry.clone());
    }
    
    // Single RPC to replicate batch
    for follower in &leader.followers {
        send_append_entries(follower, entries.clone()).await?;
    }
    
    Ok(())
}
```

## Monitoring

### Raft Metrics

```rust
pub struct RaftMetrics {
    pub role: RaftRole,
    pub term: u64,
    pub commit_index: u64,
    pub last_applied: u64,
    pub leader_id: Option<NodeId>,
    pub election_count: u64,
}
```

### Health Checks

```rust
fn check_raft_health(group: &RaftGroup) -> HealthStatus {
    if group.leader.is_none() {
        return HealthStatus::NoLeader;
    }
    
    let lag = group.leader_commit_index - group.min_follower_commit_index;
    if lag > MAX_REPLICATION_LAG {
        return HealthStatus::LagWarning;
    }
    
    HealthStatus::Healthy
}
```

## Best Practices

### Odd Number of Nodes

Use odd numbers for better fault tolerance:

```
3 nodes: Tolerates 1 failure
5 nodes: Tolerates 2 failures
7 nodes: Tolerates 3 failures
```

Even numbers provide no additional fault tolerance:

```
4 nodes: Still tolerates only 1 failure (same as 3)
6 nodes: Still tolerates only 2 failures (same as 5)
```

### Network Considerations

Place replicas in different:

- Availability zones
- Data centers
- Geographic regions (for disaster recovery)

### Monitoring

Monitor:

- Leader election frequency (should be rare)
- Replication lag (should be low)
- Commit latency (should be consistent)

### Testing

Test failure scenarios:

- Leader crashes
- Network partitions
- Follower failures
- Concurrent writes during partition

## Future Enhancements

### Multi-Raft

Multiple Raft groups per node:

```
Node A: Leader for shards 1,3,5
        Follower for shards 2,4,6
```

Better load distribution.

### Pre-Vote

Pre-vote phase before actual election to reduce disruptions.

### Leadership Transfer

Graceful leadership transfer for maintenance:

```rust
async fn transfer_leadership(
    leader: &Leader,
    target: NodeId,
) -> Result<()> {
    // Ensure target is caught up
    while leader.followers[target].match_index < leader.commit_index {
        replicate_entries(leader, target).await?;
    }
    
    // Send TimeoutNow to target
    send_timeout_now(target).await?;
    
    // Target immediately starts election and wins
    Ok(())
}
```

### Learner Nodes

Non-voting replicas for read scaling:

```rust
pub struct Learner {
    pub node_id: NodeId,
    pub receives_logs: bool,
    pub can_vote: false,
}
```
