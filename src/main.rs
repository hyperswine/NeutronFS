use bytes::Bytes;
use core::task;
use neutron_fs::driver::block::{make_block, Block, BlockDriver, ReadQueue, WriteQueue};
use std::cell::RefCell;
use std::io::Write;
use std::ops::DerefMut;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::{fs::File, ptr::null};
use std::{thread, time};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};

pub mod cli;

type Responder<T> = oneshot::Sender<T>;

#[derive(Debug)]
enum DiskRequest {
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

#[tokio::main]
async fn main() {
    let mut v_partition = VPartition::new_empty();

    // wait.. so maybe I dont need a read queue/ write queue? like not per se
    // since its already queued

    let (tx, mut rx) = mpsc::channel(64);
    // let (wtx, mut wrx) = mpsc::channel(64);

    let manager = tokio::spawn(async move {
        // Start receiving requests
        while let Some(cmd) = rx.recv().await {
            match cmd {
                DiskRequest::Read { block_id, resp } => {
                    // handle the read request by searching the block and wrapping it in a DiskResponse
                    let block = v_partition.get_block(block_id);
                    let _ = resp.send(block);
                }
                DiskRequest::Write {
                    block_id,
                    block,
                    resp,
                } => {
                    let block = v_partition.write_block(block_id, block);
                    let _ = resp.send(block);
                }
            }
        }
    });

    // spawn tokio and move tx and push a read/write request
    // in practice, would be calling a function that does this, which spawns a tokio thread
    let req_read = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();

        // send a disk request to read
        tx.send(DiskRequest::Read {
            block_id: 0,
            resp: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await;

        println!("read res = {:?}", res.unwrap());

        // WRITE REQ
        let (resp_tx, resp_rx) = oneshot::channel();

        let new_block = [1; 4096];

        // send a write req through tx..? why doesnt it work
        tx.send(DiskRequest::Write {
            block_id: 0,
            block: new_block,
            resp: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await;

        println!("write res = {:?}", res.unwrap());

        let (resp_tx, resp_rx) = oneshot::channel();

        // send a disk request to read
        tx.send(DiskRequest::Read {
            block_id: 0,
            resp: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await;

        println!("read res = {:?}", res.unwrap());
    });
    
    req_read.await.unwrap();

    // read again..
    // Hopefully the write is done before the read? Oh we can just join before hand

    manager.await.unwrap();
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

// -------------
// TESTS
// -------------

#[test]
fn test_stuff() {
    assert_eq!(1, 1);
}
