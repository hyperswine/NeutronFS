// Block driver backend

use core::sync::atomic::AtomicBool;

use super::neutronfs::{ClusterData, ClusterNumber, PAGE_SIZE};
use alloc::{borrow::ToOwned, sync::Arc, vec::Vec};
use bytes::Bytes;

pub type Block = [u8; 4096];

pub fn make_block() -> Block {
    let res: [u8; 4096] = [0; 4096];
    res
}

// I DUNNO IF WE NEED THESE STRUCTS
// maybe for an actual driver. Keep for now as an interface
// For the actual block driver in kernelspace
// It has to DMA the req to the SSD. Then the SSD DMA's the block back to the requested memory
// And sends an interrupt to the cpu global line. Then one of the cores handles the specific interrupt number
// (IO_fin, io_id) for io completed. Which searches the pending requests (after popped from queue), and finds the req with that io_id, and the kthread associated with that (?) or maybe the uthread with that. But the kthread resumes and returns to that uthread. Given the scheduler or another interrupt doesnt interrupt it!!

// -------------
// BLOCK DRIVER
// -------------

/// Do allocate this somewhere handy, like the stack. Cache if possible!
#[derive(Debug)]
pub struct RingBuffer<T, const SIZE: usize> {
    curr_head: usize,
    n_elements: usize,
    max_size: usize,
    buffer: [T; SIZE],
}

impl<T: Clone, const SIZE: usize> RingBuffer<T, SIZE> {
    pub fn new(curr_head: usize, n_elements: usize, max_size: usize, buffer: [T; SIZE]) -> Self {
        Self {
            curr_head,
            n_elements,
            max_size,
            buffer,
        }
    }

    pub fn new_cluster_queue(buf: [T; SIZE]) -> Self {
        // let  = [(0, [0 as u8; 4096]); SIZE];
        // for each elem, increment the cluster number? Nah its a request queue

        Self {
            curr_head: 0,
            n_elements: 0,
            max_size: SIZE,
            buffer: buf,
        }
    }

    /// Push a new element to the back of the queue. Or if full, to the head, and increment the curr_head
    pub fn push(&mut self, t: T) {
        // if full (n_elements = max_size), replace the head and increment (% size if needed)
        let ind = (self.curr_head + self.n_elements) % self.max_size;
        self.buffer[ind] = t;
        // move head to next
        let next = (ind + 1) % self.max_size;
        self.curr_head = next;
    }

    /// Get the head
    pub fn pop(&mut self) -> T {
        let ind = self.curr_head;
        self.curr_head = (self.curr_head + 1) % self.max_size;
        self.buffer[ind].clone()
    }
}

// USE A RING BUFFER, basically a pointer to the current head and a max size
// Then "push" or append to the buffer at head - 1/curr_size. If would overflow, just replace the head

pub const MAX_QUEUE_SIZE: usize = 64;

#[derive(Debug)]
pub struct ReadQueue {
    queue: RingBuffer<(ClusterNumber, Block), MAX_QUEUE_SIZE>,
}

#[derive(Debug)]
pub struct WriteQueue {
    queue: RingBuffer<(ClusterNumber, Block), MAX_QUEUE_SIZE>,
}

impl WriteQueue {
    pub fn new(queue: RingBuffer<(ClusterNumber, Block), MAX_QUEUE_SIZE>) -> Self {
        Self { queue }
    }

    pub fn new_empty() -> Self {
        // uhh ok
        let ringbuffer = RingBuffer::new_cluster_queue([(0, [0 as u8; 4096]); MAX_QUEUE_SIZE]);
        Self { queue: ringbuffer }
    }

    pub fn push(&mut self, cluster_number: ClusterNumber, block: Block) {
        self.queue.push((cluster_number, block));
    }

    pub fn pop(&mut self) -> Option<(ClusterNumber, Block)> {
        Some(self.queue.pop())
    }
}

impl ReadQueue {
    pub fn new(queue: RingBuffer<(ClusterNumber, Block), MAX_QUEUE_SIZE>) -> Self {
        Self { queue }
    }

    pub fn new_empty() -> Self {
        // uhh ok
        let ringbuffer = RingBuffer::new_cluster_queue([(0, [0 as u8; 4096]); MAX_QUEUE_SIZE]);
        Self { queue: ringbuffer }
    }

    pub fn push(&mut self, buf: &mut [u8], cluster_number: ClusterNumber) {
        let mut new_buf: Block = make_block();
        new_buf.copy_from_slice(&buf[..4095]);
        self.queue.push((cluster_number, new_buf));
    }

    pub fn pop(&mut self) -> Option<(ClusterNumber, Block)> {
        Some(self.queue.pop())
    }
}

// *mut is not threadsafe
// &mut is not threadsafe
// must use ARC with mutex

/// Interface for block drivers to implement
pub trait BlockDriver {
    fn push_read_request(&mut self, buf: &mut [u8], cluster_number: u64);
    fn push_write_request(&mut self, cluster_number: u64, block: Block);
}

// -------------
// NEUTRON-LIKE
// -------------

pub struct NeutronDriver {
    read_queue: ReadQueue,
    write_queue: WriteQueue,
}

impl BlockDriver for NeutronDriver {
    fn push_read_request(&mut self, buf: &mut [u8], cluster_number: u64) {
        self.read_queue.push(buf, cluster_number);
    }

    fn push_write_request(&mut self, cluster_number: u64, block: Block) {
        self.write_queue.push(cluster_number, block);
    }
}
