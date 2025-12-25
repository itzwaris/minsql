# minsql Quick Reference Guide

## Installation

```bash
# APT (Ubuntu/Debian)
sudo apt install minsql

# From source
git clone https://github.com/notwaris/minsql.git
cd minsql && ./tools/build.sh release
sudo cp target/release/minsql /usr/bin/
```

## Service Management

```bash
# Start
sudo systemctl start minsql

# Stop
sudo systemctl stop minsql

# Status
sudo systemctl status minsql

# Logs
sudo journalctl -u minsql -f

# Enable autostart
sudo systemctl enable minsql
```

## Basic Queries

### Data Retrieval
```sql
-- All rows
retrieve users

-- With filter
retrieve users where age > 18

-- With columns
retrieve name, email from users where active = true

-- With ordering
retrieve users order by created_at desc limit 10

-- Time-travel
retrieve users at timestamp '2024-11-10 12:03:21'
```

### Data Modification
```sql
-- Insert
insert into users (name, email, age) values ('Alice', 'alice@example.com', 30)

-- Update
update users set age = 31 where name = 'Alice'

-- Delete
delete from users where age < 18
```

### Schema Management
```sql
-- Create table
create table users (
  id bigint primary key,
  name text not null,
  email text unique,
  age integer
)

-- Create index
create index users_email_idx on users (email)

-- Drop table
drop table users
```

## Transactions

```sql
-- Standard
begin transaction
  update accounts set balance = balance - 100 where id = 1
  update accounts set balance = balance + 100 where id = 2
commit

-- Deterministic
begin deterministic transaction
  retrieve users order by id
commit

-- With timestamp
begin deterministic transaction at timestamp '2024-12-25 10:00:00'
  retrieve users
commit
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
- **GitHub**: https://github.com/notwaris/minsql

## Support

- Issues: https://github.com/notwaris/minsql/issues
- Discussions: https://github.com/notwaris/minsql/discussions
