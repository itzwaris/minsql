# minsql Streaming Features

## Overview

minsql includes advanced streaming capabilities for real-time data processing, event sourcing, and reactive applications. These features enable you to build modern, event-driven architectures directly within your database.

## Features

### 1. Continuous Queries 🔄

Execute queries continuously over streaming data with windowing support.

#### Creating Continuous Queries

```sql
-- Create a tumbling window continuous query
create continuous query active_users_count
  on table user_events
  window tumbling 1 minute
  as retrieve count(*) where event_type = 'login'
  output into active_users_stats

-- Sliding window
create continuous query avg_response_time
  on table api_requests
  window sliding 5 minutes step 1 minute
  as retrieve avg(response_time)
  output into performance_metrics

-- Session window (gap-based)
create continuous query user_sessions
  on table user_activity
  window session 30 minutes
  as retrieve user_id, count(*) as actions
  group by user_id
  output into session_stats
```

#### Window Types

**Tumbling Windows:**
- Fixed-size, non-overlapping windows
- Data belongs to exactly one window
- Example: Count events per minute

**Sliding Windows:**
- Fixed-size, overlapping windows
- Windows slide by a specified step
- Example: Moving average over last 5 minutes

**Session Windows:**
- Dynamic-size based on inactivity gaps
- New window starts after gap period
- Example: User session tracking

#### Managing Continuous Queries

```sql
-- List all continuous queries
show continuous queries

-- Stop continuous query
stop continuous query active_users_count

-- Resume continuous query
start continuous query active_users_count

-- Drop continuous query
drop continuous query active_users_count
```

#### Output Actions

```sql
-- Insert into table
output into target_table

-- Send notification
output notify 'channel_name'

-- Call webhook
output webhook 'https://api.example.com/events'
```

### 2. Change Data Capture (CDC) 📊

Capture and stream all data changes in real-time.

#### Enabling CDC

```sql
-- Enable CDC on table
alter table orders enable change data capture

-- Disable CDC
alter table orders disable change data capture
```

#### Subscribing to Changes

```sql
-- Subscribe to all changes
subscribe to changes on orders

-- Subscribe to specific operations
subscribe to changes on orders
  for insert, update

-- Subscribe with filter
subscribe to changes on orders
  where total > 1000

-- Subscribe to multiple tables
subscribe to changes on orders, customers, products
```

#### Change Event Structure

```json
{
  "change_id": 12345,
  "change_type": "Update",
  "table": "orders",
  "before": {
    "id": 100,
    "status": "pending",
    "total": 250.00
  },
  "after": {
    "id": 100,
    "status": "completed",
    "total": 250.00
  },
  "timestamp": "2024-12-26T10:30:00Z",
  "transaction_id": 67890
}
```

#### Consuming Changes

**Rust Client:**
```rust
let mut subscription = db.subscribe_to_changes("orders").await?;

while let Some(change) = subscription.recv().await {
    match change.change_type {
        ChangeType::Insert => println!("New order: {:?}", change.after),
        ChangeType::Update => println!("Order updated: {:?}", change.after),
        ChangeType::Delete => println!("Order deleted: {:?}", change.before),
    }
}
```

**Command Line:**
```bash
# Stream changes to stdout
minsql-client subscribe orders --format json

# Export to file
minsql-client subscribe orders --output changes.jsonl
```

#### CDC Use Cases

- **Replication**: Stream changes to other systems
- **Caching**: Invalidate caches on data changes
- **Audit**: Track all modifications
- **Analytics**: Feed data warehouses in real-time
- **Microservices**: Event-driven architecture

### 3. Event Sourcing 📝

Store all state changes as immutable events.

#### Creating Event Streams

```sql
-- Create event store for aggregate
create event store order_events
  aggregate_type 'Order'

-- Append event
append event to order_events
  aggregate_id 'order-123'
  event_type 'OrderPlaced'
  event_data '{"items": [...], "total": 250.00}'
  metadata '{"user_id": "user-456"}'
```

#### Retrieving Events

