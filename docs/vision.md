# minsql Vision

## The Problem

Modern databases operate in a world of legacy constraints:

- **Syntax-driven**: Optimizers work with query syntax rather than semantic intent
- **Nondeterministic**: Same query can produce different results or timing
- **Crash recovery complexity**: WAL replay is often fragile and hard to verify
- **Sharding as an afterthought**: Data locality is not a first-class concern
- **PostgreSQL shadow**: New databases often just reimplement PostgreSQL semantics

This creates systems that are:
- Hard to debug (nondeterminism)
- Hard to test (timing variance)
- Hard to scale (poor sharding awareness)
- Hard to innovate (bound by compatibility)

## The Solution

minsql rethinks database architecture from first principles:

### Intent-Driven Queries

Instead of optimizing syntax trees, minsql parses queries into semantic intent. The optimizer operates on what you want to achieve, not how you phrased it. This allows:

- Better optimization opportunities
- Clearer cost modeling
- More flexible query rewriting
- Language evolution without breaking changes

### Deterministic Execution

When deterministic mode is enabled, minsql guarantees:
- Identical output for identical input
- Identical timing characteristics
- Reproducible query plans
- Predictable resource usage

This enables:
- Reproducible debugging
- Reliable replication
- Audit compliance
- Performance regression detection

### First-Class Sharding

The query planner understands data distribution:
- Routing decisions happen at plan time
- Cross-shard operations are minimized
- Data locality drives physical planning
- Rebalancing is transparent

### Time Travel

Query data at any historical point:
- MVCC with garbage collection
- Point-in-time snapshots
- Audit trail capabilities
- Rollback analysis

### Query Sandboxing

Every query runs with resource limits:
- CPU budget enforcement
- Memory caps
- Time limits
- Priority scheduling

This prevents runaway queries from affecting system stability.

## Design Principles

### Safety First

- Rust for control plane correctness
- Bounds checking on all FFI boundaries
- Type safety throughout
- Resource cleanup guarantees

### Performance Where It Matters

- C for deterministic hot paths
- C++ for complex data structures
- Zero-copy where possible
- Cache-conscious algorithms

### Operational Excellence

- Observable by default
- Metrics and tracing built-in
- Clean error messages
- Actionable diagnostics

### Production Ready

- Not a toy or academic project
- Battle-tested algorithms
- Comprehensive testing
- Real-world workload focus

## Non-Goals


### Not a Key-Value Store

minsql is a relational database with:
- Schema enforcement
- Query planning
- ACID guarantees
- Relational operators

### Not NoSQL

minsql has:
- Strong consistency by default
- Schema requirements
- Transactional semantics
- Structured query language

## Target Users

### Engineering Teams

Teams building applications that need:
- Predictable performance
- Debugging capabilities
- Scale-out architecture
- Operational visibility

### Financial Services

Organizations requiring:
- Audit trails
- Reproducible execution
- Point-in-time recovery
- Compliance guarantees

### Platform Engineers

Developers building:
- Multi-tenant systems
- Resource-isolated workloads
- Distributed applications
- Sharded architectures

## Success Metrics

minsql is successful when:

1. **Determinism works**: Queries produce identical results and timing in deterministic mode
2. **Performance is competitive**: Within 2x of PostgreSQL for OLTP workloads
3. **Crash recovery is reliable**: WAL replay succeeds in all failure scenarios
4. **Sharding is transparent**: Applications don't need shard awareness
5. **Operations are simple**: Single binary, clear configuration, good observability

## Philosophy

minsql is guided by these beliefs:

**Simplicity over features**: Better to do fewer things well than many things poorly.

**Correctness over performance**: Speed means nothing if results are wrong.

**Observability over assumptions**: Measure, don't guess.

**Intent over syntax**: Understand what users want, not just what they said.

**Determinism over convenience**: Reproducibility is worth the cost.

This is not a database for everyone. It's a database for teams who value correctness, predictability, and operational clarity over maximum compatibility or bleeding-edge features.
