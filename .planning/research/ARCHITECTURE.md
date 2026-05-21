# Architecture: Testing & Documentation Infrastructure

**Project:** Octopus v1.2 Testing Coverage
**Domain:** Rust Async Distributed Systems Testing
**Researched:** 2026/05/21
**Confidence:** MEDIUM

---

## Executive Summary

Octopus is a Trino-style streaming MPP query engine using Tokio async runtime and DataFusion for query execution. The v1.2 milestone adds unit tests, integration tests, and Rustdoc comments to existing code. This document identifies integration points for tests, test utilities needed, and organization by existing crate boundaries.

The project currently has no test files despite `#[cfg(test)]` modules and `tokio-test` dev-dependencies. The codebase uses trait-based abstractions (FederatedConnector, ConnectionPool, UdfRegistry) which facilitate mocking, but lacks concrete mock implementations.

---

## Integration Points for Tests

### 1. DataFusion SessionContext (Coordinator)

**File:** `octopus-coordinator/src/query_service.rs`
**Line:** 93-96

```rust
let df = self.context.sql(sql).await
    .map_err(|e| format!("SQL parse error: {}", e))?;
```

**Test need:** Pre-configured `SessionContext` with known schemas and UDFs for integration testing.

### 2. Worker Registry (Coordinator)

**File:** `octopus-coordinator/src/scheduler.rs`
**Line:** 25

```rust
pub fn new(registry: Arc<WorkerRegistry>) -> Self
```

**Test need:** Controllable `WorkerRegistry` mock with static worker list and partition metadata for locality-aware scheduling tests.

### 3. Tokio Async Runtime (All crates)

**Pattern:** All async operations use Tokio.

**Test need:** `#[tokio::test]` infrastructure with isolated runtimes per test.

### 4. Arrow Flight Data Plane (Executor/Worker)

**Files:** `octopus-executor/src/flight_server.rs`, `flight_handler.rs`

**Test need:** Mock Flight endpoints or test server that returns predefined Arrow data.

### 5. Federated Connectors (Common)

**File:** `octopus-common/src/federated.rs`

```rust
pub trait FederatedConnector: Send + Sync {
    fn database_type(&self) -> DatabaseType;
    fn type_adapter(&self) -> Arc<dyn TypeAdapter>;
    fn connection_pool(&self) -> Arc<dyn ConnectionPool>;
    fn get_schema(&self, table: &str) -> crate::Result<Schema>;
    fn execute_query(&self, sql: &str) -> crate::Result<SendableRecordBatchStream>;
}
```

**Test need:** In-memory fake connectors that implement the trait without real database connections.

---

## Current State Assessment

### Crate Inventory

| Crate | Source Files | Existing Tests | Dev Dependencies |
|-------|--------------|----------------|-------------------|
| `octopus-common` | 4 | Yes (udf.rs only) | tokio |
| `octopus-coordinator` | 11 | None | None |
| `octopus-executor` | 8 | None | tokio-test |
| `octopus-worker` | 9 | None | tokio-test |

### Existing Test Code

Only `octopus-common/src/udf.rs` contains tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_registry_operations() { ... }
}
```

All other crates have `#[cfg(test)]` infrastructure but no test files.

---

## Test Organization by Crate

### octopus-common/

```
src/
├── udf.rs           # Has existing #[cfg(test)] module
├── federated.rs     # Trait definitions - needs mock implementations
└── error.rs         # Error type tests
```

**Test utilities to create:**
- `MockTypeAdapter` — fake type mapping for testing
- `MockConnectionPool` — in-memory connection pool
- `FakeFederatedConnector` — implements FederatedConnector without DB

### octopus-coordinator/

```
src/
├── query_service.rs  # Needs: TestQueryContext
├── scheduler.rs       # Needs: MockWorkerRegistry
├── stage_planner.rs  # Needs: StagePlannerTestUtils
├── worker_registry.rs
└── server.rs         # HTTP server - needs integration test harness

tests/
└── integration/
    └── coordinator_api_test.rs  # HTTP endpoint tests
```

**Test utilities to create:**
- `MockWorkerRegistry` — controllable worker list
- `TestQueryContext` — SessionContext with test UDFs
- `InMemoryTaskTracker` — mock for tracking test tasks

### octopus-executor/

```
src/
├── task_processor.rs  # Needs: MockTaskProcessor
├── runtime.rs         # Needs: MockRuntime
├── flight_server.rs   # Needs: TestFlightServer
└── flight_handler.rs

tests/
└── integration/
    └── executor_task_test.rs  # Task execution integration tests
```

**Test utilities to create:**
- `TestRecordBatchFactory` — generate Arrow test data
- `MockFlightClient` — test Flight client without real server
- `FakeTask` — controllable task for processor tests

### octopus-worker/

```
src/
├── main.rs
└── worker_service.rs

tests/
└── integration/
    └── worker_service_test.rs  # Worker startup/shutdown tests
```

---

## New Test Utilities Needed

### Priority 1: Coordinator Testing

| Utility | File | Purpose |
|---------|------|---------|
| `MockWorkerRegistry` | coordinator/tests/utils.rs | Controllable worker list for scheduler |
| `TestQueryContext` | coordinator/tests/utils.rs | Pre-configured SessionContext |
| `FakeWorkerInfo` | coordinator/tests/utils.rs | Worker with known partitions |

