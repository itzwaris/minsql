---
name: Bug Report
about: Report a bug to help us improve minsql
title: '[BUG] '
labels: bug
assignees: ''
---

## Bug Description
<!-- A clear and concise description of what the bug is -->

## Environment
**minsql Version:**
<!-- Run: minsql --version -->

**Operating System:**
<!-- e.g., Ubuntu 22.04, Debian 11 -->

**Installation Method:**
- [ ] APT Package
- [ ] Built from source
- [ ] Docker
- [ ] Other (please specify):

**Deployment Mode:**
- [ ] Single node
- [ ] Cluster (specify number of nodes):

## Steps to Reproduce
<!-- Provide detailed steps to reproduce the behavior -->

1. Start minsql with: `...`
2. Execute query: `...`
3. Observe error: `...`

## Expected Behavior
<!-- What you expected to happen -->

## Actual Behavior
<!-- What actually happened -->

## Query/Code Sample
<!-- If applicable, provide the query or code that triggers the bug -->

```sql
-- Your query here
```

## Error Messages
<!-- Paste any error messages, logs, or stack traces -->

```
Paste error messages here
```

## Logs
<!-- Provide relevant log entries from /var/log/minsql or journalctl -->

```
Paste logs here
```

## Configuration
<!-- Share your configuration file (remove sensitive information) -->

```toml
# /etc/minsql/minsql.conf
node_id = 1
port = 5433
...
```

## Database State
**Number of Tables:**
**Approximate Data Size:**
**Active Connections:**
**Replication Status (if applicable):**

## Performance Metrics (if relevant)
<!-- Run: show performance stats -->

**CPU Usage:**
**Memory Usage:**
**Disk I/O:**
**Query Latency:**

## Workaround
<!-- If you found a workaround, please share it -->

## Additional Context
<!-- Add any other context about the problem here -->

## Checklist
- [ ] I have searched existing issues to ensure this is not a duplicate
- [ ] I have included all relevant information above
- [ ] I have provided steps to reproduce the issue
- [ ] I have included error messages and logs
- [ ] I have tested on the latest version of minsql

## Screenshots
<!-- If applicable, add screenshots to help explain the problem -->
