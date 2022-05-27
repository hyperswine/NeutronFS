// good idea maybe?
// prob not

// if simpler to use then cool. I want a way to index sectors like FAT. I like the LIFO free sectors idea. Its prob fine to just have 2GB of indexing. I like to just write to a page at a time. If we can just allocate a single page atomically that would be good. Since not a hdd

// with ssds. doesnt matter about farer away addresses. No NUMA either so all address accesses should be equally fast
// makes good use of the cache. It should cache recently used pages in an NVRAM. Those pages that change may be flushed to disk if its kicked out. If its dirty

// if possible, allocate a new level. In a 4K sector we can alloc a whole bunch of entries. So max 512 levels per sector

// NOTE: the first part is literally just the in memory-file view
// the second part converts that into MMIO requests

// put other metadata in a raw file /sys/fs/meta.raw

// -------------
// USES
// -------------

extern crate alloc;
use alloc::vec::Vec;
use bincode::{Decode, Encode};
use rand_mt::Mt19937GenRand64;

// -------------
// STRUCTURES
// -------------

// each node should 'own' the next
// its possible to just use references or ARC
// but for now

#[repr(C, packed)]
#[derive(Debug, Encode, Decode)]

pub struct HeadNode {
    n_levels: u64,
    // should be n_levels long
    // each level points to the next one with the same level available
    nodes: Vec<Node>,
}

impl HeadNode {
    pub fn new(n_levels: u64, nodes: Vec<Node>) -> Self {
        Self { n_levels, nodes }
    }

    // search for a value (inode number). And maybe return a ref to that node
    pub fn search(&mut self, val: u64) -> Option<&Node> {
        let mut curr_node = &self.nodes[self.n_levels as usize - 1];

        // for each level, compare
        for level in 0..self.n_levels as usize {
            // idk if packed stuff will work properly
            // maybe we implement packed when we write and depack when we go into memory
            let mut next = &curr_node.next_nodes[level];

            // node found
            if next.val() == val {
                return Some(next);
            }
            // node bounded, go down a level
            else if next.val() < val {
                continue;
            }
            // node farer away, go next node
            else {
                curr_node = next;
            }
        }

        None
    }

    /// Assumes all vals should be unique (check inode table)
    /// generate a level via MT or something. On an OS, do it with std
    pub fn add_node(&mut self, val: u64, level: u64) {
        // search for the place to put it in. Like search() but except you have a pointer to the prev node as well
    }

    /// Gets rid of a node
    pub fn remove_node(&mut self, val: u64) {}
}

// a node has n levels
#[repr(C, packed)]
#[derive(Debug, Encode, Decode)]
pub struct Node {
    value: u64,
    next_nodes: Vec<Node>,
}

impl Node {
    pub fn new(value: u64, next_nodes: Vec<Node>) -> Self {
        Self { value, next_nodes }
    }

    pub fn val(&self) -> u64 {
        self.value
    }

    // methods to point to a new node for a certain level

    // idk if recursive search or bottom up. I think just iterative on the main
}

// -----------------
// INTERNAL API
// -----------------

const SECTOR_SIZE: u64 = 4096;
const PAGE_SIZE: u64 = 4096;

// -----------------
// USER API
// -----------------

pub fn add_node() {
    // use mt
    let mut mt = Mt19937GenRand64::new_unseeded();
    let mut level = 0;

    // keep generating a level by % 2
    loop {
        let val = mt.next_u64() % 2;
        // rolled a nothing, break
        if val == 0 {
            break;
        }

        // rolled a 1, increment level
        level += 1;
    }
}
