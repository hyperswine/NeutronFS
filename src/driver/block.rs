// Block driver backend
// a block driver should implement a backend, like an SSD or just a regular file on the host
// the block driver makes read/write requests

use super::{ClusterData, ClusterNumber, PAGE_SIZE};
use alloc::vec::Vec;

// -------------
// MEMORY READER
// -------------

// ? I DONT THINK WE NEED THIS

/// A type that impls Read should be a file like type
/// that starts at its starting offset
pub trait Read {
    // e.g. [u8; 4096]. Should be moved across memory or from disk to buffer
    // to dual copy, use .clone()
    type Res;

    // a source S, which is either a trait or something else
    // defaultly used function
    fn read_from_source(&self, offset: u64, n_bytes: u64) -> Self::Res;
}

pub struct MemReader4096;

impl Read for MemReader4096 {
    type Res = [u8; 4096];

    fn read_from_source(&self, offset: u64, n_bytes: u64) -> Self::Res {
        // do a copy() from the source page

        [0 as u8; 4096]
    }
}

// -------------
// BLOCK DRIVER
// -------------

// or maybe just have a more generic transaction queue for both reading and writing
// the open file descriptors are handled elsewhere and can be pushed here as an arg

pub struct ReadQueue {
    // want to read these blocks
    queue: Vec<ClusterNumber>,
}

impl ReadQueue {
    pub fn new(queue: Vec<ClusterNumber>) -> Self {
        Self { queue }
    }
    pub fn push(&mut self, cluster_number: ClusterNumber) {
        self.queue.push(cluster_number);
    }
}

pub trait BlockDriver {
    fn push_read_request(&mut self, cluster_number: u64);
    /// Access a cluster number to get the data
    fn access_cluster<R: Read<Res = ClusterData>>(
        &mut self,
        cluster_number: ClusterNumber,
        feature_offset: u64,
        reader: R,
    ) -> ClusterData;
}

// sample implementation (neutronapi)

pub struct HostDriver {
    read_queue: ReadQueue,
}

impl BlockDriver for HostDriver {
    fn push_read_request(&mut self, cluster_number: u64) {
        self.read_queue.push(cluster_number);
    }

    fn access_cluster<R: Read<Res = ClusterData>>(
        &mut self,
        cluster_number: ClusterNumber,
        feature_offset: u64,
        reader: R,
    ) -> ClusterData {
        // use the Reader R
        // a starting offset may or may not be given. May be always just pass the partition reader and the feature offset. The cluster number is also an offset
        // if starting at the beginning, feature_offset = 0
        let mut actual_offset = cluster_number * PAGE_SIZE;
        actual_offset += feature_offset;

        // where Res: ClusterData

        let res = reader.read_from_source(actual_offset, 4096);

        // read from R at feature_offset and 4096 Bytes
        // NOTE: in memory, we can just copy() the data For actual disk io, you need the MMIO API which prob involves core::ptr::read/write. Just need to pass the struct that implements Read here

        [0 as u8; 4096]
    }
}
