//! Future types

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::ready;
use pin_project_lite::pin_project;

use crate::error::BoxError;

pin_project! {
    /// Future for the [`MemoryLimit`] service.
    ///
    /// [`MemoryLimit`]: crate::service::MemoryLimit
    pub struct ResponseFuture<F> {
        #[pin]
        state: ResponseState<F>,
    }
}

pin_project! {
    #[project = ResponseStateProj]
    enum ResponseState<F> {
        Called {
            #[pin]
            fut: F
        },
        MemCheckFailure{
            e: super::error::MemCheckFailure
        },
    }
}

impl<F> ResponseFuture<F> {
    pub(crate) fn called(fut: F) -> Self {
        ResponseFuture {
            state: ResponseState::Called { fut },
        }
    }

    pub(crate) fn failed(e: super::error::MemCheckFailure) -> Self {
        ResponseFuture {
            state: ResponseState::MemCheckFailure { e },
        }
    }
}

impl<F, T, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<T, E>>,
    E: Into<BoxError>,
{
    type Output = Result<T, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().state.project() {
            ResponseStateProj::Called { fut } => {
                Poll::Ready(ready!(fut.poll(cx)).map_err(Into::into))
            }
            ResponseStateProj::MemCheckFailure { e } => {
                let e = std::mem::take(e);
                Poll::Ready(Err(e.into()))
            }
        }
    }
}

impl<F> fmt::Debug for ResponseFuture<F>
where
    // bounds for future-proofing...
    F: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("ResponseFuture")
    }
}