```sql
-- Get all events for aggregate
retrieve events from order_events
  where aggregate_id = 'order-123'

-- Get events from version
retrieve events from order_events
  where aggregate_id = 'order-123'
  from version 5

-- Get events by type
retrieve events from order_events
  where event_type = 'OrderPlaced'
  and timestamp > '2024-12-01'
```

#### Snapshots

```sql
-- Create snapshot
create snapshot for order_events
  aggregate_id 'order-123'
  at version 100

-- Retrieve snapshot
retrieve snapshot from order_events
  where aggregate_id = 'order-123'

-- Rebuild aggregate from snapshot
rebuild aggregate 'order-123'
  from order_events
```

#### Event Sourcing Patterns

**Command Handler:**
```rust
async fn handle_place_order(cmd: PlaceOrderCommand) -> Result<()> {
    // Validate command
    validate_order(&cmd)?;
    
    // Create event
    let event = Event {
        aggregate_id: cmd.order_id,
        event_type: "OrderPlaced".to_string(),
        event_data: serde_json::to_value(&cmd)?,
        version: get_next_version(&cmd.order_id).await?,
        ..Default::default()
    };
    
    // Append to event store
    event_store.append_event(event).await?;
    
    Ok(())
}
```

**Projection:**
```rust
async fn project_order_summary(event: &Event) -> Result<()> {
    match event.event_type.as_str() {
        "OrderPlaced" => {
            // Insert into read model
            db.execute("INSERT INTO order_summary ...")?;
        }
        "OrderShipped" => {
            // Update read model
            db.execute("UPDATE order_summary SET status = 'shipped' ...")?;
        }
        _ => {}
    }
    Ok(())
}
```

### 4. Pub/Sub Messaging 📢

Built-in publish/subscribe messaging system.

#### Publishing Messages

```sql
-- Publish to channel
publish to channel 'notifications'
  payload '{"type": "alert", "message": "High CPU usage"}'

-- Publish with headers
publish to channel 'events'
  payload '{"event": "user_signup", "user_id": 123}'
  headers '{"priority": "high", "source": "api"}'
```

#### Subscribing to Channels

```sql
-- Subscribe to channel
subscribe to channel 'notifications'

-- Subscribe to multiple channels
subscribe to channels 'notifications', 'alerts', 'events'
```

#### Channel Patterns

**Topic-based routing:**
```
channels:
  - orders.created
  - orders.updated
  - orders.cancelled
  - users.registered
  - users.verified
```

**Wildcard subscriptions:**
```sql
-- Subscribe to all order events
subscribe to channel 'orders.*'

-- Subscribe to all events
subscribe to channel '*'
```

#### Message History

```sql
-- Get recent messages
retrieve message history from channel 'notifications'
  limit 100

-- Get messages from timestamp
retrieve message history from channel 'events'
  since '2024-12-26 00:00:00'
```

#### Use Cases

- **Real-time notifications**: Push updates to connected clients
- **Job queues**: Distribute work across workers
- **Event bus**: Decouple services
- **Live dashboards**: Stream metrics
- **Chat systems**: Real-time messaging

### 5. GraphQL API 🚀

Auto-generated GraphQL API for your data.

#### Enabling GraphQL

```sql
-- Enable GraphQL endpoint
enable graphql on port 8080

-- Generate schema from tables
generate graphql schema from tables users, orders, products
```

#### Auto-Generated Schema

```graphql
type User {
  id: ID!
  name: String!
  email: String!
  createdAt: String
}

type Order {
  id: ID!
  userId: ID!
  total: Float!
  status: String!
  user: User
}

type Query {
  getUser(id: ID!): User
  listUsers(limit: Int, offset: Int): [User]
  getOrder(id: ID!): Order
  listOrders(limit: Int, offset: Int): [Order]
}

type Mutation {
  createUser(name: String!, email: String!): User
  updateUser(id: ID!, name: String, email: String): User
  deleteUser(id: ID!): Boolean
}

type Subscription {
  userCreated: User
  orderPlaced: Order
}
```

#### Querying via GraphQL

```graphql
query {
  getUser(id: "123") {
    id
    name
    email
    orders {
      id
      total
      status
    }
  }
}

mutation {
  createUser(name: "Alice", email: "alice@example.com") {
    id
    name
  }
}

subscription {
  orderPlaced {
    id
    total
    user {
      name
      email
    }
  }
}
```

