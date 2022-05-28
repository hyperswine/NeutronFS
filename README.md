# NeutronFS

NeutronFS driver and app code.

## Executable

NeutronFS userspace apps for reading/writing an NeFS partition.

Uses dioxus-wgpu for web and desktop rendering.

It is possible to use additional backends, like QFS, for viewing QFS virtual files on an NEFS partition.

## Library

A no std library that can be linked as a kernel driver. Though requires alloc. The allocation isnt really that big of a deal, mostly for convenience and some functionalities require it.

If needed, use UEFI-alloc or linked-list-alloc. Generally 2 pages of heap space should be enough. Just keep it away from other things so you dont accidentally overwrite it, at least on a bare metal system.
