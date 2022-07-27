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
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

pub mod block_tokio;
pub mod cli;
pub mod client_server;

#[tokio::main]
async fn main() {}

// -------------
// TESTS
// -------------

#[test]
fn test_stuff() {
    assert_eq!(1, 1);
}
