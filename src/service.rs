use tower_service::Service;

use std::task::{Context, Poll};

use crate::{
    error::{BoxError, MemCheckFailure},
    future::ResponseFuture,
    memory::{AvailableMemory, Threshold},
};

/// Enforces a limit on the underlying service when a memory threshold is met.
#[derive(Debug)]
pub struct MemoryLimit<T, M>
where
    M: AvailableMemory,
{
    inner: T,
    threshold: Threshold,
    mem_checker: M,
    is_ready: bool,
}

impl<T, M> MemoryLimit<T, M>
where
    M: AvailableMemory,
{
    /// Create a new memory limiter.
    pub fn new(inner: T, threshold: Threshold, mem_checker: M) -> Self {
        Self {
            inner,
            threshold,
            mem_checker,
            is_ready: false,
        }
    }

    /// Get a reference to the inner service
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Get a mutable reference to the inner service
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Consume `self`, returning the inner service
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<S, Request, M> Service<Request> for MemoryLimit<S, M>
where
    S: Service<Request>,
    M: AvailableMemory,
    S::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Check current memory usage
        match self.threshold {
            Threshold::MinAvailableBytes(min_m) => match self.mem_checker.available_memory() {
                Ok(v) => {
                    if v < min_m as usize {
                        return Poll::Pending;
                    } else {
                        self.is_ready = true;
                    }
                }
                Err(e) => return Poll::Ready(Err(MemCheckFailure::new(e).into())),
            },
        }

        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        if self.is_ready {
            ResponseFuture::called(self.inner.call(request))
        } else {
            panic!("service not ready; poll_ready must be called first")
        }
    }
}

impl<S: Clone, M> Clone for MemoryLimit<S, M>
where
    M: AvailableMemory,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            threshold: self.threshold.clone(),
            mem_checker: self.mem_checker.clone(),
            is_ready: false,
        }
    }
}
