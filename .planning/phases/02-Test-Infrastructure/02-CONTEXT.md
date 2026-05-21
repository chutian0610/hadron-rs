# Phase 2: Test Infrastructure - Context

**Gathered:** 2026-05-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Set up test utilities and mock infrastructure needed for all subsequent testing (Phase 3-5). This foundational phase creates reusable test components that all v1.2 tests depend on.

</domain>

<decisions>
## Implementation Decisions

### Mock Infrastructure Approach
- **D-01:** Use `mockall 0.13` with `#[automock]` for trait mocking
- **D-02:** Native async trait support from mockall 0.13, compatible with existing `#[async_trait]`
- **D-03:** Run `cargo expand` to verify mock generated correctly in CI

### Test Data Creation
- **D-04:** Create `TestRecordBatchFactory` with builder pattern in `octopus-common::test_utils`
- **D-05:** Use `RecordBatch::try_from_iter` for flexible test data construction
- **D-06:** Provide schema builders for common cases (orders table, users table, etc.)

### Async Test Utilities
- **D-07:** Always wrap async test code in `tokio::time::timeout` to prevent hanging tests
- **D-08:** Default timeout: 30 seconds for unit tests, 60 seconds for integration tests
- **D-09:** Use `#[track_caller]` on timeout helpers for better error location

### Test Utilities Organization
- **D-10:** Centralize all test utilities in `octopus-common/src/test_utils/`
- **D-11:** Modules: `mock.rs` (mockall helpers), `fixture.rs` (TestRecordBatchFactory), `async.rs` (timeout helpers)
- **D-12:** Each crate adds `octopus-common = { path = "../octopus-common", features = ["test-utils"] }` as dev-dependency
- **D-13:** Test-utils feature gate in Cargo.toml to avoid shipping test code in release builds

### Claude's Discretion
- Cargo workspace dev-dependencies configuration
-具体 timeout 常量值 (30s/60s) 可根据实际调整

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Research
- `.planning/research/STACK.md` — Testing stack recommendations (mockall, tempfile, tower-test)
- `.planning/research/FEATURES.md` — Testing patterns mapped to Octopus crates
- `.planning/research/ARCHITECTURE.md` — Test infrastructure architecture
- `.planning/research/PITFALLS.md` — Common Rust testing pitfalls to avoid

### Requirements
- `.planning/REQUIREMENTS.md` — v1.2 requirements (16 total, 10 in Phase 3)
- `.planning/ROADMAP.md` — Phase 2-5 structure

### Project
- `.planning/PROJECT.md` — Core value and architecture context

</canonical_refs>

<codebase_context>
## Existing Code Insights

### Reusable Assets
- `octopus-common/src/udf.rs` — Existing `#[cfg(test)]` module with UDF tests (only test code in project)
- `octopus-common/src/error.rs` — OctopusError type needing error path tests (COMMON-03)
- Trait signatures in `octopus-common/src/federated.rs` — FederatedConnector, ConnectionPool traits for mockall

### Established Patterns
- Project uses `#[async_trait::async_trait]` throughout — mockall 0.13 handles this natively
- tokio-test 0.4 already in workspace dev-dependencies
- DataFusion SessionContext is the main test harness for query execution

### Integration Points
- Test utilities will be consumed by coordinator, executor, worker crates
- mockall mocks need access to trait definitions (in common or respective crates)

</codebase_context>

<specifics>
## Specific Ideas

- TestRecordBatchFactory should support common test schemas: "id, name, value", "order_id, amount, date"
- Timeout wrapper utility: `async_with_timeout<T>(future, duration) -> Result<T, TimeoutError>`
- MockWorkerRegistry for scheduler tests, MockQueryExecutor for service tests

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

---

*Phase: 2-Test Infrastructure*
*Context gathered: 2026-05-21*