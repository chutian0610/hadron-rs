# Phase 3: Unit Tests - Research

**Researched:** 2026-05-22
**Domain:** Rust unit testing patterns for distributed query engine
**Confidence:** HIGH (implementation files read, dependencies verified)

## Summary

Phase 3 implements comprehensive unit tests for Octopus core components following a "maximal real, minimal mock" strategy. The key architectural decision is using real DataFusion SessionContext for SQL execution while mocking external dependencies (workers, Flight transport). QueryScheduler partition locality scoring is the most algorithmically complex component requiring careful test design. QueryService state machine tests need explicit coverage of all 6 state transitions. Exchange operators are currently placeholder markers that return errors when executed locally - the actual sender/receiver implementation is deferred.

**Primary recommendation:** Focus scheduler tests on partition scoring algorithm verification using MockWorkerRegistry with counter-based predictable UUIDs. State machine tests should use tower-test/TestClient for HTTP layer validation. Exchange backpressure testing is premature until the sender/receiver transport is implemented.

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **TEST-FIXTURE-01:** Minimal constants for relational test data + TestRecordBatchFactory for variants
- **TEST-MOCK-01:** Mock workers/Flight only; real DataFusion SessionContext
- **TEST-REAL-01:** Real DataFusion + in-memory; no external files or DBs

### Claude's Discretion
- Coverage targets (line vs critical path)
- Error path prioritization
- Test organization within source files

### Deferred Ideas (OUT OF SCOPE)
- External fixture files for complex multi-table schemas
- Integration tests for MySQL/Postgres connectors

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| QueryScheduler task scheduling | Coordinator | — | Creates/assigns tasks, owns pending_tasks |
| Partition locality scoring | Coordinator | — | Algorithm in find_best_worker, workers are external |
| QueryService state machine | Coordinator | — | Manages query lifecycle states |
| HTTP API endpoints | Coordinator | — | Axum server layer, tested via tower-test |
| ExecutorSession query execution | Executor | — | Wraps SessionContext for local execution |
| Exchange operator (placeholder) | Coordinator | — | Currently returns error, not fully implemented |
| UDF registry | Common | Coordinator | Shared trait, used by both |
| OctopusError | Common | All | Error types span all crates |

## Standard Stack

### Core Testing Dependencies
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio-test | 0.4.5 | Async test runtime | First-party Tokio testing |
| mockall | 0.13.1 | Trait mocking | Standard for Rust interface mocking |
| tower-test | 0.4.0 | HTTP service testing | Part of Tower ecosystem for axum |
| tempfile | 3.0 | Temp files for integration | Standard for Rust temp file handling |

**Installation:** Already configured in workspace dev-dependencies

### Reusable Test Assets (from Phase 2)
| Asset | Path | Purpose |
|-------|------|---------|
| MockWorkerRegistry | octopus-common/src/test_utils/mock.rs | Controllable worker list with predictable UUIDs |
| TestRecordBatchFactory | octopus-common/src/test_utils/fixture.rs | Arrow RecordBatch builder |
| async_with_timeout | octopus-common/src/test_utils/timeout.rs | Prevent hanging tests (30s default) |
| TestQueryContext | octopus-coordinator/src/test_utils.rs | Pre-configured SessionContext |

## Architecture Patterns

### Project Structure for Tests
```
octopus-coordinator/src/
├── scheduler.rs       # QueryScheduler + #[cfg(test)] mod tests
├── query_service.rs  # QueryService + #[cfg(test)] mod tests
├── exchange_operator.rs
├── server.rs         # HTTP server (coordination layer)
└── test_utils.rs     # TestQueryContext

octopus-executor/src/
├── session.rs        # ExecutorSession + #[cfg(test)] mod tests
└── query.rs          # QueryExecutor + #[cfg(test)] mod tests

octopus-common/src/
├── udf.rs            # UdfRegistry + #[cfg(test)] mod tests
├── federated.rs      # FederatedConnector trait (no impl)
└── error.rs          # OctopusError (no impl, is error type)
```

### Pattern 1: QueryScheduler Partition Locality Testing

**What:** Test the `find_best_worker` locality scoring algorithm and round-robin fallback

**When to use:** COORD-01 scheduler tests

