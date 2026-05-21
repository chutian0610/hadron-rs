# Feature Landscape: Rust Testing for Octopus v1.2

**Domain:** Unit/Integration Testing for Rust Distributed Query Engine
**Project:** Octopus (v1.2 milestone: test coverage and code comments)
**Researched:** 2026-05-21
**Overall confidence:** MEDIUM-HIGH (Rust official docs + established ecosystem patterns)

---

## Executive Summary

Rust testing follows a well-established pattern: unit tests co-located in `#[cfg(test)]` modules within source files, integration tests in `tests/` directory, and async tests via `#[tokio::test]`. The ecosystem provides mature tooling (mockall, tokio-test, proptest) for mocking and property-based testing. For Octopus, the testing challenge lies in async coordinator/executor code, DataFusion integration, and gRPC/Flight service mocking.

**Key finding:** Octopus currently has NO tests (confirmed via file search). The testing surface is straightforward: scheduler logic, query service state transitions, UDF registry, federated connector traits, and exchange sender/receiver pairs. Complexity is not in test patterns but in mocking async services and Arrow Flight data flows.

---

## Table Stakes (Expected in Well-Tested Rust Projects)

These features are standard Rust testing conventions. Missing them signals an incomplete or immature project.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Unit tests in `#[cfg(test)]` modules** | Standard Rust pattern. Tests live next to code they test, in same module. | LOW | Use `use super::*` to access parent items |
| **`#[test]` attribute on test functions** | Cargo convention. Functions annotated `#[test]` become test cases. | LOW | Tests that panic fail; returning `Ok(())` passes |
| **`#[tokio::test]` for async tests** | Tokio provides test runtime for async code. | LOW-MED | Required for any `async fn` test; uses multi-thread runtime |
| **Integration tests in `tests/` directory** | Cargo convention. Each `.rs` file in `tests/` compiles as separate test binary. | LOW | Use `use crate_name::*` to access library items |
| **`#[should_panic]` for expected panics** | Verify error paths fail with expected messages. | LOW | Use `expected = "substring"` for precise matching |
| **`assert_eq!`/`assert_ne!` with `PartialEq` + `Debug`** | Standard assertions on custom types. | LOW | Always derive both traits on test-supporting types |
| **Mocking with `#[automock]`** | Trait mocking via mockall. Generates `Mock*` structs from traits. | LOW-MED | Place on trait definitions; use `MockStruct::new()` |
| **`#[ignore]` for flaky/slow tests** | Mark tests to skip in normal runs. CI runs with `--ignored`. | LOW | Use for integration tests hitting real services |
| **Custom failure messages in assertions** | Add context to assertion failures. | LOW | `assert!(cond, "expected X but got Y")` |

**Confidence: HIGH** — These are Rust book-standard patterns, well-documented at doc.rust-lang.org.

---

## Differentiators (What Sets Professional Projects Apart)

These elevate code quality and signal engineering maturity. Not required for v1.2 but recommended for long-term.

| Feature | Value Proposition | Complexity | When to Use |
|---------|-------------------|------------|-------------|
| **Property-based testing (proptest)** | Generates hundreds of random inputs automatically. Catches edge cases human testers miss. | MED | Critical algorithms: hashing, serialization, scheduling heuristics |
| **Code coverage tracking (cargo-llvm-cov)** | Visual HTML report shows which lines are exercised. Target >80% for core logic. | LOW | Run `cargo llvm-cov --html` after tests exist |
| **Mutation testing (cargo-mutant)** | Injects bugs to verify tests catch them. Validates test quality. | HIGH | After basic tests exist; too noisy for early stage |
| **Criterion benchmarks (criterion-rs)** | Measures performance regression in CI. | MED | Critical paths: query parsing, exchange serialization, scheduler hot path |
| **State machine testing (proptest-state)** | Model-based state transition testing. | MED-HIGH | Query state machine, task lifecycle, connection pool transitions |
| **Mockall static method mocking** | Handle `fn static_method(&self)` vs `fn static_method()` differently via `Context`. | MED | Required for factory traits with static methods |
| **`tokio-test`'s `MockStream` for async** | Test async streams without real I/O. | MED | Exchange sender/receiver pairs |
| **Doc tests in Rustdoc comments** | Examples in `///` comments become tests. | LOW | Add `assert_eq!` to existing Rustdoc examples |

