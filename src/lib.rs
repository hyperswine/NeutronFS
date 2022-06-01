#![no_std]

// NEEDS ALLOC FOR SPECIFIC FUNCTIONALITY
extern crate alloc;

// Most of the memory->disk functions prob dont need alloc
// But the memory persistent stuff do. The memory persistent stuff could also use std in some ways
pub mod driver;
