// -------------
// ROOTFS RAM STRUCTURES
// -------------

// initramfs
// syscalls should use RAM based structures

// On NeFS, we dont care about file extensions

use alloc::{borrow::Cow, string::String, vec::Vec};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum FileEncoding {
    /// Includes ASCII and the entire unicode set
    UTF8,
    /// Usually displayed as UTF-8 0-9/A-F
    Hex,
    /// "raw" = bytes of whatever kind of encoding
    Raw,
}

// I dont see why we dont just store the file's data on the heap
// or a pseudo heap on the stack
// in shared memory and CoW

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FileData {
    name: String,
    // high level description of the file type, e.g. PNG
    file_type: String,
    encoding: FileEncoding,
    data: Vec<u8>,
}

/// Always contains . and .. links to dirs
#[repr(C)]
#[derive(Debug, Clone)]
pub struct DirData {
    name: String,
    files: Vec<NeFSFile>,
    parent: *mut DirData,
}

impl DirData {
    pub fn new_dir(name: String, files: Vec<NeFSFile>, parent: *mut DirData) -> Self {
        Self {
            name,
            files,
            parent,
        }
    }

    // add a file

    // remove a file
}

#[repr(C)]
#[derive(Debug, Clone)]
pub enum NeFSFile {
    File(FileData),
    Dir(DirData),
    /// No separation between char/block/socket?
    Device,
    Symlink,
    Socket,
    /// Named pipe. Anonymous pipes are not files
    Pipe,
}

impl NeFSFile {
    // rename
    pub fn rename(&mut self) {}
}

pub struct NeFSFileCoW<'file>(Cow<'file, NeFSFile>);

/*
/
    .
    ..
    sys/
    dev/
*/

pub struct RootFS {}

// -------------
// NEUTRONFS RAM STRUCTURES
// -------------

// skiplists and such?
// actually that would be a pretty bad idea compared to the usual tree