**Approach:**
1. Create MockWorkerRegistry with pre-configured partitions per worker
2. Create tasks with `required_partitions` matching specific workers
3. Call `assign_task()` and verify correct worker is selected
4. Verify round-robin fallback when no locality info

**Key test cases:**
- No workers available → returns None
- Single worker, no locality → round-robin assignment
- Multiple workers, one with matching partitions → selects best locality match
- Multiple workers with ties → selects first (iteration order)
- Empty required_partitions → triggers round-robin, increments counter
- Non-empty required_partitions → triggers locality scoring, no counter increment

```rust
// Pseudocode structure
#[cfg(test)]
mod tests {
    use super::*;
    use octopus_common::test_utils::mock::MockWorkerRegistry;

    #[tokio::test]
    async fn test_locality_scoring_prefers_best_match() {
        let registry = MockWorkerRegistry::new();
        // Setup: worker-a has partitions [p1, p2], worker-b has [p2, p3]
        registry.add_worker(worker_with_partitions("worker-a", vec!["p1", "p2"])).await;
        registry.add_worker(worker_with_partitions("worker-b", vec!["p2", "p3"])).await;

        let scheduler = QueryScheduler::new(Arc::new(registry));
        let task = scheduler.create_task("q1", 0, 0, vec!["p1".to_string()]).await;

        // Worker-a has p1, worker-b does not → worker-a should be selected
        let assigned = scheduler.assign_task(&task.task_id).await;
        assert_eq!(assigned, Some("worker-a".to_string()));
    }
}
```

### Pattern 2: QueryService State Machine Testing

**What:** Test all valid state transitions and invalid operation handling

**When to use:** COORD-02 state machine tests

**State Transition Diagram:**
```
Received → Planning → Planned → Executing → Completed
                            ↘ Failed
```

**Test cases needed:**
| From State | Operation | To State | Valid? |
|------------|-----------|----------|--------|
| (initial) | submit_query | Received | Yes |
| Received | plan_query | Planning | Yes |
| Planning | (plan created) | Planned | Yes |
| Planned | (execution starts) | Executing | Yes |
| Executing | (success) | Completed | Yes |
| Executing | (error) | Failed | Yes |
| Received | plan_query | — | No (wrong state) |
| Completed | plan_query | — | No (terminal) |
| Failed | plan_query | — | No (terminal) |
| Non-existent query | get_query_state | None | Yes (returns None) |

**Key insight:** State transitions are triggered by async methods that take ownership of `query.logical_plan`. Tests should verify:
1. Valid transitions succeed and state updates correctly
2. Invalid transitions return errors without state mutation
3. Terminal states (Completed, Failed) reject further operations

### Pattern 3: tower-test HTTP Endpoint Testing

**What:** Use `tower_test::Task` and `tower_test::assert_response` for HTTP API testing

**When to use:** COORD-03 HTTP endpoint tests

**Pattern:**
```rust
use tower_test::Task;
use axum::http::Request;

// Create app with routes
let app = axum::Router::new()
    .route("/query/submit", post(submit_query))
    .route("/query/state/{id}", get(get_state))
    .route("/query/explain", post(explain_query));

// Use tower-test to mock HTTP client
tower_test::Task::new(app)
    .expect_request(Request::builder().uri("/query/submit").body(body).build())
    .await;
```

**Important:** The current `server.rs` only has `CoordinatorServer` struct (gRPC wrapper), not the actual HTTP router. HTTP endpoints are defined in `main.rs` as inline handlers. COORD-03 tests need to extract the HTTP handler logic for isolated testing.

### Pattern 4: ExchangeOperator Backpressure Testing

**What:** Exchange operators define stage boundaries; actual sender/receiver not yet implemented

**Current state:** `exchange_operator.rs` ExchangeOperator returns error on `execute()`:
```rust
fn execute(...) -> ExecResult<SendableRecordBatchStream> {
    Err(datafusion::common::DataFusionError::Execution(
        "Exchange operator must be executed distributed".to_string()
    ))
}
```

**Implication:** EXEC-02 exchange operator tests cannot test actual stream backpressure until the distributed sender/receiver transport is implemented. Current focus should be:
1. Testing `ExchangeMode` enum variants and `partition_count()`
2. Testing `to_stage_partitioning()` conversion
3. Testing `create_exchange_ticket()` formatting
4. Testing `with_new_children()` validation (requires exactly 1 child)

