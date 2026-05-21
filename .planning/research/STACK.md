# Technology Stack: Testing & Documentation

**Project:** Octopus v1.2 - Test Coverage & Code Comments
**Researched:** 2026-05-21
**Confidence:** HIGH

## Executive Summary

For the v1.2 milestone (adding unit tests, integration tests, and code comments), the existing Tokio 1.52 runtime provides the foundation. Key additions needed are mockall for trait mocking, tempfile for file-based tests, and tower-test for service integration testing. No major stack changes required; the focus is on test utilities that integrate cleanly with the existing DataFusion/Tokio/Axum stack.

---

## Recommended Stack

### Core Test Framework
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **tokio** | 1.52 (workspace) | Async runtime for tests | Already in workspace; `#[tokio::test]` macro is the standard for async Rust testing |
| **tokio-test** | 0.4 (workspace) | Async test utilities | Already present in octopus-executor dev-dependencies; provides `block_on`, `task` helpers |

**Confidence: HIGH** — Already in workspace. The `#[tokio::test]` macro handles runtime setup automatically.

### Mocking
| Technology | Version | Purpose | When to Use |
|-----------|---------|---------|-------------|
| **mockall** | 0.13 | Mock generation from traits | Mocking `async_trait` interfaces (UdfRegistry, federated connectors); supports Rust 1.75+ native async traits |
| **mockall_double** | 0.13 | `d` modifier for `&self` vs `&mut self` | When mocking both shared and mutable reference methods |

**Why mockall:** The codebase uses `async_trait` extensively (UdfRegistry in octopus-common, federated connectors). mockall 0.13+ supports native async traits and `async_trait::async_trait`. Generates mocks at compile-time via macros, no runtime overhead.

**Key features:**
- `#[automock]` attribute generates `Mock*` structs from traits
- `#[mockable]` is an alternative for more control
- `expect_*` methods set expectations with `returning()` closures
- `mock!` macro for manual mock definitions

```rust
use mockall::*;
use async_trait::async_trait;

#[automock]
#[async_trait]
trait UdfRegistry: Send + Sync {
    async fn register_scalar(&self, name: &str, func: ScalarUDF) -> UdfResult<()>;
    fn get_scalar(&self, name: &str) -> Option<ScalarUDF>;
}

#[tokio::test]
async fn test_registry() {
    let mut mock = MockUdfRegistry::new();
    mock.expect_register_scalar()
        .returning(|_, _| Ok(()));
    // test using mock...
}
```

**Confidence: HIGH** — mockall is the standard for Rust trait mocking.

### Integration Testing
| Technology | Version | Purpose | When to Use |
|-----------|---------|---------|-------------|
| **tower-test** | 0.4 | Service integration testing | Testing axum HTTP handlers, tower-based gRPC services |
| **hyper-utils** | 0.4 | Mock hyper connections | Low-level HTTP protocol testing |
| **portpicker** | 0.1 | Ephemeral port allocation | Integration tests needing real server instances without port conflicts |
| **testcontainers-rs** | 15 | Docker-based test containers | End-to-end tests for PostgreSQL/MySQL federated connectors |

**Approach for gRPC/HTTP integration tests:** Use real instances with ephemeral ports (portpicker) rather than heavy mocking. This catches more bugs and tests the actual serialization/deserialization path.

```rust
use portpicker::pick_unused_port;
use std::net::SocketAddr;

// In integration test setup:
let port = pick_unused_port().expect("No available ports");
let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
let listener = TcpListener::bind(addr).unwrap();
// Use listener for server...
```

**Confidence: MEDIUM for testcontainers** — testcontainers adds Docker dependency to test environment. Consider if federated connector tests become a bottleneck.

### Test Utilities
| Technology | Version | Purpose | When to Use |
|-----------|---------|---------|-------------|
| **tempfile** | 3 | RAII temporary files/directories | Tests involving Parquet, CSV, JSON file operations; automatic cleanup on drop |
| **assert_cmd** | 2 | CLI binary testing | Testing octopus-cli binary arguments, exit codes, output |
| **predicates** | 3 | Assertion helpers | More expressive assertions (`predicates::str::contains()`, `predicates::path::exists()`) |

```rust
use tempfile::TempDir;

#[tokio::test]
async fn test_parquet_reader() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.parquet");
    write_parquet_file(&file_path).await;

    let reader = ParquetReader::new(&file_path).await.unwrap();
    // temp_dir automatically cleaned up when dropped
}
```

**Confidence: HIGH** — tempfile and assert_cmd are stable, widely-used utilities.

