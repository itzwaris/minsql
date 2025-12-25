# minsql

A production-grade, intent-driven database system with deterministic execution capabilities.

## What is minsql?

minsql is a new class of database that prioritizes:

- **Intent-driven queries**: Operations are parsed into semantic intent rather than rigid syntax
- **Deterministic execution**: Same input produces identical output and timing when enabled
- **Time-travel queries**: Query historical data at any point in time
- **First-class sharding**: The planner understands data locality and minimizes cross-shard operations
- **Crash safety**: WAL-based recovery with idempotent replay guarantees
- **Query sandboxing**: Per-query CPU, memory, and time limits

## Core Differentiators

### Intent-Aware Query Model
Unlike traditional SQL databases that operate on syntax trees, minsql parses queries into semantic intent. The optimizer operates on what you want to accomplish, not how you phrased it.

### Deterministic Execution Mode
When deterministic mode is enabled, queries produce identical results with identical timing. This enables:
- Reproducible debugging
- Consistent replication
- Predictable performance testing
- Audit compliance

### Time-Travel Queries
Query data as it existed at any point in the past:
```
retrieve users where created_at < '2024-01-01' at timestamp '2024-11-10 12:03:21'
```

### Query-Level Sandboxing
Every query runs with configurable resource limits:
- CPU budget enforcement
- Memory allocation caps
- Wall-clock time limits
- Per-query isolation

## Installation

### From APT Repository

```bash
sudo apt install minsql
```

### Building from Source

```bash
git clone https://github.com/notwaris/minsql.git
cd minsql
cargo build --release
```

## Quick Start

Start the minsql server:

```bash
sudo systemctl start minsql
```

Connect with the client:

```bash
minsql-client --host localhost --port 5433
```

## Architecture

minsql uses a hybrid Rust/C/C++ architecture:

- **Rust**: Control plane, query planning, execution orchestration, networking
- **C**: WAL, page manager, buffer pool, crash recovery
- **C++**: Indexes (B-tree, hash, bloom filter), compression

This split maximizes both safety and performance.

## Documentation

See the `docs/` directory for detailed documentation:

- [Vision](docs/vision.md) - Project goals and philosophy
- [Language](docs/language.md) - Query language reference
- [Execution](docs/execution.md) - Execution model
- [Optimizer](docs/optimizer.md) - Query optimization
- [Determinism](docs/determinism.md) - Deterministic execution
- [Storage](docs/storage.md) - Storage engine internals
- [Transactions](docs/transactions.md) - MVCC and isolation
- [Sharding](docs/sharding.md) - Distributed data placement
- [Replication](docs/replication.md) - Consensus and state sync

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Project Status

minsql is under active development. Production use is not yet recommended.

## Author

Created by [notwaris](https://github.com/notwaris)
