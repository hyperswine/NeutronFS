// ------------------
// NEUTRON FILESYSTEM
// ------------------

/*
This is the greatest filesystem to ever exist
Example fs in example_fs. If single user, then /home is the only home
If "multiuser", other homes are in /home/guest/<name>
*/

// ------------------
// USE API
// ------------------

use bincode::{config, Encode};
use neutronapi::KTimestamp;

pub mod skiplist;

// ------------------
// CORE STRUCTURES
// ------------------

// PARTITION METADATA

// assumes SSD sector size = page size
// = 4K by default
const DISK_SECTOR_SIZE: usize = 4096;

type FSUUID = [u8; 16];
// CRC32C
type Checksum32 = u32;

const SUPERBLOCK_PRIMARY_PLACEMENT: u64 = 0x10_000;
const SUPERBLOCK_SECONDARY_PLACEMENT: u64 = 0x20_000;
const SUPERBLOCK_TERTIARY_PLACEMENT: u64 = 0x30_000;

// might have to be the same size as btrfs superblock for checksums to work properly. Make sure to place this at 0x10_000
// should be exactly 65536 Bytes. So 256 256-blocks for CRC32C or SHA-2 (slower)
#[repr(C)]
struct Superblock {
    checksum: Checksum32,
    // mostly for EFI support and quick search. Add to GPT entry
    fs_uuid: FSUUID,
    // label for the partition, max 256B name, like "Extras" or "Main". On Neutron, the default label for rootfs is "Main"
    label: [u8; 0x100],
    // on disk LBA of the start of this block
    physical_addr: u64,
    // flags: u64,
    // technically just 8 ASCII bytes, should be "__NeFS__"
    // unless its an extension or modified version of NeFS
    magic: u64,
    // quite important. Basically refers to the transaction number that generated this block (CoW)
    // if things dont match up, there may have been a phantom write so you dont have to find and update the parent node
    generation: u64,

    // ROOT POINTERS
    core_tree_root_logical_addr: u64,
    chunk_tree_root_logical_addr: u64,
    log_tree_root_logical_addr: u64,
    // transaction id for log root
    log_root_transaction_id: u64,

    // SIZE
    // size of partition
    total_bytes: u64,
    // size of used blocks, including superblocks and redundancies
    bytes_used: u64,
    // usually 6 for the root filesystem
    root_dir_object_id: u64,
    // at least one. Could be 2^64 for RAID
    number_of_devices: u64,
    sector_size: u32,
    node_size: u32,
    leaf_size: u32,
    // RAID? Should just be total_bytes or 0/1
    stripe_size: u32,
    // size of a single chunk (array of chunks)
    system_chunk_array_size: u32,

    // NOTE: the generation of each subtree is stored
    chunk_root_generation: u64,
    cache_generation: u64,
    uuid_tree_generation: u64,

    // OTHER
    // compatibility_flags: u64,
    // compatibility_read_only_flags: u64,
    // incompatibility_flags: u64,
    // should be CRC32C always
    // checksum_type: u16,

    // ROOT DATA
    root_level: u8,
    chunk_root_level: u8,
    dev_item: [u8; 0x62],
    sys_chunk_array: [u8; 0x800],
    super_roots: [u8; 0x2A0],
    // reserved: [u8; 0xF0],
    // No need to waste space. Although aligning it to page size isnt that bad of an idea
    // unused: [u8; 0x235],
}

// there can be 1.8 quintillion users
// Its stored as a separate config in sys/users if multi usermode is turned on
// type NeutronUUID = u64;

const MAX_FILE_SIZE_BYTES: u64 = 1024_u64.pow(6);
const BLOCK_SIZE_BYTES: usize = 4096;

type FilePermissions = u16;

// not to be confused with dir item, when you know the node is a dir
// all NeutronItem must have a NeutronItemType
enum NeutronItemType {
    // Vital type
    InodeItem,
    // Vital type
    InodeRef,
    InodeExtraRef,
    XAttrItem,
    OrphanItem,
    DirLogItem,
    DirLogIndex,
    // Vital type
    DirItem,
    DirIndex,
    ExtentData,
    ExtentChecksum,
    RootItem,
    RootBackRef,
    RootRef,
    ExtentItem,
    MetadataItem,
    TreeBlockRef,
    ExtentDataRef,
    ExtentRefV0,
    SharedBlockRef,
    SharedDataRef,
    BlockGroupItem,
    DevExtent,
    DevItem,
    ChunkItem,
    StringItem,
}

