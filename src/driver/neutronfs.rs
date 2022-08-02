// -------------
// USES
// -------------

use alloc::{string::String, vec, vec::Vec};
use bincode::{Decode, Encode};
use neutronapi::fs::{Readable, Writable};
use rand_mt::Mt19937GenRand64;

// ----------------
// DISK DEFINITIONS
// ----------------

/// 64K for a single leaf node (at least in the fs)
pub const DEFAULT_LEAF_NODE_SIZE: u16 = 4096;
/// Internal nodes are literally 40 bytes or something
pub const DEFAULT_INTERNAL_NODE_SIZE: u8 = 40;

pub const SECTOR_SIZE: u64 = 4096;
pub const PAGE_SIZE: u64 = 4096;

pub const MAX_LEVELS: usize = SECTOR_SIZE as usize - 1;

/// Actual driver lookup media number
pub type ClusterNumber = u64;
/// Key numbers to sort lists
pub type InodeNumber = u64;
pub type DataNodeNumber = u64;

// node size / entry size, prob like 2 pages
pub const MAX_DATA_NODES: u64 = 8192;

pub const MAX_INTERNAL_ITEMS_PER_NODE: usize = 20;
pub const MAX_LEAF_ITEMS_PER_NODE: usize = 20;

// ---------------
// DISK STRUCTURES
// ---------------

pub type FSUUID = [u8; 16];
pub type Checksum32 = u32;
pub type ChecksumSHA1 = [u8; 20];

/// Use this to align the data correctly in memory before writing to disk
#[repr(align(4096))]
pub struct Align4096<T>(T);

/// Core metadata of the fs in memory. On disk, uses a subset of these (implemented by method to_disk_format())
#[repr(C)]
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

// Each internal node or leaf node should have a header I think. Should they also begin at a start of a cluster?
// Maybe it doesnt matter as much, just read multiple clusters if you have to, and extract the data with offsets and dont overread
#[repr(C)]
#[derive(Debug, Encode, Decode)]
pub struct NodeHeader {
    checksum: ChecksumSHA1,
    size_bytes: u64,
    generation_number: u64,
    n_levels: u64,
}

/// A representation of an internal node that only stores keys. And at most a pointer to a leaf data structure that is formatted in some way
#[repr(C)]
#[derive(Debug, Encode, Decode)]
pub struct InternalNode {
    header: NodeHeader,
    // 0 is always a pointer to the root node
    pointers: [u64; MAX_INTERNAL_ITEMS_PER_NODE],
}

#[repr(C)]
#[derive(Debug, Encode, Decode)]
pub enum ItemType {
    Payload,
}

// For a CoW-able fs, we prob should use extent trees
// otherwise store everything in line, and bloat leaf node really hard?

#[repr(C)]
#[derive(Debug, Encode, Decode)]
pub struct Payload {}

/*
value: InodeNumber,
cluster_number: ClusterNumber,
next_node: ClusterNumber,
offset: u64,
data_nodes: Vec<u64>,
*/

#[repr(C)]
#[derive(Debug, Encode, Decode)]
pub struct LeafNode {
    header: NodeHeader,
    item_type: ItemType,
}

/// Each data node must refer to a cont block of allocated clusters
#[repr(C)]
#[derive(Debug, Encode, Decode)]
pub struct DataNode {
    clusters_used: u64,
    cluster_start_number: ClusterNumber,
}

impl DataNode {
    pub fn new(clusters_used: u64, cluster_start_number: ClusterNumber) -> Self {
        Self {
            clusters_used,
            cluster_start_number,
        }
    }
}

pub type ClusterData = [u8; PAGE_SIZE as usize];

/// Always adds LIFO (inserts at the front). Could prob be very fragmented. Maybe could also be a skip list
#[repr(C)]
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

/// Add an inode or data node to their respective trees. Maybe just have methods for them. Though a lot of the logic is the same
pub fn generate_level() {
    // another way is to slice the 64-bit generated number up into 8 chunks and check each one %2 break if 0 right away or go next if all 8 are 1
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

    // find the key and add it there
}

// -----------------
// USER API
// -----------------

pub struct Inode;

impl Readable for Inode {
    fn read_all(&mut self) -> String {
        let res = String::from("");

        // Read all the data nodes. NOTE: assuming memory is either cached in RAM
        // If you need to, call the block driver to actually read from the SSD

        res
    }

    fn read_at(&mut self, buf: &mut [u8], offset: u64) -> Result<usize, &'static str> {
        // Read into buf of len() bytes
        let bytes_to_read = buf.len();

        // For this inode, find the data nodes that overlap the offset + len
        // If file too small, just read as much as you can. Should return >= 0
        // Usually shouldnt be an error, except if the file is protected or something. Maybe the file on disk is bad

        todo!()
    }

    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> Result<(), &'static str> {
        // basically read_at, but if the file is too small (run into EOF), then it should return an error
        // and not fill the buf

        todo!()
    }
}

impl Writable for Inode {
    fn rewrite(&mut self, buf: &[u8]) {
        // check if there enough data blocks to hold buf
        // total_len >= buf
        // if not, need to allocate one more block (or multiple smaller blocks) >= the size of buf
        // nefs only allows you to allocate 4K blocks at a time

        // if the new data is actually smaller than the total len of blocks
        // check if you can deallocate some. By total_len / len. If > 1, you can deallocate floor(n_extra_blocks)
        // by using the fs drivers' deallocate() function that takes a handle to the block and deallocs it (might be cool if we can just pass the cluster number)

        todo!()
    }

    fn write_at(&mut self, buf: &[u8], offset: u64) -> Result<usize, &'static str> {
        todo!()
    }

    fn write_all_at(&mut self, buf: &[u8], offset: u64) -> Result<(), &'static str> {
        todo!()
    }
}

// ------------
// TESTS
// ------------

#[test]
fn test_basics() {}