**Deferred:** Actual backpressure stream tests (sender/receiver implementation is a future phase)

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Worker registry for tests | New implementation | MockWorkerRegistry | Predictable UUIDs, partition setup, already built in Phase 2 |
| Arrow test data | Manual RecordBatch construction | TestRecordBatchFactory | Builder pattern, type safety, boundary variant support |
| Async test timeouts | ad-hoc sleep/spawn | async_with_timeout | Consistent 30s default, structured error handling |
| HTTP service testing | Real HTTP server | tower-test/TestClient | In-process, no port binding, assertions |
| Trait mocking | Manual fake implementations | mockall | Compile-time verification, flexible expectations |
| SessionContext for tests | new() each time | TestQueryContext | Pre-configured with UDF registry support |

## Common Pitfalls

### Pitfall 1: Testing Private Implementation Details
**What goes wrong:** Tests break when internal refactoring occurs
**How to avoid:** Test via public API (public methods, trait bounds). If implementation-specific tests are needed, mark them `#[ignore]` with explanation.

### Pitfall 2: Non-Deterministic Scheduler Tests
**What goes wrong:** Round-robin counter or iteration order causes flaky tests
**How to avoid:** Use MockWorkerRegistry with `add_worker()` for deterministic setup. Pre-set worker IDs and partitions. Verify algorithm behavior, not incidental ordering.

### Pitfall 3: Forgetting State Machine Edge Cases
**What goes wrong:** Only test happy paths, miss error handling verification
**How to avoid:** Explicit transition matrix coverage. Terminal states (Completed, Failed) must reject subsequent operations.

### Pitfall 4: Mismatched RecordBatch Factory Usage
**What goes wrong:** TestRecordBatchFactory with wrong-length columns builds empty batches silently
**How to avoid:** Use `build().unwrap()` for immediate failure. Add explicit row count assertions.

### Pitfall 5: Testing Against Implementation Rather Than Behavior
**What goes wrong:** Tests assert on internal state (pending_tasks count) rather than observable outcomes
**How to avoid:** Assert on return values and external observable behavior. Internal state inspection is for debugging, not validation.

## Code Examples

### Testing QueryScheduler Locality Scoring
```rust
// From octopus-coordinator/src/scheduler.rs lines 84-104
// find_best_worker algorithm:
//   - Iterates workers, counts matching partitions
//   - Returns worker with highest score
//   - Ties broken by iteration order (first encountered)

#[tokio::test]
async fn test_find_best_worker_locality() {
    let registry = MockWorkerRegistry::new();
    // Worker A: has p1, p2
    // Worker B: has p2, p3
    // Task requires p1 → Worker A should win (score 1 vs 0)
}
```

### Testing QueryService State Transitions
```rust
// From octopus-coordinator/src/query_service.rs lines 49-57
// QueryState enum: Received, Planning, Planned, Executing, Completed, Failed

#[tokio::test]
async fn test_query_state_transitions() {
    let service = QueryService::new(...);
    let query_id = service.submit_query("SELECT 1").await.unwrap();
    assert_eq!(service.get_query_state(&query_id).await, Some(QueryState::Received));

    let plan = service.plan_query(&query_id).await.unwrap();
    assert_eq!(service.get_query_state(&query_id).await, Some(QueryState::Planned));
}
```

