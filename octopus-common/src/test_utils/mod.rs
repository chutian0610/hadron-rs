// See the License for the specific language governing permissions and
// limitations under the License.

//! Test utilities for Octopus crates.
//!
//! This module provides common test utilities including:
//! - [`federated::InMemoryConnectionPool`] - mock connection pool for testing
//! - [`federated::MockTypeAdapter`] - mock type adapter for testing
//!
//! # Usage
//!
//! Add to your crate's `Cargo.toml`:
//! ```toml
//! [dev-dependencies]
//! octopus-common = { path = "../octopus-common", features = ["test-utils"] }
//! ```
//!
//! Then in your test code:
//! ```
//! use octopus_common::test_utils::federated::{InMemoryConnectionPool, MockTypeAdapter};
//! ```

pub mod federated;
pub use federated::{InMemoryConnectionPool, MockTypeAdapter};