**Confidence: MEDIUM** — Advanced tooling, well-documented, adoption varies by project maturity.

---

## Anti-Features (Explicitly NOT Building for v1.2)

Features that are out-of-scope, inappropriate for Octopus, or require infrastructure not available in v1.2.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **cargo-insta (snapshot testing)** | Good for UI/text output. Not relevant for query engine state/logic. | Use explicit `assert_eq!` on structured types |
| **rstest (parameterized tests)** | Overkill for Octopus. Simple `#[test]` with loop is sufficient. | Loop over test cases: `for case in [...] { assert!(...) }` |
| **testcontainers for real DB** | Heavy dependency, requires Docker daemon. Out of scope for unit tests. | Mock `ConnectionPool` trait for unit tests; integration tests can use testcontainers later |
| **Golden file tests for query plans** | Fragile. Query plan text changes with DataFusion optimizer versions. | Test plan structure (stages, partitions) not exact text output |
| **Load/stress testing** | Requires multi-node cluster. Out of scope for unit/integration testing. | Defer to Phase 5 (Fault Tolerance) |
| **fuzz testing (cargo-fuzz)** | Requires stable SQL parser surface. Too early for v1.2. | Defer to v1.3 when SQL surface is stable |

**Confidence: HIGH** — Based on Octopus milestone scope (v1.2 = test coverage, not new features).

---

## Octopus Crate-Specific Test Surface

Each crate has distinct testing needs based on its responsibilities. No existing tests found in any crate.

### `octopus-common`

| Module | Test Surface | Priority | Complexity | Dependencies |
|--------|--------------|----------|------------|--------------|
| `udf.rs` | UDF registration, listing, removal | HIGH | LOW | None (isolated) |
| `federated.rs` | TypeAdapter, ConnectionPool, FederatedConnector traits | HIGH | LOW-MED | Mock implementations via mockall |
| `error.rs` | Error conversion, error chain | MED | LOW | None |

**Recommended approach:**
- `UdfRegistry` tests: Direct instantiation, register/list/unregister with mock scalar UDFs
- Trait tests: Implement `#[automock]` on traits, create mock structs for testing other modules

### `octopus-coordinator`

| Module | Test Surface | Priority | Complexity | Dependencies |
|--------|--------------|----------|------------|--------------|
| `scheduler.rs` | Task creation, locality-aware assignment, completion | HIGH | LOW | None (pure async logic) |
| `query_service.rs` | State transitions (Received→Planning→Planned→Executing→Completed/Failed) | HIGH | LOW-MED | Mock QueryScheduler with mockall |
| `stage_planner.rs` | Stage DAG creation, pipeline breaker detection | MED | MED | DataFusion `LogicalPlan` fixtures |
| `task_tracker.rs` | Task state tracking, rescheduling decisions | MED | MED | Mock worker registry |
| `exchange_operator.rs` | Exchange operator insertion in plans | MED | MED | DataFusion plan comparison |
| `worker_registry.rs` | Worker registration, listing, heartbeat | MED | LOW | None (in-memory HashMap) |
| HTTP handlers | `/query/submit`, `/query/explain`, `/query/state/{id}` | MED | MED-HIGH | `axum` test utilities or `tower::service::mock` |

**Recommended approach:**
- `scheduler.rs`: Standard unit tests with `use super::*`
- `query_service.rs`: `#[tokio::test]` with `MockQueryScheduler` via mockall
- HTTP handlers: Use `axum` companion crate `axum-testing` or raw tower service mocking

### `octopus-executor`

