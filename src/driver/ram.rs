// -------------
// ROOTFS RAM STRUCTURES
// -------------

// On NeFS, we dont care about file extensions
// I also dont see why we dont just store the file's data on the heap / shared memory and CoW

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
    pub fn new_dir(&mut self, name: String, files: Vec<NeFSFile>, parent: *mut DirData) -> Self {
        Self {
            name,
            files,
            parent,
        }
    }

    // add a file
    pub fn add_file(&mut self) {}

    // remove a file
    pub fn remove_file(&mut self) {}
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
    pub fn rename(&mut self) {}
}

pub struct NeFSFileCoW<'file>(Cow<'file, NeFSFile>);

pub struct RootFS {}

// NOTE: VFS = NeFS in memory
// skiplists and such? Maybe that specific skip-b-list
