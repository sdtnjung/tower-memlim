use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
    usize,
};
use tokio_test::{assert_pending, assert_ready_err, assert_ready_ok};
use tower::{load_shed::error::Overloaded, service_fn, util::ServiceExt};
use tower::{Service, ServiceBuilder};
use tower_memlim::{
    error::BoxError,
    future::ResponseFuture,
    layer::MemoryLimitLayer,
    memory::{AvailableMemory, Threshold},
};

use tower_test::mock::{self};

async fn service_fn_handle(_request: &str) -> Result<&str, BoxError> {
    Ok("Hello, World!")
}

#[derive(Clone, Debug)]
struct AvailableMemoryStub {
    current: Arc<AtomicUsize>,
}

impl AvailableMemory for AvailableMemoryStub {
    fn available_memory(&self) -> Result<usize, BoxError> {
        Ok((self.current.load(Ordering::SeqCst)).into())
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_waker() {
    let av_memory: Arc<AtomicUsize> = Arc::new(0.into());
    let layer = MemoryLimitLayer::new(
        Threshold::MinAvailableBytes(10),
        AvailableMemoryStub {
            current: av_memory.clone(),
        },
    );

    let mut svc = ServiceBuilder::new()
        .layer(layer)
        .service(service_fn(service_fn_handle));

    let mut mock = tokio_test::task::spawn(svc.ready());

    assert!(mock.poll().is_pending());

    assert!(!mock.is_woken());

    av_memory.store(usize::MAX, Ordering::SeqCst);

    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(mock.is_woken());

    assert!(mock.poll().is_ready());
}

#[tokio::test(flavor = "current_thread")]
async fn test_basis_with_functional_mem_provider_stub() {
    let av_memory: Arc<AtomicUsize> = Arc::new(10.into());
    let layer = MemoryLimitLayer::new(
        Threshold::MinAvailableBytes(10),
        AvailableMemoryStub {
            current: av_memory.clone(),
        },
    );
    let (mut service, _handle) = tower_test::mock::spawn_layer(layer);

    assert_ready_ok!(service.poll_ready());

    let _r1: ResponseFuture<tower_test::mock::future::ResponseFuture<&str>> = service.call("hello");

    av_memory.store(9, Ordering::SeqCst);

    assert_pending!(service.poll_ready());

    av_memory.store(11, Ordering::SeqCst);

    assert_ready_ok!(service.poll_ready());

    av_memory.store(usize::MAX, Ordering::SeqCst);

    assert_ready_ok!(service.poll_ready());

    av_memory.store(usize::MIN, Ordering::SeqCst);

    assert_pending!(service.poll_ready());
}

#[cfg(not(target_os = "linux"))]
#[tokio::test(flavor = "current_thread")]
#[should_panic = "service not ready; poll_ready must be called first"]
async fn test_error_with_discfunctional_mem_provider_on_non_linux() {
    use tokio_test::assert_ready_err;
    use tower_memlim::memory::LinuxCgroupMemory;

    let layer = MemoryLimitLayer::new(Threshold::MinAvailableBytes(10), LinuxCgroupMemory);
    let (mut service, _handle) = mock::spawn_layer(layer);

    assert_ready_err!(service.poll_ready());

    let _r1: ResponseFuture<tower_test::mock::future::ResponseFuture<&str>> =
        service.call("hello 1");
}

#[tokio::test(flavor = "current_thread")]
async fn test_with_load_shed_error() {
    let memory: Arc<AtomicUsize> = Arc::new(10.into());
    let mem_layer = MemoryLimitLayer::new(
        Threshold::MinAvailableBytes(11),
        AvailableMemoryStub {
            current: memory.clone(),
        },
    );

    let mut svc = ServiceBuilder::new()
        .load_shed()
        .layer(mem_layer)
        .service(service_fn(service_fn_handle));

    let err = svc.ready().await.unwrap().call("test").await.err().unwrap();

    err.downcast::<Overloaded>().unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn test_with_load_shed_error_conversion() {
    let memory: Arc<AtomicUsize> = Arc::new(10.into());
    let mem_layer = MemoryLimitLayer::new(
        Threshold::MinAvailableBytes(11),
        AvailableMemoryStub {
            current: memory.clone(),
        },
    );

    let mut svc = ServiceBuilder::new()
        .map_result(|result: Result<_, BoxError>| match result {
            Ok(resp) => Ok(resp),
            Err(err) => {
                if err.is::<tower::load_shed::error::Overloaded>() {
                    Ok("Too many requests")
                } else {
                    Err(err)
                }
            }
        })
        .load_shed()
        .layer(mem_layer)
        .service(service_fn(service_fn_handle));

    let err = svc.ready().await.unwrap().call("test").await.unwrap();

    assert_eq!(err, "Too many requests");
}
