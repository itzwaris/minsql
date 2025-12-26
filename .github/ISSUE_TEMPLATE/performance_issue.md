---
name: Performance Issue
about: Report a performance problem or slow query
title: '[PERFORMANCE] '
labels: performance
assignees: ''
---

## Performance Issue Description
<!-- Describe the performance problem -->

## Environment
**minsql Version:**
**OS:**
**Hardware:**
- CPU:
- RAM:
- Disk Type (SSD/HDD):
- Network:

**Deployment:**
- [ ] Single node
- [ ] Cluster: ___ nodes

## Problem Details

### Query Performance
**Query:**
```sql
-- Your query here
```

**Execution Time:**
- Current: ___ms
- Expected: ___ms

**Execution Plan:**
```
-- Output of: explain analyze <your query>
```

### System Metrics During Issue
**CPU Usage:** ___%
**Memory Usage:** ___%
**Disk I/O:** ___
**Network I/O:** ___
**Active Connections:** ___

### Query Statistics
```sql
-- Output of: show performance stats
```

**Slow Queries:**
```sql
-- Output of: show slow queries threshold 1000ms
```

## Data Characteristics
**Table Size:**
**Row Count:**
**Index Count:**
**Data Growth Rate:**

## Configuration
```toml
# Relevant configuration settings
buffer_pool_size = 
query_cache_size = 
max_parallel_workers = 
```

## Reproducibility
- [ ] Consistently slow
- [ ] Intermittently slow
- [ ] Slow during specific times
- [ ] Slow with specific data patterns

## Steps to Reproduce
1. 
2. 
3. 

## Baseline Performance
<!-- If you have baseline metrics, share them -->

**Previous Performance:**
**When did degradation start:**

## Attempted Solutions
<!-- What have you tried to improve performance? -->

- [ ] Added indexes
- [ ] Increased buffer pool
- [ ] Updated statistics (ANALYZE)
- [ ] Query rewrite
- [ ] Other:

## Expected Performance
<!-- What performance level do you expect? -->

## Additional Context
<!-- Any other relevant information -->

## Checklist
- [ ] I have run EXPLAIN ANALYZE on the slow query
- [ ] I have checked system resources (CPU, memory, disk)
- [ ] I have reviewed slow query logs
- [ ] I have tried basic optimizations
- [ ] I have included all relevant metrics
