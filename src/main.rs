use bytes::Bytes;
use core::task;
use neutron_fs::driver::block::{Block, BlockDriver, ReadQueue, WriteQueue};
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
        resp: Responder<Bytes>,
    },
    Write {
        block_id: u64,
        block: Bytes,
        resp: Responder<()>,
    },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(64);

    // spawn tokio and move tx and push a read/write request
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

        println!("res = {:?}", res.unwrap());
    });

    let manager = tokio::spawn(async move {
        // Start receiving requests
        while let Some(cmd) = rx.recv().await {
            match cmd {
                DiskRequest::Read { block_id, resp } => {
                    // handle the read request by searching the block and wrapping it in a DiskResponse
                    let res = Bytes::from("RANDOM JUNK");
                    let _ = resp.send(res);
                }
                DiskRequest::Write {
                    block_id,
                    block,
                    resp,
                } => {}
            }
        }
    });

    req_read.await.unwrap();
    manager.await.unwrap();
}

fn simulate() -> ! {
    let mut blocks: Vec<Block> = vec![[0 as u8; 4096]; 100];
    // let mut read_queue: ReadQueue = ReadQueue::new(vec![]);
    // let mut write_queue: WriteQueue = WriteQueue::new(vec![]);
    // let mut vpartition: VPartition = VPartition::new(100, blocks, read_queue, write_queue);
    // let mut partition = Arc::new(vpartition);

    // println!(
    //     "created a virtual partition of 100 blocks. Partition = {:?}",
    //     partition
    // );

    // let mut buf = Vec::with_capacity(4096);
    static mut buf: [u8; 4096] = [0 as u8; 4096];
    let cluster_number = 1;

    loop {}
}

// -------------
// VIRTUAL PARTITION
// -------------

#[derive(Debug)]
pub struct VPartition {
    n_blocks: u64,
    blocks: Vec<Block>,
    read_queue: ReadQueue,
    write_queue: WriteQueue,
}

impl VPartition {
    pub fn new(
        n_blocks: u64,
        blocks: Vec<Block>,
        read_queue: ReadQueue,
        write_queue: WriteQueue,
    ) -> Self {
        Self {
            n_blocks,
            blocks,
            read_queue,
            write_queue,
        }
    }

    pub fn push_read_request(&mut self, cluster_number: u64) {}

    fn push_write_request(&mut self, cluster_number: u64, block_to_write: Block) {}
}

// -------------
// TESTS
// -------------

#[test]
fn test_stuff() {
    assert_eq!(1, 1);
}
