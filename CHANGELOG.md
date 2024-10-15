# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# 0.3.0 (October 15, 2024)

### Fixed
- Add missing wake up mechanism via configurable `tokio::time::Sleep`

# 0.2.0 (September 22, 2024)

### Changed

- `MemoryLimit<T, M>` implements `Clone`
- Remove `err` field from `MemoryLimit<T, M>`
- Document `Threshold`
- `Service` impl `fn call` panics if `poll_ready` was not called or did not return `Poll::Ready(Ok(())).`
- Remove `fn failed` from `ResponseFuture`

# 0.1.0 (September 22, 2024)

### Added

- Initial release
