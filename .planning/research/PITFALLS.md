# Domain Pitfalls: Testing & Documentation for Rust Projects

**Domain:** Adding comprehensive tests and documentation to existing Rust project
**Context:** Octopus distributed MPP query engine using tokio, DataFusion, Arrow Flight, gRPC
**Researched:** 2026/05/21
**Confidence:** MEDIUM (based on Rust async testing patterns; web search unavailable for verification)

---

## Critical Pitfalls

Mistakes that cause test suite instability, maintenance burden, or false confidence in test coverage.

### Pitfall 1: Blocking the Tokio Runtime

**What goes wrong:** Tests hang indefinitely, panic with stack overflow, or exhibit intermittent failures that appear random.

**Why it happens:** Tokio uses a work-stealing multi-threaded scheduler. Using blocking I/O (`std::thread::sleep`, `std::fs::read`, blocking mutex locks) inside async contexts exhausts the worker thread pool. In tests, this causes deadlocks because the runtime cannot make progress.

**Consequences:**
- Test suite hangs requiring `kill -9`
- Stack overflow in deeply nested async call chains
- Flaky tests that pass/fail unpredictably

**Prevention:**
```rust
// WRONG — blocks the runtime thread
#[tokio::test]
async fn test_bad() {
    std::thread::sleep(std::time::Duration::from_millis(100));
    let data = std::fs::read("file.txt").unwrap();
}

// CORRECT — yields to runtime
#[tokio::test]
async fn test_good() {
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let data = tokio::fs::read("file.txt").await.unwrap();
}
```

**For Octopus specifically:** When testing query execution, ensure any filesystem operations (reading Parquet/CSV test fixtures) use `tokio::fs` not `std::fs`.

**Detection:** Tests hang; `RUST_BACKTRACE=1` shows `tokio::runtime` in stack traces.

---

### Pitfall 2: Shared Mutable State Across Parallel Tests

**What goes wrong:** Tests pass individually but fail together. Test A pollutes Test B's state (flaky tests).

**Why it happens:** Rust's `cargo test` runs tests in parallel by default. Static variables, global state, `lazy_static` singletons, or `Rc<RefCell<>>` shared between tests cause data races and state corruption. With DataFusion's `SessionContext`, reusing a single context across tests creates table registration conflicts.

**Consequences:**
- "poisoned" mutex errors
- Data races (undefined behavior in release mode)
- Tests that fail only on CI (more parallelism) but pass locally

**Prevention:**
```rust
// CORRECT — fresh state per test
#[tokio::test]
async fn test_query_planning() {
    let ctx = SessionContext::new();
    ctx.register_csv("test_data", "test.csv", CsvReadOptions::new()).await.unwrap();
    let df = ctx.sql("SELECT * FROM test_data").await.unwrap();
    // SessionContext dropped at end of test
}
```

- Use `#[serial]` attribute from `serial_test` crate only when serialization is truly required
- Avoid global mutable state in test modules
- Create unique identifiers for test resources (tables, channels, workers)

---

### Pitfall 3: Deadlocks in Multi-Stage Async Test Scenarios

**What goes wrong:** Test hangs forever because it awaits a message that never arrives due to cyclic dependencies or missing task spawning.

**Why it happens:** In async Rust, forgetting to `.await` a `send()` on a channel, or creating a cyclic wait between tasks (Task A waits for Task B, Task B waits for Task A), causes indefinite hangs. The test runtime has no timeout by default.

**Consequences:**
- Test suite hangs, blocking CI
- No error message — test appears to be "running"

**Prevention:**
```rust
#[tokio::test]
async fn test_with_timeout() {
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        async_operation_that_may_deadlock()
    ).await;

    assert!(result.is_ok(), "operation timed out - possible deadlock or hang");
}
```

**For Octopus specifically:** When testing Exchange operator communication between stages, always wrap in timeouts since multi-stage pipelines are prone to deadlock if stage dependencies are misconfigured.

---

### Pitfall 4: Integration Tests Without Proper Resource Cleanup

**What goes wrong:** Tests pass in isolation but fail on CI due to port conflicts, file handle leaks, or memory exhaustion.

**Why it happens:** Integration tests spawn actual servers, bind ports, create temp files. Without explicit cleanup, these accumulate during test runs. CI runs more tests concurrently than local development, amplifying resource exhaustion.

**Prevention:**
```rust
use tempfile::TempDir;

#[tokio::test]
async fn test_with_temp_dir() {
    let temp_dir = TempDir::new().unwrap();
    // temp_dir automatically deleted when dropped, even on panic
    let path = temp_dir.path().to_str().unwrap();
    // use path for test data
}
```

