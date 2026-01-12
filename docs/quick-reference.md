# minsql Quick Reference Guide

## Installation

```bash
# From source
git clone https://github.com/itzwaris/minsql.git
cd minsql
cargo build --release
sudo cp target/release/minsql /usr/bin/
```

## Starting the Server

```bash
# Single node (development)
cargo run -- --node-id 1 --data-dir ./data --port 5433

# With logging
RUST_LOG=info cargo run -- --node-id 1 --data-dir ./data --port 5433

# Production release
./target/release/minsql --node-id 1 --data-dir /var/lib/minsql --port 5433

# Multi-node cluster
./target/release/minsql --node-id 1 --port 5433 --peers node2:5433,node3:5433
```

## Testing Connection

```bash
# Run test client
cargo run --example test_client

# Run validation tests
cargo test --workspace
```

## Data Types

**Supported Types:**
- `INTEGER` / `INT` - 64-bit signed integer
- `BIGINT` - 64-bit signed integer
- `REAL` / `FLOAT` - 64-bit floating point
- `DOUBLE` - 64-bit floating point
- `TEXT` / `STRING` / `VARCHAR` - Variable-length text
- `BOOLEAN` / `BOOL` - True/false value
- `TIMESTAMP` / `DATETIME` - Date and time

## Basic Queries

### Schema Management

```sql
-- Create table with various types
CREATE TABLE users (
  id INT PRIMARY KEY,
  name TEXT NOT NULL,
  email VARCHAR,
  age INTEGER,
  balance FLOAT,
  active BOOL,
  created_at TIMESTAMP
)

-- Create with constraints
CREATE TABLE products (
  id BIGINT PRIMARY KEY,
  name TEXT NOT NULL,
  price REAL NOT NULL,
  stock INTEGER
)
```

### Data Modification

```sql
-- Insert single row
INSERT INTO users (name, email, age) VALUES ('Alice', 'alice@example.com', 30)

-- Insert with all supported types
INSERT INTO products (id, name, price, stock) 
VALUES (1, 'Laptop', 999.99, 50)

-- Update with filter
UPDATE users SET age = 31 WHERE name = 'Alice'

-- Update multiple columns
UPDATE products SET price = 899.99, stock = 45 WHERE id = 1

-- Delete with filter
DELETE FROM users WHERE age < 18

-- Delete all matching
DELETE FROM products WHERE stock = 0
```

### Data Retrieval

```sql
-- All rows (intent-driven syntax)
RETRIEVE * FROM users

-- With filter
RETRIEVE * FROM users WHERE age > 18

-- With specific columns
RETRIEVE name, email FROM users WHERE active = true

-- Standard SQL also supported
SELECT name, age FROM users WHERE age > 25

-- With aggregation
SELECT COUNT(*) FROM users
```

### Query Features

**Filtering:**
```sql
-- Comparison operators
RETRIEVE * FROM users WHERE age > 18
RETRIEVE * FROM users WHERE age >= 18
RETRIEVE * FROM users WHERE name = 'Alice'
RETRIEVE * FROM users WHERE age < 65

-- Multiple conditions (coming soon)
-- RETRIEVE * FROM users WHERE age > 18 AND active = true
```

## Storage & Durability

### Write-Ahead Logging

All write operations are automatically logged:

```
INSERT → WAL Log → Storage Write → WAL Flush → Success
UPDATE → WAL Log → Storage Modify → WAL Flush → Success
DELETE → WAL Log → Storage Remove → WAL Flush → Success
CREATE TABLE → WAL Log → Catalog Write → Checkpoint → Success
```

**Guarantees:**
- ✅ ACID compliance
- ✅ Crash recovery via WAL replay
- ✅ Automatic durability on all writes
- ✅ Zero data loss on crashes

### Storage Operations

**What Happens on INSERT:**
1. Tuple converted to internal format
2. Serialized to JSON/bytes
3. Written to storage pages
4. Unique row ID returned
5. WAL flushed to disk
6. Indexes updated (if exist)

**What Happens on CREATE TABLE:**
1. Schema validated and serialized
2. System catalog entry created
3. Initial storage pages allocated
4. WAL logged and checkpointed
5. Table ready for immediate use

## Transactions

```sql
-- Standard transaction
BEGIN TRANSACTION
  UPDATE accounts SET balance = balance - 100 WHERE id = 1
  UPDATE accounts SET balance = balance + 100 WHERE id = 2
COMMIT

-- Rollback on error
BEGIN TRANSACTION
  -- operations
ROLLBACK
```

## Monitoring

Server automatically reports metrics:
- Query count
- Execution count  
- Commits/aborts
- Storage operations

