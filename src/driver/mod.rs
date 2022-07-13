// ------------------
// NEUTRON FILESYSTEM
// ------------------

/*
This is the greatest filesystem to ever exist
Example fs in example_fs. If single user, then /home is the only home
If "multiuser", other homes are in /home/guest/<name>
*/

// -------------
// API
// -------------

pub mod block;
pub mod ram;

// -------------
// USES
// -------------

use alloc::{string::String, vec, vec::Vec};
use bincode::{Decode, Encode};
use neutronapi::fs::{Readable, Writable};
use rand_mt::Mt19937GenRand64;

// -------------
// DISK TYPES
// -------------

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

// -------------
// DISK STRUCTURES
// -------------

pub type FSUUID = [u8; 16];
pub type Checksum32 = u32;

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

#[repr(C)]
#[derive(Debug, Encode, Decode)]
pub struct InternalNodeData {
    level: u64,
    value: InodeNumber,
    cluster_number: ClusterNumber,
    next_node: ClusterNumber,
    lower_node: ClusterNumber,
}

#[repr(C)]
#[derive(Debug, Encode, Decode)]
pub struct LeafNodeData {
    // NOTE: always level 0. Next in the chain
    value: InodeNumber,
    cluster_number: ClusterNumber,
    next_node: ClusterNumber,
    // actual data location
    offset: u64,
    // search the data skiplist for the nodes. Usually only MAX_DATA_NODES allowed. Each data skiplistnode
    data_nodes: Vec<u64>,
}

pub enum Inode {
    // level > 0
    InternalNode(InternalNodeData),
    LeafNode(LeafNodeData),
}

impl Inode {
    /// Given inode number I (self), find its associated cluster number C
    pub fn get_cluster_number(&self) -> ClusterNumber {
        match self {
            Inode::InternalNode(i) => i.cluster_number,
            Inode::LeafNode(l) => l.cluster_number,
        }
    }

    /// Get a list of the cluster numbers of all associated data nodes. Uses a backend
    pub fn get_all_data_nodes(&self) -> Vec<ClusterData> {
        let res: Vec<ClusterData> = vec![];

        // TODO: call block driver to find the nodes (or block driver backend) which simulates a file

        res
    }
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

type ClusterData = [u8; PAGE_SIZE as usize];

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
