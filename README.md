# tower-memlim

Enforces a limit on the underlying service when a machine's memory [`Threshold`] is met.

## Load Shedding

By combining [`MemoryLimitLayer`] with tower's [`load_shed`] feature, incoming requests can be rejected once a certain memory [`Threshold`] is met. Ths can help to protect a system from running out of memory.

It also helps to maximize the usage of available resources while maintaining system stability. Compared to setting a limit that does not account for system resource variables, such as requests per second, relative resource bound limits like `MinAvailableBytes` do not require constant adjustment whenever system resources change. Hence memory based load shedding is a perfect match for memory based auto scaling strategies. 

Exemplary scaling pattern:
* Auto-scaler provisions systems based on memory/cpu thresholds
* Load shedder rejects request upon threshold exceedance to prevent out of memory issues and to signal hard resource exhaustion
* Load balancer/upfront webserver detects exhausted system via rejected requests or failing health probes and redirects (or retries) traffic to healthy systems

### Example

```rust
use tower::ServiceBuilder;
use tower_memlim::layer::MemoryLimitLayer;
use tower_memlim::error::BoxError;
use tower_memlim::memory::{Threshold, LinuxCgroupMemory};
use tower::service_fn;

// The friendliest service in town!
// Spreading joy, until the memory limit layer threshold is not exceeded.
async fn svc_handle(_req: &str) -> Result<&str, BoxError> {
    Ok("Nice to see you! (while memory lasts)")
}

let mut svc = ServiceBuilder::new()
    // Map the error to your needs
    .map_result(|result: Result<_, BoxError>| match result {
        Ok(resp) => Ok(resp),
        Err(err) => {
            if err.is::<tower::load_shed::error::Overloaded>() {
                // A web server may want to return a http status code instead
                Ok("Too many requests")
            } else {
                Err(err)
            }
        },
    })
    // Load shed and throw `Overloaded` error
    // when the next layer responds with `Poll::Ready(Err(_))`.
    // Without load shedder requests would be enqueued.
    .load_shed()
    // Upon memory exceeding, this layer responds with `Poll::Ready(Err(_))` 
    // to signal that the service is no longer able to service requests.
    // That allows other layers such as `load_shed` to react on it.
    .layer(MemoryLimitLayer::new(
        Threshold::MinAvailableBytes(11),
        LinuxCgroupMemory
    ))
    .service(service_fn(svc_handle));
```

## Operating Systems

This crate provides support for a Linux memory stats provider, but any other struct implementing [`AvailableMemory`] can be used. When developing on an unsupported platform, consider disabling the layer using [`tower::util::option_layer`].

[`AvailableMemory`] implementors:
* [`LinuxCgroupMemory`]

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in tower-memlim by you, shall be licensed as MIT, without any additional
terms or conditions.

[`LinuxCgroupMemory`]: https://docs.rs/tower/latest/tower_memlim/memory/struct.LinuxCgroupMemory.html
[`MemoryLimitLayer`]: https://docs.rs/tower/latest/tower_memlim/layer/struct.MemoryLimitLayer.html
[`AvailableMemory`]: https://docs.rs/tower/latest/tower_memlim/memory/trait.AvailableMemory.html
[`Threshold`]: https://docs.rs/tower/latest/tower_memlim/memory/enum.Threshold.html
[`tower::util::option_layer`]: https://docs.rs/tower/latest/tower/util/fn.option_layer.html
[`load_shed`]: https://docs.rs/tower/latest/tower/load_shed/index.html