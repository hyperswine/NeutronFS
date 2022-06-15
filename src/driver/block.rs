// Block driver backend
// a block driver should implement a backend, like an SSD or just a regular file on the host
// the block driver makes read/write requests

use core::sync::atomic::AtomicBool;

use super::{ClusterData, ClusterNumber, PAGE_SIZE};
use alloc::{vec::Vec, sync::Arc};

pub type Block = [u8; 4096];

// -------------
// BLOCK DRIVER
// -------------

// or maybe just have a more generic transaction queue for both reading and writing
// the open file descriptors are handled elsewhere and can be pushed here as an arg
#[derive(Debug)]
pub struct ReadQueue<'a> {
    // want to read these blocks
    queue: Vec<(ClusterNumber, &'a mut [u8], Arc<AtomicBool>)>,
}

#[derive(Debug)]
pub struct WriteQueue {
    // want to read these blocks
    // should just clone() it
    queue: Vec<(ClusterNumber, Block, Arc<AtomicBool>)>,
}

// ? what sync primitives to use?
// read and write queues are different
// multiple userspace processes want to access multiple a single queue at once
// so must acquire a lock
// and push req. Then block and get descheduled until the request is done
// when req is done, the data is in the buffer or write has done and it can continue
// how to signal done? global signals prob expensive and blocks whole system
// signal only that core/thread. And reschedule it

// problem is you may need to return, which is a bit of a problem since async fns return
// and you just have an idle loop
// idk if its good idea to busy wait or sleep the fs thread. maybe that every 1 sec is a good idea
// so what then?
// some rust singalling method
// maybe disk is so fast you can just busy wait for flag = 1

impl WriteQueue {
    pub fn new(queue: Vec<(ClusterNumber, Block, Arc<AtomicBool>)>) -> Self {
        Self { queue }
    }
    pub fn push(&mut self, cluster_number: ClusterNumber, block: Block, arc: Arc<AtomicBool>) {
        self.queue.push((cluster_number, block, arc));
    }
    pub fn pop(&mut self) -> Option<(ClusterNumber, Block, Arc<AtomicBool>)> {
        self.queue.pop()
    }
}

impl<'a> ReadQueue<'a> {
    pub fn new(queue: Vec<(ClusterNumber, &'a mut [u8], Arc<AtomicBool>)>) -> Self {
        Self { queue }
    }
    pub fn push(&mut self, buf: &'a mut [u8], cluster_number: ClusterNumber, arc: Arc<AtomicBool>) {
        self.queue.push((cluster_number, buf, arc));
    }
    pub fn pop(&mut self) -> Option<(ClusterNumber, &'a mut [u8], Arc<AtomicBool>)> {
        self.queue.pop()
    }
}

// *mut is not threadsafe
// &mut is not threadsafe
// must use ARC with mutex

/// Interface for block drivers to implement
pub trait BlockDriver<'a> {
    fn push_read_request(&mut self, buf: &'a mut [u8], cluster_number: u64, complete: Arc<AtomicBool>);
    fn push_write_request(&mut self, cluster_number: u64, block: Block, complete: Arc<AtomicBool>);
}

// -------------
// NEUTRON-LIKE
// -------------

// neutronapi like implementation

// would also have the actual ACPI or MMIO fields to read/write from
// with a core::ptr::read/write volatile
pub struct NeutronDriver<'a> {
    read_queue: ReadQueue<'a>,
    write_queue: WriteQueue,
}

impl<'a> BlockDriver<'a> for NeutronDriver<'a> {
    fn push_read_request(&mut self, buf: &'a mut [u8], cluster_number: u64, complete: Arc<AtomicBool>) {
        self.read_queue.push(buf, cluster_number, complete);
    }

    fn push_write_request(&mut self, cluster_number: u64, block: Block, complete: Arc<AtomicBool>) {
        self.write_queue.push(cluster_number, block, complete);
    }
}
