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

File Handles:
Instead of dealing with pointers and references, which can get messy, we address everything by their 'cluster number' or 'sector number'. Its P(1) to then go to that cluster

The structs themselves dont know how that works, even in memory. In memory you basically map the entire fs tree aligned to page size

Stack Based:
Its hard to do a stack based in memory structure. So instead we dont. We use the heap and grow and shrink as needed, some overhead yes but at least we dont have to write it to the write_queue and jam the bus

We can convert the heap based memory struct to a stack based one somehow. I think its possible since all nodes are the same size so we could pop one of them out and put a certain one in. And just reference the stack addr of it

But the actual memory mapped data will def have to use a 'heap' like structure. Thats prob the bigger thing. Though very low latency file indexing is also good
*/

// -------------
// USES
// -------------

extern crate alloc;
use alloc::{boxed::Box, rc::Rc, vec::Vec};
use bincode::{Decode, Encode};
use core::{
    borrow::BorrowMut,
    cell::{RefCell, RefMut},
};
use rand_mt::Mt19937GenRand64;

// -------------
// TYPE INTERFACE
// -------------

pub const DEFAULT_FS_NODE_SIZE: u16 = u16::MAX;

pub const SECTOR_SIZE: u64 = 4096;
pub const PAGE_SIZE: u64 = 4096;

pub const MAX_LEVELS: usize = SECTOR_SIZE as usize - 1;

pub type ClusterNumber = u64;
pub type InodeNumber = u64;

// node size / entry size, prob like 2 pages
pub const MAX_DATA_NODES: u64 = 8192;

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

#[repr(C)]
#[derive(Debug, Encode, Decode)]
pub struct InternalNodeData {
    // maybe a level?
    level: u64,
    value: InodeNumber,
    cluster_number: ClusterNumber,
    next_node: ClusterNumber,
    lower_node: ClusterNumber,
}

#[repr(C)]
#[derive(Debug, Encode, Decode)]
pub struct LeafNodeData {
    // NOTE: always level 0
    // next in the chain
    value: InodeNumber,
    cluster_number: ClusterNumber,
    next_node: ClusterNumber,
    // actual data
    offset: u64,
    // search the data skiplist for the nodes
    // usually only MAX_DATA_NODES allowed
    // each data skiplistnode
    data_nodes: u64,
}

pub enum Inode {
    // level > 0
    InternalNode(InternalNodeData),
    LeafNode(LeafNodeData),
}

impl Inode {
    /// Given inode number I, find its associated cluster number C
    pub fn find_inode_cluster_number(
        &self,
        inode_number: InodeNumber,
    ) -> Result<ClusterNumber, ClusterNumber> {
        match self {
            Inode::InternalNode(i) => {
                if inode_number == i.value {
                    return Ok(i.cluster_number);
                }
                // <, return return the cluster number of the next one
                else if inode_number < i.value {
                    return Err(i.lower_node);
                }
                // >
                else {
                    return Err(i.next_node);
                }
            }
            // either return or go next
            Inode::LeafNode(l) => {
                if inode_number == l.value {
                    return Ok(l.cluster_number);
                } else {
                    return Err(l.next_node);
                }
            }
        }
    }
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