struct NeutronFSNodeHeader {
    checksum: Checksum32,
    fs_uuid: FSUUID,
    logical_address: u64,
    // flags: [u8; 7],
    // should be 1 for new filesystems otherwise 0 for an old filesystem
    back_reference: u8,
    chunk_tree_uuid: FSUUID,
    // I think this is the generation of the header. I dont need
    // generation: u64,
    id_of_parent: u64,
    number_of_child_items: u32,
    // 0 = leaf, I think also includes core root nodes
    level: u8,
}

// yyyy-mm--ddThh:mm:ss + nanosecs
// prob not that accurate anyway due to hardware latency
// struct UnixTime {
//     seeconds_since_epoch: u64,
//     nanoseconds: u32,
// }

struct NeutronFSKey {
    object_id: u64,
    item_type: u8,
    // meaning of this actually depends on the type
    offset: u64,
}

struct InternalNode {
    key: NeutronFSKey,
    block_num: u64,
    generation: u64,
}

struct LeafNode {
    key: NeutronFSKey,
    // relative to the end of the header. So where the first payload starts for the data section
    data_offset: u32,
    data_size: u32,
}

enum DirItemType {
    Unknown,
    RegularFile,
    Dir,
    CharDevice,
    BlockDevice,
    Fifo,
    Socket,
    Symlink,
    XAttr,
}

struct NeutronFSItem {
    // the key of the inode_item or root_item associated with this item
    // prob the key of the parent dir
    key: NeutronFSKey,
    offset: u32,
    size: u32,
}

// Actual NeutronFS item
// Can either be another dir, regular file, device file, link
struct NeutronFSDirItem {
    location: NeutronFSKey,
    transaction_id: u64,
    // length of the extended attributes associated with this item. Just 0 for a dir. For a file or something else, might be 0-16k
    data_length: u16,
    // name of the directory entry (not the file)
    name_length: u16,
    d_type: DirItemType,
}

// Supposed to be indexed through (inode_number, inode_item, parent_inode)
// to find the DirItem entires/filename for a given inode
struct NeutronFSInodeRef {
    index: u64,
    name_length: u16,
}

// A type of Node that describes a file
// just needs to contain the Metadata struct for rust/rei. Note, time is in Minima time, "dd-mm-yyyy hh:mm:ss:mm:nn" or amount of nanoseconds since 12000BC or 300K years ago stored as a u128
// https://btrfs.wiki.kernel.org/index.php/Data_Structures#btrfs_inode_item
#[repr(C, packed)]
struct NeutronFSINode {
    // ------USER INFO------
    // THIS DATA IS STORED IN /sys/users if multiuser mode is on. It basically mirrors the RootFS tree and overrides any supported FS mounts permissions unless override_permissions: false
    // creator_id: NeutronUUID,
    // owner_id: NeutronUUID,
    // d-rwx-rwx-rwx (10 bits, 6 bits empty)
    // permissions: FilePermissions,
    // ------FLAGS------
    // I think generation time here
    generation: u64,
    // a user file can be made readonly for safety reasons. system_use may or may not be read_only, but can only be interacted with kernel code or privileged access
    rd_only: bool,
    hidden: bool,
    // A file that is not meant for generic use
    // I guess could also be stored in /sys/fs
    system_use: bool,
    // if 1, then back it up
    needs_to_be_backed_up: bool,
    // ascii or binary (including utf 8)
    is_ascii: bool,
    // 0 if sequential access only, 1 if not
    allow_random_access: bool,
    // for filesystemd to figure out
    locked: bool,
    // ------KEY INFO------
    // number of bytes in a record
    bytes_per_record: u64,
    offset_of_key: u64,
    key_length: u64,
    // ------TIME INFO------
    creation_time: KTimestamp,
    last_accessed: KTimestamp,
    last_changed: KTimestamp,
    // ------BLOCK INFO------
    // in bytes, content only, not inode/blocks
    curr_size: u64,
    // should always be MAX_FILE_SIZE_BYTES (16 EiB)
    max_size: u64,
}

// calculation of Maxima Time:
/*
X nanoseconds since 300K years ago
X/1000 microsec/1000 sec/60 min/24 hr/365 days
(+ extra leap years)

Result = "dd-mm-yyyy hh:mm:ss:mm:nn"
*/

