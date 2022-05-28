/*
NeutronFS B (neutron skiplist fs)

Assumes:
- a relatively fast secondary storage drive with sector size == page size (4K)
- pretty fast random reads, no seek time

Design:
Superblock => stores an offset to every other substructure

Free List => fixed LIFO list of free sectors

Root List => stores an index to the main fs list

Kernel Bookkeeping:
/sys/users => stores permissions for each user on the system. And their names and passwords. If enabled. By default, non existent. Can be used with software to determine whether a user can read/write a specific vnode number

/sys/fs/rootfs_meta => TOML that stores extra metadata for the rootfs. A list of pairs of [inode: <times>, <etc>]. Used by `ls`. And usually a memory mapped file. The [free_inode] stores a list of free inodes LIFO. In a serialised state .serde or a readable state yml for quick viewing just in case
*/

// -------------
// USES
// -------------

extern crate alloc;
use alloc::vec::Vec;
use bincode::{Decode, Encode};
use rand_mt::Mt19937GenRand64;

// -------------
// TYPE INTERFACE
// -------------

pub const DEFAULT_FS_NODE_SIZE: u16 = u16::MAX;

pub const SECTOR_SIZE: u64 = 4096;
pub const PAGE_SIZE: u64 = 4096;

pub const MAX_LEVELS: usize = SECTOR_SIZE as usize - 1;

// -------------
// IN MEMORY STRUCTURES
// -------------

pub type FSUUID = [u8; 16];
pub type Checksum32 = u32;

/// Use this to align the data correctly in memory before writing to disk
#[repr(align(4096))]
pub struct Align4096<T>(T);

/// Core metadata of the fs in memory
/// On disk, uses a subset of these (implemented by method to_disk_format())
#[repr(C, packed)]
#[derive(Debug, Encode, Decode)]
pub struct SuperBlock {
    // AUTHENTICITY OF FS
    magic: u64,
    fs_uuid: FSUUID,

    // INTEGRITY
    checksum: Checksum32,

    // LABELS
    label: [u8; 0x100],
    generation: u64,

    // OFFSETS
    physical_addr_of_partition: u64,
    core_fs_skiplist_addr: u64,
    free_cluster_list_addr: u64,

    // TOTAL SIZES
    n_sectors_total: u64,
    n_sectors_used: u64,

    // FEATURE SIZE
    sector_size_bytes: u16,
    fs_node_size_bytes: u16,
}

/*
We want it to be able to be empty. So Option<> maybe
We also dont need a root list per se. We can just cache the n levels per inode so we dont have to call .len() each time
*/

#[repr(C, packed)]
#[derive(Debug, Encode, Decode)]
pub struct RootList {
    n_inode_levels: u64,
    // each level points to the next one with the same level available
    // will have to alloc quite a bit more data for this. All we need is .next() and .down()
    inodes: Vec<INode>,
}

impl RootList {
    pub fn new(n_inode_levels: u64, inodes: Vec<INode>) -> Self {
        Self {
            n_inode_levels,
            inodes,
        }
    }

    // search for a value (inode number). And maybe return a ref to that node
    pub fn search(&mut self, val: u64) -> Option<&INode> {
        let mut curr_node = &self.inodes[self.n_inode_levels as usize - 1];

        // for each level, compare
        for level in 0..self.n_inode_levels as usize {
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

/// Inode = Index Node
#[repr(C, packed)]
#[derive(Debug, Encode, Decode)]
pub struct INode {
    value: u64,
    data: Vec<DataNode>,
    // its actual data (pointers to chunks) is also a skiplist
    next_nodes: Vec<INode>,
}

impl INode {
    pub fn new(value: u64, data: Vec<DataNode>, next_nodes: Vec<INode>) -> Self {
        Self {
            value,
            data,
            next_nodes,
        }
    }

    pub fn val(&self) -> u64 {
        self.value
    }

    // best idea for most small-med range sized files
    // returns it all as a contiguous chunk of bytes
    pub fn get_all_data(&mut self) {}

    // returns one or more blocks depending on the offset and size you want
    // more efficient for bigger files
    pub fn get_data(&mut self) {}

    // methods to point to a new node for a certain level

    // idk if recursive search or bottom up. I think just iterative on the main
}

// it points to an offset
#[repr(C, packed)]
#[derive(Debug, Encode, Decode)]
pub struct DataNode {
    offset: u64,
    next_nodes: Vec<DataNode>,
}

/// Always adds LIFO (inserts at the front)
/// could prob be very fragmented
/// Maybe could also be a skip list
#[repr(C, packed)]
#[derive(Debug, Encode, Decode)]
pub struct FreeClusterNode {
    cluster_number: u64,
    next_nodes: Vec<FreeClusterNode>,
}

impl FreeClusterNode {
    pub fn new(cluster_number: u64, next_nodes: Vec<FreeClusterNode>) -> Self {
        Self {
            cluster_number,
            next_nodes,
        }
    }
}

// -----------------
// INTERNAL API
// -----------------

pub struct ReadQueue {
    // want to read these blocks
    queue: Vec<u64>,
}

pub fn push_read_request(cluster_number: u64) {}

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
