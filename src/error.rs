//! Error types

use std::fmt;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// An error returned by [`MemoryLimit`] when the service's
/// memory checker was unable to determine the available amount of memory.
///
/// [`MemoryLimit`]: crate::service::MemoryLimit
#[derive(Default)]
pub struct MemCheckFailure {
    inner: Option<BoxError>,
}

impl MemCheckFailure {
    /// Construct a new overloaded error
    pub const fn new(inner: BoxError) -> Self {
        MemCheckFailure { inner: Some(inner)  }
    }
}

impl fmt::Debug for MemCheckFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(format!("Failed to determine available memory: {:?}", self.inner).as_str())
    }
}

impl fmt::Display for MemCheckFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(format!("Failed to determine available memory: {:?}", self.inner).as_str())
    }
}

impl std::error::Error for MemCheckFailure {}