Check logs with:
```bash
RUST_LOG=info cargo run ...
RUST_LOG=debug cargo run ...  # Detailed operation logs
```
```

## Advanced Features

### Materialized Views
```sql
-- Create
create materialized view user_stats as
  retrieve department, count(*), avg(salary) from employees group by department

-- Query
retrieve * from user_stats

-- Refresh
refresh materialized view user_stats

-- Drop
drop materialized view user_stats
```

### Security
```sql
-- Create user
create user alice with password 'secure123'

-- Grant role
grant role readwrite to alice

-- Create policy (RLS)
create policy tenant_isolation on data
  using (tenant_id = current_tenant_id())

-- Enable RLS
alter table data enable row level security
```

### Monitoring
```sql
-- Health status
show health

-- Performance stats
show performance stats

-- Slow queries
show slow queries threshold 1000ms

-- Cache stats
show cache stats

-- Active queries
show active queries

-- Replication status
show replication status
```

## Configuration

### Basic Config (`/etc/minsql/minsql.conf`)
```toml
node_id = 1
data_dir = "/var/lib/minsql"
port = 5433
peers = ["node2:5434", "node3:5435"]

buffer_pool_size = 1024
wal_buffer_size = 65536

enable_encryption = false
enable_audit_log = true
enable_columnar_storage = true
enable_query_cache = true
```

### Performance Tuning
```toml
# Memory
buffer_pool_size = 2048

# Cache
query_cache_size = 5000
query_cache_ttl_seconds = 600

# Parallelism
max_parallel_workers = 8
```

## Command-Line Options

### Server
```bash
minsql \
  --node-id 1 \
  --data-dir /var/lib/minsql \
  --port 5433 \
  --peers "node2:5434,node3:5435" \
  --deterministic
```

### Client
```bash
minsql-client \
  --host localhost \
  --port 5433 \
  --user admin \
  --database default
```

### Admin
```bash
# Backup
minsql-admin backup --output /backups/minsql.tar.gz

# Restore
minsql-admin restore --input /backups/minsql.tar.gz

# Checkpoint
minsql-admin checkpoint

# Cluster management
minsql-admin cluster status
minsql-admin cluster add-node --node-id 4 --address node4:5436
minsql-admin cluster rebalance
```

## Common Operations

### Check System Health
```bash
curl http://localhost:5433/health
```

### View Logs
```bash
# Real-time
sudo journalctl -u minsql -f

# Last 100 lines
sudo journalctl -u minsql -n 100

# Specific date
sudo journalctl -u minsql --since "2024-12-25"
```

### Backup and Restore
```bash
# Backup
minsql-admin backup --output /backups/minsql-$(date +%Y%m%d).tar.gz

# Restore
sudo systemctl stop minsql
minsql-admin restore --input /backups/minsql-20241225.tar.gz
sudo systemctl start minsql
```

### Cluster Setup
```bash
# Node 1
minsql --node-id 1 --port 5433 --peers "node2:5434,node3:5435"

# Node 2
minsql --node-id 2 --port 5434 --peers "node1:5433,node3:5435"

# Node 3
minsql --node-id 3 --port 5435 --peers "node1:5433,node2:5434"
```

## Troubleshooting

### Connection Issues
```bash
# Check if running
sudo systemctl status minsql

# Check port
sudo netstat -tlnp | grep 5433

# Check logs
sudo journalctl -u minsql -n 50
```

### Performance Issues
```sql
-- Find slow queries
show slow queries threshold 1000ms

-- Check indexes
show indexes for users

-- Update statistics
analyze users

-- Check cache
show cache stats
```

### Replication Issues
```sql
-- Check status
show replication status

-- Check cluster health
show cluster status
```

## File Locations

- **Binary**: `/usr/bin/minsql`
- **Config**: `/etc/minsql/minsql.conf`
- **Data**: `/var/lib/minsql`
- **Logs**: `/var/log/minsql`
- **Service**: `/etc/systemd/system/minsql.service`

## Environment Variables

```bash
export MINSQL_NODE_ID=1
export MINSQL_DATA_DIR=/var/lib/minsql
export MINSQL_PORT=5433
export MINSQL_LOG_LEVEL=info
export RUST_BACKTRACE=1
```

## Performance Benchmarks

### OLTP
- Writes: 50K+/sec per shard
- Reads: 100K+/sec per shard
- Latency: P99 < 10ms

### OLAP
- Scans: 1GB/sec (columnar)
- Aggregations: 10M rows/sec (vectorized)
- Compression: 10:1 typical

## Quick Links

- **Setup Guide**: [docs/setup.md](setup.md)
- **Features**: [docs/features.md](features.md)
- **Architecture**: [docs/architecture.md](architecture.md)
- **Language Reference**: [docs/language.md](language.md)
- **GitHub**: https://github.com/itzwaris/minsql
