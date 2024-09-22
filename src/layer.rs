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

    /// Create a new concurrency limit layer.
    pub const fn new(threshold: Threshold, provider: M) -> Self {
        MemoryLimitLayer {
            threshold,
            provider,
        }
    }
}

impl<S, M> Layer<S> for MemoryLimitLayer<M>
where
    M: AvailableMemory,
{
    type Service = MemoryLimit<S, M>;

    fn layer(&self, service: S) -> Self::Service {
        MemoryLimit::new(service, self.threshold.clone(), self.provider.clone())
    }
}
