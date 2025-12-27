# minsql Setup and Deployment Guide

## Table of Contents
- [System Requirements](#system-requirements)
- [Installation Methods](#installation-methods)
- [Configuration](#configuration)
- [Running minsql](#running-minsql)
- [Command Reference](#command-reference)
- [Cluster Setup](#cluster-setup)
- [Production Deployment](#production-deployment)
- [Monitoring and Maintenance](#monitoring-and-maintenance)

## System Requirements

### Minimum Requirements
- **OS**: Linux (Ubuntu 25.10+ or Debian 13+ recommended)
- **CPU**: 2 cores
- **RAM**: 4 GB
- **Disk**: 20 GB available space
- **Network**: 1 Gbps recommended for cluster deployments

### Recommended Requirements
- **OS**: Ubuntu 25.10 LTS
- **CPU**: 8 cores
- **RAM**: 16 GB
- **Disk**: 100 GB SSD
- **Network**: 10 Gbps for high-traffic clusters

### Software Dependencies
- GCC 11+ or Clang 14+ (for C/C++ compilation)
- Rust 1.82+ (for building from source)
- systemd (for service management)

## Installation Methods

### Method 1: APT Package (Recommended)

```bash
# Add minsql repository (when available)
sudo apt update
sudo apt install minsql
```

The package installation automatically:
- Creates `minsql` user and group
- Sets up directories in `/var/lib/minsql` and `/var/log/minsql`
- Installs systemd service
- Configures basic security settings

### Method 2: Build from Source

```bash
# Clone repository
git clone https://github.com/notwaris/minsql.git
cd minsql

# Build release binary
./tools/build.sh release

# Install binary
sudo cp target/release/minsql /usr/bin/
sudo chmod +x /usr/bin/minsql

# Create user and directories
sudo useradd --system --home-dir /var/lib/minsql --shell /bin/false minsql
sudo mkdir -p /var/lib/minsql /var/log/minsql
sudo chown -R minsql:minsql /var/lib/minsql /var/log/minsql

# Install systemd service
sudo cp packaging/systemd/minsql.service /etc/systemd/system/
sudo systemctl daemon-reload
```

### Method 3: Development Setup

```bash
# Clone and build in debug mode
git clone https://github.com/notwaris/minsql.git
cd minsql
./tools/build.sh debug

# Run directly without installing
./target/debug/minsql --node-id 1 --data-dir ./dev-data --port 5433
```

## Configuration

### Configuration File

Create `/etc/minsql/minsql.conf`:

```toml
# Node configuration
node_id = 1
data_dir = "/var/lib/minsql"
port = 5433

# Cluster configuration
peers = ["node2:5433", "node3:5433"]

# Performance settings
buffer_pool_size = 1024  # Number of pages in buffer pool
wal_buffer_size = 65536  # WAL buffer size in bytes

# Feature flags
deterministic = false
num_shards = 16

# Security settings
enable_encryption = false
enable_audit_log = true
enable_row_level_security = true

# Analytics features
enable_columnar_storage = true
enable_query_cache = true
query_cache_size = 1000
query_cache_ttl_seconds = 300

# Monitoring
enable_health_checks = true
health_check_interval_seconds = 60
enable_performance_monitoring = true
```

### Environment Variables

```bash
export MINSQL_NODE_ID=1
export MINSQL_DATA_DIR=/var/lib/minsql
export MINSQL_PORT=5433
export MINSQL_LOG_LEVEL=info
export RUST_BACKTRACE=1
```

## Running minsql

### Single Node

#### Using systemd (Production)

```bash
# Start service
sudo systemctl start minsql

# Check status
sudo systemctl status minsql

# View logs
sudo journalctl -u minsql -f

# Enable autostart
sudo systemctl enable minsql

# Stop service
sudo systemctl stop minsql

# Restart service
sudo systemctl restart minsql
```

#### Manual Start (Development)

```bash
# Start with default settings
minsql --node-id 1 --data-dir ./data --port 5433

# Start with custom configuration
minsql --node-id 1 --data-dir ./data --port 5433 --peers "node2:5433,node3:5433"

# Start in deterministic mode
minsql --node-id 1 --data-dir ./data --port 5433 --deterministic
```

### Using Helper Scripts

```bash
# Build the project
./tools/build.sh release

# Run single node
./tools/run-node.sh 1 ./data/node-1 5433

# Start 3-node cluster
./tools/start-cluster.sh 3
```

## Command Reference

### Server Commands

#### Start Server
```bash
minsql [OPTIONS]
```

**Options:**
- `--node-id <ID>` - Node identifier (required)
- `--data-dir <PATH>` - Data directory path (default: ./data)
- `--port <PORT>` - Server port (default: 5433)
- `--peers <LIST>` - Comma-separated list of peer addresses
- `--deterministic` - Enable deterministic execution mode
- `--config <FILE>` - Path to configuration file

**Examples:**
```bash
# Basic single node
minsql --node-id 1 --data-dir /var/lib/minsql --port 5433

# Cluster node with peers
minsql --node-id 2 --data-dir /var/lib/minsql --port 5434 \
  --peers "node1:5433,node3:5435"

# Deterministic mode
minsql --node-id 1 --data-dir /var/lib/minsql --port 5433 --deterministic
```

### Client Commands

#### Connect to Database
```bash
minsql-client --host <HOST> --port <PORT> --user <USER>
```

**Options:**
- `--host <HOST>` - Server hostname (default: localhost)
- `--port <PORT>` - Server port (default: 5433)
- `--user <USER>` - Username for authentication
- `--database <DB>` - Database name (default: default)

**Example:**
```bash
minsql-client --host localhost --port 5433 --user admin
```

### Query Language Commands

#### Data Retrieval
```sql
-- Basic retrieval
retrieve users

-- With filter
retrieve users where age > 18

-- With projection
retrieve name, email from users where active = true

-- With ordering and limit
retrieve users where age > 18 order by created_at desc limit 10

-- Time-travel query
retrieve users where age > 18 at timestamp '2024-11-10 12:03:21'

-- Joins
retrieve users.name, orders.total
from users
join orders on users.id = orders.user_id
where orders.created_at > '2024-01-01'
```

#### Data Modification
```sql
-- Insert
insert into users (name, email, age) values ('Alice', 'alice@example.com', 30)

-- Update
update users set age = 31 where name = 'Alice'

-- Delete
delete from users where age < 18
```

#### Schema Management
```sql
-- Create table
create table users (
  id bigint primary key,
  name text not null,
  email text unique not null,
  age integer,
  created_at timestamp default now()
)

-- Create index
create index users_email_idx on users (email)

-- Drop table
drop table users
```

#### Transaction Management
```sql
-- Standard transaction
begin transaction
  update accounts set balance = balance - 100 where id = 1
  update accounts set balance = balance + 100 where id = 2
commit

-- Deterministic transaction
begin deterministic transaction
  retrieve users order by id
commit

-- Time-travel transaction
begin deterministic transaction at timestamp '2024-11-10 12:03:21'
  retrieve users where age > 18
commit
```

### Administrative Commands

#### User Management
```sql
-- Create user
create user alice with password 'secure_password'

-- Grant role
grant role readwrite to alice

-- Revoke role
revoke role readwrite from alice

-- List users
show users
```

#### Role Management
```sql
-- Create role
create role analyst

-- Grant permission
grant select, insert on users to analyst

-- Revoke permission
revoke insert on users from analyst

-- List roles
show roles
```

#### Materialized Views
```sql
-- Create materialized view
create materialized view user_stats as
  retrieve count(*), avg(age) from users group by active

-- Refresh materialized view
refresh materialized view user_stats

-- Drop materialized view
drop materialized view user_stats

-- List materialized views
show materialized views
```

#### Monitoring Commands
```sql
-- Show health status
show health

-- Show performance statistics
show performance stats

-- Show active queries
show active queries

-- Show slow queries
show slow queries threshold 1000ms

-- Show cache statistics
show cache stats

-- Show replication status
show replication status

-- Show audit log
show audit log limit 100
```

### System Commands

#### Checkpoint
```bash
# Create checkpoint
minsql-admin checkpoint

# Force checkpoint
minsql-admin checkpoint --force
```

#### Backup
```bash
# Create backup
minsql-admin backup --output /backups/minsql-backup-$(date +%Y%m%d).tar.gz

# Restore backup
minsql-admin restore --input /backups/minsql-backup-20241225.tar.gz
```

#### Cluster Management
```bash
# Add node to cluster
minsql-admin cluster add-node --node-id 4 --address node4:5436

# Remove node from cluster
minsql-admin cluster remove-node --node-id 4

# Rebalance shards
minsql-admin cluster rebalance

# Show cluster status
minsql-admin cluster status
```

## Cluster Setup

### 3-Node Cluster Deployment

#### Node 1 (Primary)
```bash
minsql --node-id 1 \
  --data-dir /var/lib/minsql/node1 \
  --port 5433 \
  --peers "node2:5434,node3:5435"
```

#### Node 2
```bash
minsql --node-id 2 \
  --data-dir /var/lib/minsql/node2 \
  --port 5434 \
  --peers "node1:5433,node3:5435"
```

#### Node 3
```bash
minsql --node-id 3 \
  --data-dir /var/lib/minsql/node3 \
  --port 5435 \
  --peers "node1:5433,node2:5434"
```

### Automated Cluster Setup

```bash
# Use provided script
./tools/start-cluster.sh 3
```

This starts a 3-node cluster on localhost with ports 5433, 5434, 5435.

### Cluster Verification

```bash
# Check cluster health
minsql-client --host localhost --port 5433 << EOF
show cluster status
EOF
```

## Production Deployment

### Pre-Deployment Checklist

- [ ] Hardware meets recommended specifications
- [ ] OS is updated with latest security patches
- [ ] Firewall rules configured (allow port 5433 or custom port)
- [ ] Data directory has sufficient space
- [ ] Backup strategy defined
- [ ] Monitoring configured
- [ ] Security policies reviewed

### Firewall Configuration

```bash
# Allow minsql port
sudo ufw allow 5433/tcp

# For cluster, allow peer ports
sudo ufw allow 5434/tcp
sudo ufw allow 5435/tcp

# Enable firewall
sudo ufw enable
```

### Security Hardening

#### Enable Encryption
```bash
# Generate master key
openssl rand -base64 32 > /etc/minsql/master.key
sudo chmod 600 /etc/minsql/master.key
sudo chown minsql:minsql /etc/minsql/master.key
```

Add to configuration:
```toml
enable_encryption = true
master_key_path = "/etc/minsql/master.key"
```

#### Enable Audit Logging
```toml
enable_audit_log = true
audit_log_path = "/var/log/minsql/audit.log"
```

#### Configure Row-Level Security
```sql
-- Create policy
create policy user_isolation on users
  using (user_id = current_user_id())
  for all operations
  to all roles

-- Enable RLS on table
alter table users enable row level security
```

### Performance Tuning

#### Memory Settings
```toml
buffer_pool_size = 2048  # Increase for larger datasets
wal_buffer_size = 131072  # Increase for write-heavy workloads
```

#### Query Cache
```toml
enable_query_cache = true
query_cache_size = 5000
query_cache_ttl_seconds = 600
```

#### Columnar Storage (for analytics)
```toml
enable_columnar_storage = true
columnar_compression = true
```

### Backup Strategy

#### Automated Backups
```bash
# Add to cron
0 2 * * * /usr/bin/minsql-admin backup --output /backups/minsql-$(date +\%Y\%m\%d).tar.gz
```

#### Backup Script
```bash
#!/bin/bash
BACKUP_DIR="/backups/minsql"
RETENTION_DAYS=30

# Create backup
minsql-admin backup --output "${BACKUP_DIR}/minsql-$(date +%Y%m%d-%H%M%S).tar.gz"

# Remove old backups
find "${BACKUP_DIR}" -name "minsql-*.tar.gz" -mtime +${RETENTION_DAYS} -delete
```

## Monitoring and Maintenance

### Health Monitoring

```bash
# Check health status
curl http://localhost:5433/health

# Expected response:
# {"status":"Healthy","checks":[...]}
```

### Performance Monitoring

```bash
# Get performance statistics
minsql-client << EOF
show performance stats
EOF
```

### Log Management

```bash
# View logs
sudo journalctl -u minsql -f

# View last 100 lines
sudo journalctl -u minsql -n 100

# View logs for specific date
sudo journalctl -u minsql --since "2024-12-25" --until "2024-12-26"

# Export logs
sudo journalctl -u minsql --since "2024-12-01" > /tmp/minsql-logs.txt
```

### Maintenance Tasks

#### Vacuum (Garbage Collection)
```sql
-- Manual vacuum
vacuum users

-- Vacuum with full analysis
vacuum analyze users
```

#### Statistics Update
```sql
-- Update table statistics
analyze users

-- Update all tables
analyze
```

#### Index Maintenance
```sql
-- Rebuild index
reindex users_email_idx

-- Rebuild all indexes
reindex table users
```

### Troubleshooting

#### Common Issues

**Problem: Connection refused**
```bash
# Check if service is running
sudo systemctl status minsql

# Check port binding
sudo netstat -tlnp | grep 5433

# Check logs
sudo journalctl -u minsql -n 50
```

**Problem: High memory usage**
```bash
# Reduce buffer pool size in config
buffer_pool_size = 512

# Restart service
sudo systemctl restart minsql
```

**Problem: Slow queries**
```sql
-- Check slow queries
show slow queries threshold 1000ms

-- Add missing indexes
create index on users (email)

-- Update statistics
analyze users
```

**Problem: Replication lag**
```bash
# Check replication status
minsql-client << EOF
show replication status
EOF

# Check network connectivity between nodes
ping node2
telnet node2 5434
```

### Upgrade Procedure

```bash
# 1. Backup data
minsql-admin backup --output /backups/pre-upgrade-backup.tar.gz

# 2. Stop service
sudo systemctl stop minsql

# 3. Install new version
sudo apt update
sudo apt install minsql

# 4. Run migrations (if needed)
minsql-admin migrate

# 5. Start service
sudo systemctl start minsql

# 6. Verify
minsql-client << EOF
show version
EOF
```

## Next Steps

- Review [Architecture Documentation](vision.md)
- Learn about [Query Language](language.md)
- Understand [Deterministic Execution](determinism.md)
- Explore [Sharding](sharding.md) and [Replication](replication.md)
- Configure [Security Features](#security-hardening)
