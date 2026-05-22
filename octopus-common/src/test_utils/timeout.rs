//! Async timeout helpers for preventing hanging tests.
//!
//! Provides [`async_with_timeout`] and [`async_with_timeout_or_panic`] utilities
//! per D-07, D-08, D-09.

use std::time::Duration;
use tokio::time::error::Elapsed;

/// Default timeout for unit tests (30 seconds).
///
/// Per D-08 decision.
pub const DEFAULT_UNIT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default timeout for integration tests (60 seconds).
///
/// Per D-08 decision.
pub const DEFAULT_INTEGRATION_TIMEOUT: Duration = Duration::from_secs(60);

/// Wraps an async future with a timeout, returning Result<T, Elapsed>.
///
/// # Example
/// ```
/// use octopus_common::test_utils::timeout::{async_with_timeout, DEFAULT_UNIT_TIMEOUT};
///
/// # tokio::runtime::Builder::new_current_thread()
/// # .enable_all()
/// # .build()
/// # .unwrap()
/// # .block_on(async {
/// let result = async_with_timeout(
///     async { "completed".to_string() },
///     DEFAULT_UNIT_TIMEOUT,
/// ).await;
/// assert!(result.is_ok());
/// # });
/// ```
pub async fn async_with_timeout<T>(
    future: impl std::future::Future<Output = T>,
    duration: Duration,
) -> Result<T, Elapsed> {
    tokio::time::timeout(duration, future).await
}

/// Wraps an async future with a timeout, panicking on timeout.
///
/// This is useful for tests where hanging indicates a serious bug
/// and we want clear error messages with caller location.
///
/// # Example
/// ```
/// use octopus_common::test_utils::timeout::{async_with_timeout_or_panic, DEFAULT_UNIT_TIMEOUT};
///
/// # tokio::runtime::Builder::new_current_thread()
/// # .enable_all()
/// # .build()
/// # .unwrap()
/// # .block_on(async {
/// async_with_timeout_or_panic(
///     async { "done".to_string() },
///     DEFAULT_UNIT_TIMEOUT,
///     "Operation timed out",
/// ).await;
/// # });
/// ```
pub async fn async_with_timeout_or_panic<T>(
    future: impl std::future::Future<Output = T>,
    duration: Duration,
    msg: &str,
) -> T {
    match tokio::time::timeout(duration, future).await {
        Ok(result) => result,
        Err(_) => panic!("{}: timed out after {:?}", msg, duration),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timeout_success() {
        let result = async_with_timeout(
            async { 42 },
            Duration::from_secs(1),
        ).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_timeout_elapsed() {
        let result = async_with_timeout(
            tokio::time::sleep(Duration::from_millis(50)),
            Duration::from_millis(10),
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_timeout_or_panic_success() {
        let result = async_with_timeout_or_panic(
            async { "ok".to_string() },
            Duration::from_secs(1),
            "should not panic",
        ).await;
        assert_eq!(result, "ok");
    }

    #[tokio::test]
    #[should_panic(expected = "should panic: timed out")]
    async fn test_timeout_or_panic_panics() {
        async_with_timeout_or_panic(
            tokio::time::sleep(Duration::from_secs(10)),
            Duration::from_millis(10),
            "should panic",
        ).await;
    }
}