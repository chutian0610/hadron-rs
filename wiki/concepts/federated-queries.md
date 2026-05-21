---
title: Federated Queries
type: concept
tags: [PostgreSQL,MySQL,connector,federated]
sources:
  - octopus-common/src/
  - octopus-executor/src/
related:
  - "[[Architecture]]"
  - "[[Execution]]"
---

# Federated Queries

Octopus supports federated query execution against external databases via a connector system. Currently, PostgreSQL and MySQL adapters are implemented.

## Federated Connector Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    QueryExecutor                        │
│                    (DataFusion)                         │
└───────────────────────┬─────────────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────────────┐
│              FederatedConnector Trait                   │
│              (octopus-common/src/)                      │
└───────┬─────────────────────────────────────┬──────────┘
        │                                     │
┌───────▼───────┐                    ┌────────▼────────┐
│ PostgreSQL    │                    │     MySQL       │
│ Connector     │                    │    Connector    │
└───────┬───────┘                    └────────┬────────┘
        │                                     │
┌───────▼───────┐                    ┌────────▼────────┐
│deadpool-postgres│                  │   mysql_async   │
└───────────────┘                    └─────────────────┘
```

## DatabaseType Enum

`octopus-common/src/federated.rs:1`

```rust
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
}
```

## TypeAdapter Trait

Defines the interface for federated database adapters:

```rust
pub trait TypeAdapter {
    fn database_type(&self) -> DatabaseType;
    async fn connect(&self, url: &str) -> Result<ConnectionPool>;
    async fn execute(&self, pool: &ConnectionPool, sql: &str) -> Result<RecordBatch>;
}
```

## ConnectionPool

Generic connection pool interface for managing database connections.

`octopus-common/src/connection_pool.rs:1`

### Supported Pool Implementations

- **PostgreSQL**: Uses `deadpool-postgres` for pooled connections
- **MySQL**: Uses `mysql_async` with async connection management

## PostgreSQL Connector

### Dependencies

- `deadpool-postgres`: Pooled PostgreSQL connections
- `tokio-postgres`: Async PostgreSQL driver

### Configuration

```rust
let pg_connector = PostgreSQLConnector::new();
let pool = pg_connector.connect("postgres://user:pass@host:5432/db").await?;
```

### Query Execution

```rust
let result = pg_connector.execute(&pool, "SELECT * FROM orders").await?;
```

## MySQL Connector

### Dependencies

- `mysql_async`: Async MySQL driver

### Configuration

```rust
let mysql_connector = MySQLConnector::new();
let pool = mysql_connector.connect("mysql://user:pass@host:3306/db").await?;
```

### Query Execution

```rust
let result = mysql_connector.execute(&pool, "SELECT * FROM users").await?;
```

## FederatedConnector Trait

`octopus-common/src/federated_connector.rs:1`

Main trait for all federated connectors:

```rust
pub trait FederatedConnector {
    fn database_type(&self) -> DatabaseType;
    async fn connect(&self, url: &str) -> Result<Self::ConnectionPool>;
    async fn execute(&self, pool: &Self::ConnectionPool, sql: &str) -> Result<RecordBatch>;
}
```

## Registering with DataFusion

Federated data sources are registered with DataFusion's table functions:

```rust
// Register PostgreSQL foreign table
ctx.sql("CREATE EXTERNAL TABLE orders 
    CONNECTION 'postgres://user:pass@host:5432/db'
    FROM postgres SELECT * FROM orders")?;
```

## Code References

- `octopus-common/src/federated.rs:1` - DatabaseType enum
- `octopus-common/src/connection_pool.rs:1` - ConnectionPool trait
- `octopus-common/src/federated_connector.rs:1` - FederatedConnector trait