- Use `portpicker` crate for unique port allocation per test
- Use `tempfile` crate for automatic temp file/directory cleanup
- Spawn servers in spawned tasks with proper shutdown signals

---

### Pitfall 5: Over-Mocking Internal Dependencies

**What goes wrong:** Tests pass but code integration fails because mock behavior diverges from real implementation.

**Why it happens:** Excessive mocking of DataFusion internal types (RecordBatch builders, Schema objects, execution plan nodes) creates test doubles that don't reflect real validation logic. Tests become white-box tests of implementation details rather than black-box tests of behavior.

**Consequences:**
- Tests pass but actual query execution fails
- Tests break whenever implementation details change (even when behavior is correct)
- Maintenance burden: every internal refactor requires updating dozens of mocks

**Prevention:**
- Test at the highest level possible: integration tests with real DataFusion execution
- Mock only at system boundaries (network, filesystem, external services)
- For DataFusion specifically: use `RecordBatch::try_from_iter` to create real batches, not hand-crafted mocks

```rust
// PREFER: real RecordBatch
let batch = RecordBatch::try_from_iter(vec![
    ("id", Arc::new(Int64Array::from_vec(vec![1, 2, 3])) as ArrayRef),
    ("value", Arc::new(Float64Array::from_vec(vec![1.0, 2.0, 3.0])) as ArrayRef),
]).unwrap();

// AVOID: mocking internals unless absolutely necessary
```

---

## Moderate Pitfalls

Design issues that degrade test maintainability over time.

### Pitfall 6: Missing Error Path Coverage

**What goes wrong:** Happy path works in production, but error cases cause unexpected panics or incorrect error messages.

**Why it happens:** Writing tests for success is easier; error paths (`Err` variants, `panic!`, `unwrap()`) are often untested. This is especially dangerous in distributed systems where network failures, timeouts, and malformed data are common.

**Prevention:**
```rust
#[tokio::test]
async fn test_error_case() {
    let result = parse_invalid_sql("SELECT * FROM nonexistent").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, QueryError::TableNotFound(_)));
}
```

- Every `?` operator in code should have a corresponding error test
- Use `matches!` macro to verify error variant and message content
- Test both the error type and the error message string

---

### Pitfall 7: Doc Tests That Fail After API Changes

**What goes wrong:** `cargo test --doc` fails because doc examples are outdated after refactoring.

**Why it happens:** Doc tests embedded in `///` comments run as part of the test suite. When APIs change, these examples become stale but are easily forgotten since they're not in the `tests/` directory.

**Prevention:**
```rust
/// Brief description of function.
///
/// # Example
///
/// ```
/// # use octopus_common::query::QueryPlanner;
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// let planner = QueryPlanner::new();
/// // ... complex setup that would make example unreadable ...
/// # Ok(())
/// # }
/// ```
///
/// Note: For complex examples, prefer `ignore` or separate test files.
fn complex_function() {}
```

- Use `ignore` for examples that require complex setup
- Use `compile_fail` for examples showing incorrect usage
- Run `cargo test --doc` in CI to catch doc test failures

---

### Pitfall 8: Slow Test Suite Accumulation

**What goes wrong:** Developers stop running tests locally; CI takes >15 minutes; test suite is ignored.

**Why it happens:** As more tests are added without discipline, integration tests that spawn processes, hit real networks, or process large datasets accumulate. `#[ignore]` attributes pile up for "temporary" slow tests.

**Prevention:**
```rust
// Mark slow tests explicitly
#[tokio::test]
#[ignore = "slow integration test - run with cargo test --ignored"]
async fn test_full_distributed_query_pipeline() {
    // ...
}

// Keep unit tests fast (<100ms each)
// Keep integration tests separate: tests/integration_*.rs
```

- Set CI timeouts and fail builds that exceed them
- Target: unit tests complete in <5 minutes total
- Integration tests run in separate CI job, triggered on PR approval

---

### Pitfall 9: Tests That Require Specific Environment

**What goes wrong:** Tests pass locally but fail on CI because they depend on environment variables, specific ports, or external services not available in CI.

**Why it happens:** Tests written for developer convenience (hardcoded paths, env vars in `.env` file, local service instances) break in CI where these are not configured.

**Prevention:**
- Use `#[cfg(test)]` module attributes to isolate environment-dependent tests
- Provide defaults that work in CI; require explicit env vars for local override
- Document required environment setup in test module documentation
- Use Docker Compose for integration tests that require services (Postgres, etc.)

---

### Pitfall 10: `Send + Sync` Trait Bounds Not Verified

**What goes wrong:** Code compiles, tests pass, but code fails in multi-threaded production context due to missing `Send` or `Sync` trait bounds.

