//! Test utilities for Octopus crates.
//!
//! Enabled by the "test-utils" feature in octopus-common.
//!
//! Modules:
//! - mock: Mock implementations for traits and structs (MockWorkerRegistry)
//! - fixture: TestRecordBatchFactory for Arrow test data
//! - timeout: Timeout helpers for async tests

pub mod mock;
pub mod fixture;
pub mod timeout;