| Module | Test Surface | Priority | Complexity | Dependencies |
|--------|--------------|----------|------------|--------------|
| `query.rs` | Query execution with DataFusion | MED | MED | DataFusion `SessionContext` with test object store |
| `datasource.rs` | DataSource trait implementation | MED | MED | Mock `ObjectStore` or test files |
| `exchange_receiver.rs` | Arrow Flight data receiving | MED | MED | `#[tokio::test]` with in-memory channel |
| `exchange_sender.rs` | Arrow Flight data sending | MED | MED | `#[tokio::test]` with in-memory channel |
| `session.rs` | Session state management | MED | LOW | None (simple state) |
| `logging.rs` | Logging configuration | LOW | LOW | None |

**Recommended approach:**
- `query.rs`: Use DataFusion's built-in test utilities; test against Parquet/CSV in temp directory
- Exchange sender/receiver: Use `tokio::sync::mpsc` channels as stand-ins for Arrow Flight streams

### `octopus-worker`

| Module | Test Surface | Priority | Complexity | Dependencies |
|--------|--------------|----------|------------|--------------|
| Task execution | Worker task loop, task polling | MED | MED-HIGH | Mock coordinator client |
| Flight server | Arrow Flight server startup/shutdown | MED | MED | `#[tokio::test]` with in-memory transport |

**Recommended approach:** Mock `CoordinatorClient` with mockall; test task execution in isolation

### `octopus-cli`

| Module | Test Surface | Priority | Complexity | Dependencies |
|--------|--------------|----------|------------|--------------|
| CLI argument parsing | clap derive parsing | LOW | LOW | Standard clap test patterns |
| REPL mode | Command parsing, history | LOW | MED | Mock coordinator client |

---

## Feature Dependencies

```
Phase 1: Infrastructure
    │
    ├─► mockall dependency added to Cargo.toml dev-dependencies
    │
    ├─► tokio-test dependency added
    │
    └─► PartialEq + Debug derives added to test-supporting types

Phase 2: Unit Tests
    │
    ├─► octopus-common tests (udf, federated traits)
    │
    ├─► octopus-coordinator tests (scheduler, query_service)
    │
    ├─► octopus-executor tests (query execution basics)
    │
    └─► Integration test files in tests/ directory

Phase 3: Advanced Testing (defer to v1.3+)
    │
    ├─► property-based tests for scheduler heuristics
    │
    ├─► criterion benchmarks for hot paths
    │
    └─► coverage tracking with cargo-llvm-cov
```

---

## MVP Recommendation for v1.2

**Priority order (by dependency and risk):**

### Tier 1 (Foundational - start here)

1. **`octopus-coordinator::scheduler`** — Task creation, locality scoring, round-robin fallback
   - Complexity: LOW
   - Pattern: `#[cfg(test)] mod tests { use super::*; #[test] fn ... }`
   - No mocking needed: pure logic with in-memory HashMap