### Priority 2: Executor Testing

| Utility | File | Purpose |
|---------|------|---------|
| `TestRecordBatchFactory` | executor/tests/utils.rs | Generate Arrow test data |
| `MockFlightEndpoint` | executor/tests/utils.rs | Fake Flight server |
| `FakeTaskDefinition` | executor/tests/utils.rs | Task with controllable parameters |

### Priority 3: Common Testing

| Utility | File | Purpose |
|---------|------|---------|
| `InMemoryConnectionPool` | common/tests/utils.rs | Fake pool without DB |
| `MockTypeAdapter` | common/tests/utils.rs | Type mapping for tests |

---

## Recommended Test Patterns

### Pattern 1: Trait-Based Mocking

Components use async traits. Leverage this for mocking.

```rust
// octopus-common/src/federated.rs already defines
#[async_trait]
pub trait ConnectionPool: Send + Sync {
    async fn get(&self) -> crate::Result<Box<dyn std::any::Any + Send>>;
    async fn release(&self, conn: Box<dyn std::any::Any + Send>) -> crate::Result<()>;
    fn stats(&self) -> PoolStats;
}

// Create mock in tests
struct FakeConnectionPool { ... }

#[async_trait]
impl ConnectionPool for FakeConnectionPool {
    async fn get(&self) -> crate::Result<Box<dyn std::any::Any + Send>> {
        Ok(Box::new(FakeConnection))
    }
}
```

### Pattern 2: Constructor Injection

Dependencies passed via constructors enable test doubles.

```rust
// octopus-coordinator/src/scheduler.rs already does this
pub fn new(registry: Arc<WorkerRegistry>) -> Self

// Test with mock
let mock_registry = Arc::new(MockWorkerRegistry::new());
let scheduler = QueryScheduler::new(mock_registry);
```

### Pattern 3: Async Test Infrastructure

Use `#[tokio::test]` with isolated runtime.

```rust
// octopus-common/src/udf.rs shows the pattern
#[tokio::test]
async fn test_registry_operations() {
    let registry = UdfRegistryImpl::new();
    // test code
}
```

### Pattern 4: Channel-Based Component Testing

Test message passing with controlled channels.

```rust
#[tokio::test]
async fn test_task_processor() {
    let (tx, rx) = mpsc::channel(100);
    let processor = TaskProcessor::new(rx);
    tx.send(test_task()).await.unwrap();
    // verify behavior
}
```

---

## Anti-Patterns to Avoid

1. **No real database connections** — Use mocks instead of requiring PostgreSQL/MySQL
2. **No global mutable state** — Each test isolated with fresh instances
3. **No sleep-based timing** — Use mock clocks or channels instead
4. **No real S3/HDFS** — Use local file fixtures
5. **No hardcoded ports** — Use port 0 for auto-allocation in tests

---

## Phase Recommendations

### Phase 1: Test Infrastructure Foundations

**Focus:** Create test utilities and mock implementations

- Create `MockWorkerRegistry` for scheduler tests
- Create `TestQueryContext` for query service tests
- Create `InMemoryConnectionPool` for connector tests
- Create `TestRecordBatchFactory` for Arrow data generation

### Phase 2: Unit Tests for Core Components

**Focus:** Cover critical paths with unit tests

- `QueryService`: SQL parsing, query state machine, UDF registration
- `QueryScheduler`: Task creation, locality-aware assignment, round-robin fallback
- `TaskProcessor`: Task execution flow, error handling
- `FederatedConnector` implementations: Type mapping, query execution

### Phase 3: Integration Tests

**Focus:** Test component interaction

- HTTP API integration tests for coordinator endpoints
- Flight endpoint tests for executor data plane
- Federated connector mock tests with in-memory fake DB

### Phase 4: Documentation

**Focus:** Rustdoc for public APIs

- Document public types in coordinator, executor, worker
- Add usage examples to test utilities
- Document test organization for future contributors

---

## Scalability Considerations

| Test Type | 10 tests | 100 tests | 1000 tests |
|-----------|----------|-----------|------------|
| Unit | Fast | Fast | Parallel execution needed |
| Integration | Medium | Shared DB conflicts | Testcontainers or in-memory |
| Property-based | N/A | Useful for UDFs | Consider for data transformations |

---

## Architecture Summary

Testing Octopus requires infrastructure for:
1. **Async runtime testing** — `tokio-test` already in dev-dependencies
2. **Trait mocking** — Async traits already used (FederatedConnector, ConnectionPool)
3. **DataFusion integration** — SessionContext test harness needed
4. **Arrow data testing** — RecordBatch factory needed
5. **HTTP API testing** — Test client for coordinator endpoints

The project is well-structured for testing: traits are defined, dependencies are injected via constructors, and async patterns are consistent. The main gap is concrete mock implementations and test utilities.

---

## Sources

- Tokio testing: `tokio-test` documentation — HIGH confidence
- DataFusion testing: Integration with `SessionContext` — HIGH confidence
- Rust async testing patterns: Standard practice — HIGH confidence