use std::time::Duration;

use crate::{
    error::BoxError,
    memory::{AvailableMemory, Threshold},
    service::MemoryLimit,
};

use tower_layer::Layer;

/// Enforces a limit on the underlying service when a memory [Threshold] is met.
///
/// A common use case is to load shed (reject requests), once the threshold is met. For doing so you must add tower's `load_shed` layer.
/// Otherwise the service enqueues requests until the memory is available again.
#[derive(Debug, Clone)]
pub struct MemoryLimitLayer<M>
where
    M: AvailableMemory,
{
    threshold: Threshold,
    /// Interval in which the next memory check is performed, if the threshold is exceeded.
    ///
    /// The `retryìnterval` has no effect if this layer is wrapped within a load shed layer.
    retry_interval: std::time::Duration,
    /// Memory stats provider
    provider: M,
}

impl<M> MemoryLimitLayer<M>
where
    M: AvailableMemory,
{
    /// Get the available memory. Can be used to validate the
    /// inner memory stat provder before adding the layer.
    pub fn available_memory(&self) -> Result<usize, BoxError> {
        self.provider.available_memory()
    }

    /// Create a new concurrency limit layer with a default [MemoryLimitLayer::retry_interval] of 50ms.
    pub const fn new(threshold: Threshold, provider: M) -> Self {
        MemoryLimitLayer {
            threshold,
            provider,
            retry_interval: std::time::Duration::from_millis(50),
        }
    }

    /// Set a custom [MemoryLimitLayer::retry_interval] which determines when the next memory check is performed, if the threshold is exceeded.
    pub fn with_retry_interval(self, retry_interval: Duration) -> Self {
        MemoryLimitLayer {
            threshold: self.threshold,
            provider: self.provider,
            retry_interval,
        }
    }
}

impl<S, M> Layer<S> for MemoryLimitLayer<M>
where
    M: AvailableMemory,
{
    type Service = MemoryLimit<S, M>;

    fn layer(&self, service: S) -> Self::Service {
        MemoryLimit::new(
            service,
            self.threshold.clone(),
            self.provider.clone(),
            self.retry_interval,
        )
    }
}
