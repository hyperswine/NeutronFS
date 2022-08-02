use neutron_fs::driver::block::{Block, BlockDriver};
use tokio::io::copy_buf;

/// A simple block driver that blocks on requests

pub struct SimpleBlockDriver {
    clusters: [Block; 1000],
    curr_gpt_entries: usize,
}

/// Note: GPT name is in UTF-16
#[repr(C, packed)]
pub struct GPTEntry {
    partition_type: u128,
    guid: u128,
    first_lba: u64,
    last_lba: u64,
    flags: u64,
    name: [u16; 36],
}

impl GPTEntry {
    pub fn new(
        partition_type: u128,
        guid: u128,
        first_lba: u64,
        last_lba: u64,
        flags: u64,
        name: [u16; 36],
    ) -> Self {
        Self {
            partition_type,
            guid,
            first_lba,
            last_lba,
            flags,
            name,
        }
    }
}

impl SimpleBlockDriver {
    pub fn new(clusters: [Block; 1000], curr_gpt_entries: usize) -> Self {
        Self {
            clusters,
            curr_gpt_entries,
        }
    }

    pub fn create_efi_partition(&mut self) {
        // create a protective MBR on cluster 0

        // create a GPT on cluster 1
    }

    // RN, can only create 1 partition
    pub fn create_nefs_partition(&mut self, size_bytes: usize) {
        // create an entry from cluster 2 + curr_entries offset
        // self.clusters[2] =

        // create an NeFS partition at cluster 34->size
        let start_addr = 34 * 4096;
        let end_cluster =
            SimpleBlockDriver::ceil_addr_to_cluster_number(start_addr + size_bytes as u64);

        // self.clusters[end_cluster] =
    }

    pub fn ceil_addr_to_cluster_number(addr: u64) -> u64 {
        addr / 4096
    }
}

impl BlockDriver for SimpleBlockDriver {
    fn push_read_request(&mut self, buf: &mut [u8], cluster_number: u64) {
        buf.copy_from_slice(&self.clusters[cluster_number as usize])
    }

    fn push_write_request(&mut self, cluster_number: u64, block: Block) {
        self.clusters[cluster_number as usize] = block
    }
}