### Testing OctopusError Variants
```rust
// From octopus-common/src/error.rs
// OctopusError variants: SqlError, DataSourceError, ExecutionError,
//   ParseError, IoError (from std::io::Error), ObjectStoreError, ConnectionPoolError

#[test]
fn test_octopus_error_display() {
    let err = OctopusError::SqlError("syntax error".to_string());
    assert_eq!(err.to_string(), "SQL execution error: syntax error");
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual fake implementations | mockall + MockWorkerRegistry | Phase 2 | Deterministic, typed mocks |
| Inline test data | TestRecordBatchFactory builder | Phase 2 | Reusable, variant generation |
| ad-hoc async timeouts | async_with_timeout helpers | Phase 2 | Consistent timeout handling |
| Real HTTP server for tests | tower-test | Phase 3 | In-process, faster tests |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Exchange sender/receiver not yet implemented (only placeholder) | Pattern 4 | If implementation exists, EXEC-02 scope is larger |
| A2 | HTTP router defined inline in main.rs, not in server.rs | Pattern 3 | COORD-03 tests may need refactoring to extract handlers |
| A3 | MockWorkerRegistry from Phase 2 compatible with WorkerRegistry trait | Don't Hand-Roll | If trait interface changed, mock may not compile |

## Open Questions

1. **Exchange distributed execution**
   - What we know: ExchangeOperator.execute() returns error indicating distributed execution required
   - What's unclear: When will sender/receiver transport be implemented? Is EXEC-02 meant to test something else?
   - Recommendation: Proceed with EXEC-02 focused on ExchangeMode, ticket creation, and child validation. Mark backpressure tests as deferred.

2. **HTTP handler extraction**
   - What we know: HTTP endpoints in main.rs as inline handlers
   - What's unclear: Will tests require main.rs changes to expose handlers for testing?
   - Recommendation: COORD-03 plan should include handler extraction task if not already modularized

3. **WorkerRegistry trait for mockall**
   - What we know: Real WorkerRegistry is concrete struct, not a trait
   - What's unclear: Should WorkerRegistry be refactored to a trait for mockall compatibility, or continue using MockWorkerRegistry composition?
   - Recommendation: Use MockWorkerRegistry directly as the mock implementation. If mockall mocking of the real interface is needed, propose trait extraction in this phase.

## Environment Availability

Step 2.6: SKIPPED (no external dependencies identified - testing uses only in-workspace crates and standard Rust test tooling)

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | tokio-test 0.4.5 + mockall 0.13.1 + tower-test 0.4.0 |
| Config file | None — use #[cfg(test)] modules |
| Quick run command | `cargo test -p <crate> -- --test-threads=1` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Test Location |
|--------|----------|-----------|---------------|
| COORD-01 | QueryScheduler task scheduling | unit | scheduler.rs::tests |
| COORD-02 | QueryService state machine | unit | query_service.rs::tests |
| COORD-03 | HTTP API endpoints | integration | (new file or server.rs::tests) |
| EXEC-01 | ExecutorSession query execution | unit | session.rs::tests, query.rs::tests |
| EXEC-02 | Exchange operator stream | unit | exchange_operator.rs::tests |
| WORKER-01 | Worker service task execution | unit | (worker crate TBD) |
| WORKER-02 | Arrow Flight endpoint tests | integration | (worker crate TBD) |
| COMMON-01 | UDF registry | unit | udf.rs::tests |
| COMMON-02 | Federated connector traits | unit | federated.rs tests with mockall |
| COMMON-03 | OctopusError error paths | unit | error.rs tests |

### Wave 0 Gaps
- [ ] HTTP router handler extraction for COORD-03 (if not already modularized)
- [ ] Worker crate structure discovery (WORKER-01, WORKER-02 may need new test files)

## Security Domain

Not applicable — unit tests do not handle security-sensitive operations. All tests use in-memory data and synthetic fixtures.

## Sources

### Primary (HIGH confidence)
- `/workspace/octopus/octopus-coordinator/src/scheduler.rs` — QueryScheduler implementation
- `/workspace/octopus/octopus-coordinator/src/query_service.rs` — QueryService state machine
- `/workspace/octopus/octopus-coordinator/src/exchange_operator.rs` — Exchange operator placeholder
- `/workspace/octopus/octopus-executor/src/session.rs` — ExecutorSession
- `/workspace/octopus/octopus-executor/src/query.rs` — QueryExecutor
- `/workspace/octopus/octopus-common/src/error.rs` — OctopusError
- `/workspace/octopus/octopus-common/src/udf.rs` — UDF registry
- `/workspace/octopus/octopus-common/src/federated.rs` — Federated connector traits

### Secondary (MEDIUM confidence)
- `/workspace/octopus/octopus-coordinator/Cargo.toml` — dev-dependencies verified (tower-test 0.4)
- `/workspace/octopus/Cargo.lock` — mockall 0.13.1, tokio-test 0.4.5 verified

### Tertiary (LOW confidence)
- tower-test API usage — based on standard Tower ecosystem patterns, not verified against specific version docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies verified in Cargo.lock
- Architecture: HIGH — implementation files read directly
- Pitfalls: MEDIUM — based on general Rust testing experience, not project-specific history

**Research date:** 2026-05-22
**Valid until:** 2026-06-22 (30 days — stable domain)