#### Custom Resolvers

```sql
-- Add custom resolver
create graphql resolver 'userOrders'
  for type 'User'
  as retrieve orders where user_id = $user.id

-- Add computed field
create graphql field 'fullName'
  for type 'User'
  as concat(first_name, ' ', last_name)
```

## Configuration

### Enabling Streaming Features

```toml
# /etc/minsql/minsql.conf

[streaming]
enable_continuous_queries = true
enable_cdc = true
enable_event_sourcing = true
enable_pubsub = true

# CDC settings
cdc_buffer_size = 10000
cdc_retention_hours = 72

# Pub/Sub settings
pubsub_max_channels = 1000
pubsub_message_history = 1000

# GraphQL settings
enable_graphql = true
graphql_port = 8080
graphql_playground = true
```

## Performance Considerations

### Continuous Queries
- Window size affects memory usage
- Smaller windows = more frequent output
- Use appropriate window type for use case

### CDC
- Minimal overhead (< 5%)
- Filtered subscriptions reduce network traffic
- Consider retention period for change log

### Event Sourcing
- Snapshots reduce rebuild time
- Balance snapshot frequency vs storage
- Consider archiving old events

### Pub/Sub
- Use appropriate channels for organization
- Clean up idle subscriptions
- Monitor message backlog

## Monitoring

### Streaming Metrics

```sql
-- Continuous query stats
show continuous query stats

-- CDC subscriber count
show cdc subscribers

-- Pub/Sub channel stats
show pubsub channels

-- Event store metrics
show event store stats
```

### Health Checks

```sql
-- Check streaming health
show streaming health

-- Output:
{
  "continuous_queries": {
    "active": 15,
    "total_events_processed": 1250000
  },
  "cdc": {
    "subscribers": 8,
    "changes_captured": 450000
  },
  "pubsub": {
    "channels": 25,
    "messages_sent": 890000
  }
}
```

## Best Practices

### Continuous Queries
1. Choose appropriate window sizes
2. Monitor query performance
3. Use output buffering for high-volume streams
4. Clean up unused queries

### CDC
1. Subscribe only to needed tables
2. Use filters to reduce noise
3. Handle duplicate events (at-least-once delivery)
4. Monitor subscriber lag

### Event Sourcing
1. Use meaningful event types
2. Include enough context in events
3. Create snapshots for large aggregates
4. Version your events

### Pub/Sub
1. Use descriptive channel names
2. Structure channels hierarchically
3. Don't overuse wildcard subscriptions
4. Clean up dead subscriptions

## Examples

### Real-Time Analytics Dashboard

```sql
-- Track active users
create continuous query active_users
  on table user_events
  window tumbling 1 minute
  as retrieve count(distinct user_id)
  output into dashboard_metrics

-- Monitor error rate
create continuous query error_rate
  on table api_logs
  window sliding 5 minutes step 1 minute
  as retrieve 
    count(*) filter (where status >= 500) as errors,
    count(*) as total
  output notify 'alerts'
```

### Audit Trail with CDC

```rust
// Subscribe to all table changes for audit
let mut audit_sub = db.subscribe_to_changes("*").await?;

while let Some(change) = audit_sub.recv().await {
    audit_log.write(AuditEntry {
        timestamp: change.timestamp,
        user: current_user(),
        table: change.table,
        operation: change.change_type,
        before: change.before,
        after: change.after,
    }).await?;
}
```

### Event-Sourced Order System

```sql
-- Place order
append event to orders
  aggregate_id 'order-123'
  event_type 'OrderPlaced'
  event_data '{
    "customer_id": "cust-456",
    "items": [...],
    "total": 250.00
  }'

-- Ship order
append event to orders
  aggregate_id 'order-123'
  event_type 'OrderShipped'
  event_data '{
    "tracking_number": "1Z999AA1..."
  }'

-- Rebuild order state
rebuild aggregate 'order-123' from orders
```

## Conclusion

minsql's streaming features enable you to build modern, reactive applications with real-time data processing, event sourcing, and pub/sub messaging—all within your database. This eliminates the need for external streaming platforms and simplifies your architecture.
