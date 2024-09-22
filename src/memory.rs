//! Memory determination and thresholds

use cgroup_memory::memory_available;

use crate::error::BoxError;

#[derive(Clone, Debug)]
pub enum Threshold {
    MinAvailableBytes(u64),
}

pub trait AvailableMemory
where
    Self: Clone,
{
    fn available_memory(&self) -> Result<usize, BoxError>;
}

/// Implements [AvailableMemory] with help of Linux `/sys/fs/cgroup` files / the `cgroup-memory` crate.
#[derive(Clone)]
pub struct LinuxCgroupMemory;

impl AvailableMemory for LinuxCgroupMemory {
    fn available_memory(&self) -> Result<usize, BoxError> {
        match memory_available() {
            Ok(Some(m)) => Ok(m as usize),
            Ok(None) => Err("Memory cannot be determined".into()),
            Err(e) => Err(e.into()),
        }
    }
}
