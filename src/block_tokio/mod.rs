use neutron_fs::driver::block::{Block, BlockDriver, make_block};
use tokio::{
    sync::{
        mpsc::{self},
        oneshot,
    },
    task::JoinHandle,
};

type Responder<T> = oneshot::Sender<T>;

#[derive(Debug)]
pub enum DiskRequest {
    Read {
        block_id: u64,
        resp: Responder<Block>,
    },
    Write {
        block_id: u64,
        block: Block,
        resp: Responder<()>,
    },
}

/// Expose to the actual higher driver
pub struct BlockDriverTokio {
    vpartition: VPartition,
}

impl BlockDriverTokio {
    pub fn new(vpartition: VPartition) -> Self {
        Self { vpartition }
    }

    pub fn init_manager(
        mut self,
        mut rx: mpsc::Receiver<DiskRequest>,
    ) -> JoinHandle<BlockDriverTokio> {
        let manager = tokio::spawn(async move {
            // Start receiving and handling requests. Maybe give it away after? IDK
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    DiskRequest::Read { block_id, resp } => {
                        // handle the read request by searching the block and wrapping it in a DiskResponse
                        let block = self.vpartition.get_block(block_id);
                        let _ = resp.send(block);
                    }
                    DiskRequest::Write {
                        block_id,
                        block,
                        resp,
                    } => {
                        let block = self.vpartition.write_block(block_id, block);
                        let _ = resp.send(block);
                    }
                }
            }
            self
        });

        manager
    }
}

impl BlockDriver for BlockDriverTokio {
    fn push_read_request(&mut self, buf: &mut [u8], cluster_number: u64) {
        todo!()
    }

    fn push_write_request(&mut self, cluster_number: u64, block: Block) {
        todo!()
    }
}

// -------------
// VIRTUAL PARTITION
// -------------

#[derive(Debug)]
pub struct VPartition {
    n_blocks: u64,
    blocks: Vec<Block>,
}

impl VPartition {
    pub fn new(n_blocks: u64, blocks: Vec<Block>) -> Self {
        Self { n_blocks, blocks }
    }

    pub fn new_empty() -> Self {
        let blocks_zeroed: Vec<Block> = vec![make_block(); 1000];

        Self {
            n_blocks: 1000,
            blocks: blocks_zeroed,
        }
    }

    pub fn get_block(&mut self, block_id: u64) -> Block {
        self.blocks.get(block_id as usize).unwrap().clone()
    }

    pub fn write_block(&mut self, block_id: u64, block: Block) {
        self.blocks[block_id as usize] = block;
    }
}