### Code Documentation
| Technology | Purpose | Why |
|------------|---------|-----|
| **rustdoc** | Built into `cargo doc` | Standard Rust documentation tool; use `#[doc = "..."]` for rich comments, `#[example]` for runnable examples |
| **cargo-deadlinks** | Broken doc link checker | Ensures `doc = ...` links don't rot; run in CI |
| **cargo-readme** | README generation from doc comments | Keep README in sync with crate documentation |

**Rustdoc best practices for this codebase:**
- Use `//!` for crate-level docs in lib.rs
- Use `///` for public API docs
- Use `#[example]` code blocks for runnable examples (use `ignore` if example requires complex setup)
- Mark examples that compile but cannot run standalone with `ignore`

```rust
/// UdfRegistry provides UDF (User-Defined Function) registration.
///
/// # Example
///
/// ```ignore
/// let registry = UdfRegistryImpl::new();
/// registry.register_scalar("to_upper", to_upper).await?;
/// ```
pub trait UdfRegistry: Send + Sync {
    // ...
}
```

**Confidence: HIGH** — rustdoc is built into cargo; no additional dependencies needed for basic doc generation.

---

## What NOT to Add

| Library | Why Avoid |
|---------|-----------|
| **rstest** | Parametric testing framework; overkill for this milestone's needs |
| **proptest** | Property-based testing; adds complexity without clear benefit for this codebase |
| **fake** | Fake data generation; can add manually in test modules if needed |
| **criterion** | Benchmarking framework; separate from unit/integration testing |
| **cargo-expand** | Macro expansion for debugging; not a testing tool |
| **trybuild** | Compile-fail test framework; not needed for this milestone |

---

## Crate-Specific Additions

### octopus-common/Cargo.toml
```toml
[dev-dependencies]
mockall = "0.13"
tempfile = "3"
```

### octopus-executor/Cargo.toml
```toml
[dev-dependencies]
mockall = "0.13"
tempfile = "3"
tokio-test = { workspace = true }  # already present
```

### octopus-coordinator/Cargo.toml
```toml
[dev-dependencies]
mockall = "0.13"
tempfile = "3"
portpicker = "0.1"
tower-test = "0.4"
hyper-utils = { version = "0.4", features = ["mock"] }
```

### octopus-worker/Cargo.toml
```toml
[dev-dependencies]
mockall = "0.13"
tempfile = "3"
portpicker = "0.1"
```

### octopus-cli/Cargo.toml
```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

---

## Integration with Existing Stack

### Tokio Runtime Considerations
- `#[tokio::test]` creates a multi-threaded runtime by default
- Use `#[tokio::test(flavor = "single_thread")]` for tests that require single-threaded execution (e.g., some DataFusion tests with specific threading assumptions)
- DataFusion's `SessionContext::sql()` works within `#[tokio::test]` blocks since the execution is async

### gRPC (tonic) Testing
```rust
// Use tower-test for tower-layer-based services
use tower::{Service, ServiceExt};
use tower_test::{assert_response, assert_request};
use http_body::Frame;

// For unit testing a tonic service directly:
let svc = MyGrpcService::new();
let request = my_grpc_request();
let response = svc.call(request).await?;
```

### HTTP (axum) Testing
```rust
// axum provides built-in testing via tower::ServiceExt
use axum::{Router, extract::State};
use axum::http::Request;
use tower::ServiceExt;

let app = Router::new().with_state(());
let response = app
    .oneshot(Request::builder()
        .uri("/query/state/123")
        .body(())?)
    .await?;
assert_eq!(response.status(), StatusCode::OK);
```

### Arrow Flight Testing
- Use `arrow-flight` test utilities for Flight protocol testing
- Use `RecordBatchBuilder` from arrow-array to create test data
- For exchange operator tests, create paired sender/receiver tasks

### DataFusion Testing
- `SessionContext::sql()` returns `DataFrame` which has `.collect()` for collecting results
- Use `assert_df_eq` or manual comparisons on resulting RecordBatches
- Test SQL parsing by checking the logical plan structure

---

## Sources

- [tokio test macro](https://docs.rs/tokio/latest/tokio/attr.test.html) - Async test attribute
- [mockall documentation](https://docs.rs/mockall/latest/mockall/) - Trait mocking framework
- [tower-test](https://docs.rs/tower-test/latest/tower_test/) - Service integration testing
- [tempfile](https://docs.rs/tempfile/latest/tempfile/) - RAII temporary files
- [assert_cmd](https://docs.rs/assert_cmd/latest/assert_cmd/) - CLI testing
- [portpicker](https://docs.rs/portpicker/latest/portpicker/) - Ephemeral port selection