2. **`octopus-common::udf`** — UDF registry operations
   - Complexity: LOW
   - Pattern: Direct instantiation with mock `ScalarUDF` (use DataFusion's `lit()` as test UDF)

### Tier 2 (State management - depends on Tier 1)

3. **`octopus-coordinator::query_service`** — State machine transitions
   - Complexity: LOW-MED
   - Pattern: `#[tokio::test]` with `#[automock]` on `QueryScheduler` trait
   - Use mockall to create `MockQueryScheduler` for controlled testing

4. **`octopus-coordinator::task_tracker`** — Task state tracking
   - Complexity: MED
   - Pattern: `#[tokio::test]` with mock worker registry

### Tier 3 (Trait-based - requires mocking infrastructure)

5. **`octopus-common::federated`** — Connector trait tests
   - Complexity: MED (trait mocking)
   - Pattern: `#[automock]` on `TypeAdapter`, `ConnectionPool`, `FederatedConnector`
   - Implement `MockConnectionPool` for testing other modules

### Tier 4 (Integration - depends on all above)

6. **HTTP API integration tests** — Coordinator endpoints
   - Complexity: MED-HIGH
   - Pattern: `axum::test::TestClient` or `tower::service::mock`
   - Test: submit query, explain query, get query state

7. **Exchange sender/receiver tests** — Arrow Flight data flow
   - Complexity: MED
   - Pattern: `#[tokio::test]` with `tokio::sync::mpsc` as in-memory transport

**Defer to v1.3:**
- Property-based testing (scheduler heuristics need to stabilize first)
- Criterion benchmarks (require stable hot paths)
- Mutation testing (requires existing test coverage to be meaningful)
- Fuzz testing (SQL surface not yet stable)

---

## Rustdoc Expectations for Octopus

Well-documented Rust projects have Rustdoc on all public items. Octopus already has some Rustdoc (confirmed in `federated.rs`).

**Standard Rustdoc pattern:**
```rust
/// Query service handles SQL submission, planning, and state management.
///
/// # State Machine
/// Queries transition through: Received → Planning → Planned → Executing → Completed/Failed
///
/// # Example
/// ```
/// # use octopus_coordinator::{QueryService, QueryState};
/// # async fn example() {
///     let scheduler = Arc::new(RwLock::new(QueryScheduler::new(registry)));
///     let service = QueryService::new(scheduler);
///     let query_id = service.submit_query("SELECT * FROM tbl").await?;
///     assert_eq!(service.get_query_state(&query_id).await, Some(QueryState::Received));
/// # }
/// ```
pub struct QueryService { ... }
```

**v1.2 Rustdoc targets:**
- All `pub struct`, `pub enum`, `pub fn` in coordinator and executor crates
- Key trait definitions in `federated.rs` (already documented)
- Module-level `//!` docs for each source file

---

## Sources

### Primary (HIGH confidence)
- [The Rust Programming Language - Testing Chapter](https://doc.rust-lang.org/book/ch11-00-testing.html) — Official testing guide
- [The Rust Programming Language - Writing Tests (ch11-01)](https://doc.rust-lang.org/book/ch11-01-writing-tests.html) — `#[test]`, assertions, `#[should_panic]`
- [Tokio Testing Documentation](https://docs.rs/tokio/latest/tokio/#testing) — `#[tokio::test]` and test utilities
- [Mockall Documentation](https://docs.rs/mockall/latest/mockall/) — `#[automock]` for trait mocking

### Secondary (MEDIUM confidence)
- [tokio-test crate](https://docs.rs/tokio-test/latest/tokio_test/) — Async test utilities (`MockStream`, `assert_stream_eq`)
- [proptest crate](https://docs.rs/proptest/latest/proptest/) — Property-based testing
- [cargo-llvm-cov](https://docs.rs/cargo-llvm-cov/latest/cargo_llvm_cov/) — Code coverage tracking
- [axum testing guide](https://docs.rs/axum/latest/axum/#testing) — HTTP handler testing patterns

### Tertiary (LOW confidence - inferential)
- [DataFusion/Ballista test patterns](https://github.com/apache/arrow-datafusion) — How production Rust query engines test

---

## Confidence Assessment

| Area | Confidence | Reason |
|------|------------|--------|
| Table stakes (std Rust patterns) | HIGH | Rust testing is well-standardized, documented |
| Octopus-specific test surface | MEDIUM-HIGH | Code inspection confirms modules exist; no existing tests |
| Complexity estimates | MEDIUM | Based on similar async/gRPC projects |
| Tooling recommendations | MEDIUM | Ecosystem mature; specific tool choices may vary by team |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address During Implementation

- **gRPC/Flight service mocking**: Specific patterns for testing tonic services and Arrow Flight servers need spike during implementation
- **DataFusion test helpers**: How to set up `SessionContext` for testing without hitting production object stores (use tempdir + test Parquet files)
- **Test HTTP server setup**: `axum::test::TestClient` vs tower `MockSvc` vs spinning up real server on random port
- **Mock coordinator client for worker tests**: Need to define `#[automock]` trait for `CoordinatorClient` to test worker in isolation