**Why it happens:** Rust auto-derives `Send` and `Sync`. Some types (like `Rc<T>`, `RefCell<T>`) are not `Send`. In single-threaded test context, these compile fine, but fail when used across thread boundaries in production.

**Prevention:**
```rust
#[test]
fn test_send_sync_bounds() {
    fn assert_send_sync<T: Send + Sync>() {}
    // This will fail to compile if MyType is not Send + Sync
    assert_send_sync::<MyType>();
}
```

- Add compile-fail tests for types used across thread boundaries
- Especially important for Exchange operator types, worker message types

---

## Minor Pitfalls

Implementation issues with localized impact.

### Pitfall 11: Inconsistent Test Naming Conventions

**What goes wrong:** Cannot determine what a test covers from its name; hard to find related tests; IDE search is ineffective.

**Prevention:** Use consistent naming:
- Unit tests: `test_unit_<module>_<behavior>`
- Integration tests: `test_integration_<system>_<scenario>`
- Doc tests: `doctest_<module>_<function>_example`

---

### Pitfall 12: Asserting Floating Point Values with `==`

**What goes wrong:** Floating point comparison tests fail intermittently due to precision differences across platforms.

**Prevention:**
```rust
// WRONG
assert_eq!(result, 0.1 + 0.2);

// CORRECT
assert!((result - 0.3).abs() < f64::EPSILON);
// or use approx crate
assert!(approx::assert_approx_eq!(result, 0.3));
```

---

### Pitfall 13: Not Testing `Clone` / `Debug` / `Default` Derives

**What goes wrong:** Derives work at compile time but produce unexpected results at runtime.

**Prevention:** Add basic derives tests for data structures:
```rust
#[test]
fn test_query_result_clone() {
    let result1 = QueryResult::new();
    let result2 = result1.clone();
    assert_eq!(result1.id, result2.id);
}
```

---

## Phase-Specific Warnings for v1.2

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Unit tests for coordinator scheduling | Blocking in async context | Use `#[tokio::test]`, `tokio::time::sleep` |
| Integration tests for distributed query | Shared SessionContext state | Fresh context per test |
| Testing Exchange operators | Deadlock in multi-stage pipeline | Wrap all `.await` in timeouts |
| Mocking DataFusion RecordBatch | Mock diverges from real behavior | Use `RecordBatch::try_from_iter` |
| Testing error paths | Errors untested | Every `?` needs corresponding error test |
| Doc comments | Examples stale after refactoring | Run `cargo test --doc` in CI |

---

## Warning Signs for Test Maintainability

**Red flags (address immediately):**
- Test suite takes >10 minutes to run locally
- Tests pass locally but fail on CI (environment differences)
- `#[ignore]` attributes accumulating without resolution
- Flaky tests that pass/fail without code changes
- Code coverage <70% (significant untested branches)

**Yellow flags (address soon):**
- Test names don't describe what they verify
- No error path tests (only happy path)
- Integration tests mix with unit tests (should be separate files)
- Mock objects >100 lines (likely testing the mock, not the code)

---

## Prevention Strategy for v1.2

1. **Async test discipline**: Use `#[tokio::test]`, never `#[test]` for async
2. **Fresh state per test**: No shared `SessionContext`, no globals
3. **Bound all waits**: Every `.await` should have a timeout
4. **Test error paths**: Every `?` needs an error variant test
5. **Integration tests separate**: `tests/` directory for integration, `#[cfg(test)]` in `src/` for unit
6. **CI gate**: Block merges on test failures; run `cargo test --doc` in CI
7. **Documentation**: Comment complex async control flow, not just public APIs

---

## Sources

- [Tokio: Testing async code](https://tokio.rs/tokio/testing) — Official docs, MEDIUM confidence
- [The Rust Programming Language: Testing](https://doc.rust-lang.org/book/ch11-00-testing.html) — Official book, HIGH confidence
- [DataFusion testing utilities](https://github.com/apache/arrow-datafusion) — Repository, MEDIUM confidence
- [tokio::test documentation](https://docs.rs/tokio/latest/tokio/attr.test.html) — Official docs, HIGH confidence

---

## Verification Checklist

- [ ] All async test functions use `#[tokio::test]`
- [ ] No blocking calls (`std::thread`, `std::fs`) in async test context
- [ ] Each test creates fresh state (no shared SessionContext)
- [ ] All `.await` points have timeout protection
- [ ] Error paths are tested for every `?` operator
- [ ] `cargo test --doc` passes
- [ ] `cargo clippy` passes with no warnings in test code
- [ ] Integration tests are in `tests/` directory, not `src/`
- [ ] Slow tests marked with `#[ignore]` and documented