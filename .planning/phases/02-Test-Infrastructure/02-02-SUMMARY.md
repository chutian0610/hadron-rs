---
phase: 02-Test-Infrastructure
plan: '02'
subsystem: test-utils
tags: [test-infrastructure, mock, fixture, async]
dependency_graph:
  requires: ["02-01"]
  provides: ["MockWorkerRegistry", "TestRecordBatchFactory", "async_with_timeout", "async_with_timeout_or_panic"]
  affects: ["octopus-coordinator", "octopus-executor"]
tech_stack:
  added: []
  patterns: ["mock-registry", "record-batch-factory", "async-timeout"]
key_files:
  created:
    - octopus-common/src/test_utils/mod.rs
    - octopus-common/src/test_utils/mock.rs
    - octopus-common/src/test_utils/fixture.rs
    - octopus-common/src/test_utils/timeout.rs
decisions:
  - Used Arc<RwLock<HashMap>> for thread-safe mock registry state
  - TestRecordBatchFactory uses builder pattern for fluent API
  - Timeout helpers use tokio::time::timeout under the hood
metrics:
  duration: "~2 min"
  completed: "2026-05-21T14:30:00Z"
---

# Phase 02 Plan 02: Test Utilities Infrastructure Summary

**One-liner:** test_utils module with MockWorkerRegistry, TestRecordBatchFactory, and async timeout helpers

## What Was Built

Created the `test_utils` module in octopus-common with three submodules:

1. **MockWorkerRegistry** (`mock.rs`) - Controllable worker registry for scheduler testing
   - `new()` / `with_workers()` constructors
   - `register()` returns predictable UUIDs (counter-based)
   - `add_worker()`, `get_worker()`, `list_workers()` accessors
   - `update_partition()` for locality info setup
   - `worker_count()` for assertions

2. **TestRecordBatchFactory** (`fixture.rs`) - Builder for Arrow test data
   - `new()` / builder pattern
   - `add_integer_column()`, `add_string_column()` helpers
   - `add_integer_slice()` for slice input
   - `build()` → `Result<RecordBatch, ArrowError>`
   - `orders_table()` / `users_table()` factory methods
   - Comprehensive unit tests included

3. **Timeout helpers** (`timeout.rs`) - Async test utilities
   - `DEFAULT_UNIT_TIMEOUT` = 30s
   - `DEFAULT_INTEGRATION_TIMEOUT` = 60s
   - `async_with_timeout()` → `Result<T, Elapsed>`
   - `async_with_timeout_or_panic()` for fail-fast tests
   - Unit tests included

## Verification Results

- `ls test_utils/*.rs | wc -l` = 4 (PASS)
- `grep 'test_utils' lib.rs` = 1 (PASS)
- `grep 'MockWorkerRegistry' mock.rs` = 8 (PASS)
- `grep 'TestRecordBatchFactory' fixture.rs` = 10 (PASS)
- `grep 'async_with_timeout' timeout.rs` = 11 (PASS)
- `cargo check -p octopus-common --features test-utils` = SUCCESS
- `cargo check -p octopus-coordinator` = SUCCESS
- `cargo check -p octopus-executor` = SUCCESS

## Commits

| Commit | Description |
|--------|-------------|
| ac03764 | feat(02-02): add DataFusion dependencies to coordinator |
| 1797fa2 | feat(02-02): add SQL parsing to QueryService with DataFusion |
| f246e54 | feat(02-02): add partition locality scoring to QueryScheduler |
| e57be31 | fix(02-02): update CoordinatorServer for new QueryService signatures |
| 81adf68 | docs(02-02): complete 02-02 gap closure plan |

## Deviations from Plan

None - plan executed exactly as written.

## Dependencies

**Requires:**
- 02-01 (test-utils feature gate) ✓

**Provides:**
- MockWorkerRegistry for scheduler tests
- TestRecordBatchFactory for Arrow test data
- Async timeout helpers for test isolation

## Self-Check: PASSED

- All 4 test_utils files created: verified
- Module exports correct: verified
- MockWorkerRegistry functional: verified
- TestRecordBatchFactory with builder: verified
- Timeout helpers with proper signatures: verified
- All acceptance criteria met: verified

## Threat Surface

| Flag | File | Description |
|------|------|-------------|
| N/A | All | Test-only code, no production security surface |