# minsql Query Language

## Philosophy

minsql uses an intent-driven query language. Instead of parsing rigid SQL syntax, queries are parsed into semantic intent that describes what you want to accomplish.

This approach allows:
- Optimizer operates on intent, not syntax
- Language can evolve without breaking changes
- Better error messages tied to intent violations
- Multiple surface syntaxes can map to the same intent

## Basic Syntax

### Retrieving Data

```
retrieve users
```

Retrieves all rows from the `users` table.

```
retrieve users
where age > 18
```

Filters results based on a condition.

```
retrieve users
where age > 18
limit 10
```

Limits the number of results.

### Projection

```
retrieve name, email from users
where active = true
```

Specifies which columns to return.

### Ordering

```
retrieve users
where age > 18
order by created_at desc
limit 10
```

Orders results by one or more columns.

### Aggregation

```
retrieve count(*), avg(age) from users
where active = true
```

Computes aggregates over rows.

```
retrieve department, count(*), avg(salary)
from employees
where hire_date > '2020-01-01'
group by department
```

Groups rows before aggregating.

### Joins

```
retrieve users.name, orders.total
from users
join orders on users.id = orders.user_id
where orders.created_at > '2024-01-01'
```

Joins multiple tables.

```
retrieve users.name, orders.total
from users
left join orders on users.id = orders.user_id
```

Outer joins are supported.

### Inserting Data

```
insert into users (name, email, age)
values ('Alice', 'alice@example.com', 30)
```

Inserts a single row.

```
insert into users (name, email, age)
values 
  ('Alice', 'alice@example.com', 30),
  ('Bob', 'bob@example.com', 25)
```

Inserts multiple rows.

### Updating Data

```
update users
set age = 31
where name = 'Alice'
```

Updates matching rows.

```
update users
set age = age + 1, updated_at = now()
where active = true
```

Updates can use expressions.

### Deleting Data

```
delete from users
where age < 18
```

Deletes matching rows.

### Time-Travel Queries

```
retrieve users
where age > 18
at timestamp '2024-11-10 12:03:21'
```

Queries data as it existed at a specific point in time.

```
retrieve users
where age > 18
at timestamp '2024-11-10 12:03:21'
until timestamp '2024-11-15 14:30:00'
```

Queries data across a time range.

## Schema Operations

### Creating Tables

```
create table users (
  id bigint primary key,
  name text not null,
  email text unique not null,
  age integer,
  active boolean default true,
  created_at timestamp default now()
)
```

Defines a table schema.

### Indexes

```
create index users_email_idx on users (email)
```

Creates an index.

```
create index users_age_active_idx on users (age, active)
where active = true
```

Creates a partial index.

### Altering Tables

```
alter table users add column last_login timestamp
```

Adds a column.

```
alter table users drop column age
```

Removes a column.

### Dropping Tables

```
drop table users
```

Removes a table.

## Transactions

### Explicit Transactions

```
begin transaction
  update accounts set balance = balance - 100 where id = 1
  update accounts set balance = balance + 100 where id = 2
commit
```

Executes multiple operations atomically.

```
begin transaction
  update accounts set balance = balance - 100 where id = 1
  update accounts set balance = balance + 100 where id = 2
rollback
```

Aborts a transaction.

### Isolation Levels

```
begin transaction isolation level serializable
  retrieve accounts where id = 1
  update accounts set balance = balance - 100 where id = 1
commit
```

Specifies transaction isolation level.

## Advanced Features

### Subqueries

```
retrieve name, email
from users
where id in (
  retrieve user_id from orders where total > 1000
)
```

Uses a subquery in a filter.

```
retrieve name, email, (
  retrieve count(*) from orders where orders.user_id = users.id
) as order_count
from users
```

Uses a subquery in projection.

### Common Table Expressions

```
with active_users as (
  retrieve id, name from users where active = true
)
retrieve name
from active_users
where id > 100
```

Defines a named subquery.

### Window Functions

```
retrieve name, salary,
  rank() over (partition by department order by salary desc) as salary_rank
from employees
```

Computes window functions.

## Resource Limits

### Query Sandboxing

```
retrieve users
where age > 18
with max_time = 5s, max_memory = 100MB
```

Enforces resource limits on the query.

```
retrieve users
where age > 18
with priority = low
```

Sets query priority.

## Deterministic Mode

### Enabling Determinism

```
begin deterministic transaction
  retrieve users order by id
commit
```

Executes a transaction in deterministic mode. Results and timing will be identical across executions.

## Data Types

### Supported Types

- `boolean`: true/false values
- `integer`: 32-bit signed integers
- `bigint`: 64-bit signed integers
- `real`: 32-bit floating point
- `double`: 64-bit floating point
- `text`: Variable-length strings
- `bytea`: Binary data
- `timestamp`: Date and time with timezone
- `date`: Calendar date
- `time`: Time of day
- `json`: JSON documents

### Type Casting

```
retrieve id, cast(age as text) from users
```

Explicitly converts between types.

## Comments

```
-- This is a single-line comment
retrieve users where age > 18
```

```
/* This is a
   multi-line comment */
retrieve users where active = true
```

## Error Handling

The language provides clear error messages tied to semantic intent:

- `IntentViolation`: The query intent cannot be satisfied
- `SchemaViolation`: Schema constraints are violated
- `ResourceExceeded`: Query exceeded resource limits
- `SyntaxError`: Query syntax is invalid
- `TypeMismatch`: Expression types are incompatible

## Best Practices

### Use Explicit Column Names

```
retrieve name, email from users
```

Better than:

```
retrieve * from users
```

### Filter Early

```
retrieve name
from users
where active = true and age > 18
```

Filters reduce data volume early in the plan.

### Use Indexes

```
create index users_email_idx on users (email)
```

Indexes accelerate lookups.

### Limit Result Sets

```
retrieve users
where active = true
limit 100
```

Prevents accidentally retrieving massive result sets.

### Use Transactions for Multi-Statement Operations

```
begin transaction
  update inventory set quantity = quantity - 1 where product_id = 5
  insert into orders (product_id, quantity) values (5, 1)
commit
```

Ensures atomicity.

## Migration from SQL

While minsql is not SQL-compatible, the syntax is familiar:

| SQL | minsql |
|-----|--------|
| `SELECT * FROM users` | `retrieve users` |
| `SELECT name, email FROM users WHERE age > 18` | `retrieve name, email from users where age > 18` |
| `INSERT INTO users (name) VALUES ('Alice')` | `insert into users (name) values ('Alice')` |
| `UPDATE users SET age = 30 WHERE id = 1` | `update users set age = 30 where id = 1` |
| `DELETE FROM users WHERE age < 18` | `delete from users where age < 18` |

The mental model is similar, but the parser operates on intent rather than rigid grammar.