// Logical block / payload of a leaf node
struct Block {}

// -------------------
// INTERNAL API
// -------------------

// for a single partition, containing the structs and refs? You have to build it though. Just dont build the extent tree

// good idea to just map the entire rootfs subvolume into memory

// being used
fn delete_extent_block(block_addr: u64) {}

// add an unused block
// always add in multiples of BLOCK_SIZE. So if a file needs +300 extra bytes
// and BLOCK_SIZE = 512B, then we assign 512B for the write (save)
fn assign_blocks(n: usize) {
    // 1. find free blocks and get their logical addr and ref
    // 2. mark them as being used and delete them from extent tree
    // 3. return the data
}

// -------------
// SUPPORT FUNCTIONS
// -------------

// NOTE: copy on write for all write ops
// when reading, dont do anything to the data, just copy it once to RAM
// and have the kernelmanager/sparx handle concurrency

// on aarch64 and x86, theres a crc32c instruction
// if not available, use the "soft" solution
fn compute_crc32c(input: &[u8]) -> u32 {
    let mut crc32: u32 = 0xFFFF_FFFF;
    let crc_table = [0 as u32; 256];

    for byte in input {
        // I hope the cast is right (00001111 => 00000000.... 00001111)
        let ind: usize = ((crc32 ^ *byte as u32) & 0xFF).try_into().unwrap();
        crc32 = (crc32 >> 8) ^ crc_table[ind];
    }

    crc32 ^ 0xFFFF_FFFF
}

// uses crc32c backend. Note requires alloc prob, unless maybe I dont resize?
/// Just serialises the data into bytes. And computes CRC32C for it
/// in: T: Encode
/// out: Checksum32
fn generate_checksum32<T: Encode>(encodable_struct: T) -> Checksum32 {
    // serialise
    let res = bincode::encode_to_vec(
        encodable_struct,
        config::standard()
            .with_little_endian()
            .write_fixed_array_length()
            .with_variable_int_encoding(),
    );

    let res = match res {
        Ok(r) => r,
        Err(_) => panic!("Something went wrong with serialising to bytes"),
    };

    compute_crc32c(&res)
}

fn verify_checksum32(input: Checksum32) -> bool {
    if input == 0x0 {
        true
    } else {
        false
    }
}

// -------------
// SYSTEM API
// -------------

// NeutronFSDriver should mostly reexport these functions to VFS

// 2 pages
const DEFAULT_FILE_SIZE_ON_DISK: u64 = 8192;

// CREATION / DESTRUCTION Methods

enum NeFSOpcode {
    SUCCESS,
    FAIL,
}

// creation methods can maybe return the logical addr of the newly created file/dir

fn create_file(name: &str, size_bytes: usize) {
    let n_blocks_to_assign = size_bytes / BLOCK_SIZE_BYTES;
    // assign blocks from extent tree
    let blocks = assign_blocks(n_blocks_to_assign);
}

fn delete_file(curr_file_path: &str) {}

fn create_dir(name: &str) {}

// Content Get

// open() should copy the entire file to RAM at offset 0 to n_bytes-1
// read() in the userspace should see that the file is in RAM and not request driver access and instead use VFS/memory mapped file
// no exceptions
fn read_from_file(file_path: &str, offset: u64, n_bytes: u64) {}

// Content Modification

fn write_to_file(file_path: &str, data: &[u8]) {}

// Path Modification

// new file path must be correct
// also works for dirs
fn move_file(curr_file_path: &str, new_file_path: &str) {}

// ---------------
// MMIO API
// ---------------

// on a neutron system, MMIO might be used
// either ACPI or DTB can be used to target the disk and prob partition of interest
// the MMIO controller for the pcie ssd should redir all reads/writes to a specific physical ranges to the PCI BAR
// you need to know how to interact with the PCI BAR. It will tell you the type of device and its functions. And possibly how to use them. Some device memory could prob completely mapped to memory (IOMMU). But it will be virtual

struct MMIO_API {
    write_LBA_addr: u64,
    read_LBA_addr: u64,
}

impl MMIO_API {
    fn new(write_LBA_addr: u64, read_LBA_addr: u64) -> Self {
        Self {
            write_LBA_addr,
            read_LBA_addr,
        }
    }
}

// query kernel device tree (not firmware) for partition number and extra details
fn get_mmio_api_from_partition() -> MMIO_API {
    MMIO_API::new(0, 0)
}
