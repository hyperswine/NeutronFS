# Notes

good idea maybe?
prob not

if simpler to use then cool. I want a way to index sectors like FAT. I like the LIFO free sectors idea. Its prob fine to just have 2GB of indexing. I like to just write to a page at a time. If we can just allocate a single page atomically that would be good. Since not a hdd

with ssds. doesnt matter about farer away addresses. No NUMA either so all address accesses should be equally fast
makes good use of the cache. It should cache recently used pages in an NVRAM. Those pages that change may be flushed to disk if its kicked out. If its dirty

if possible, allocate a new level. In a 4K sector we can alloc a whole bunch of entries. So max 512 levels per sector

NOTE: the first part is literally just the in memory-file view
the second part converts that into MMIO requests

put other metadata in a raw file /sys/fs/meta.raw

CoW?
IDK I think we can implement something like that
CoW is great for snapshotting. Just add an extra ref or dont deref it
for data area only. Somehow, where you have sub skip lists as well
I think you can just use ref counts. Both trees point to the same subvolume tree
basically, each data cluster has a ref count. We can dealloc a data cluster when rc = 0. We have to do with in software by keeping track of currently allocated blocks

i feel like its not that big of a deal to "steal" more blocks next to you. Or maybe just store the offset of the next block at the last entry (like a linked list) if you really need more block entries. At that point its prob a fragmentation issue and you should consider defragging

When to flush changes:

1. create a new file
2. delete a file
3. move a file (such that directory links change)
enough time has elapsed
the commit queue is building up (should be at most K commit requests to the MMIO queue)
the above can be decided by the OS or software based on the usage of the user. How many files they have open etc. No need to jam the PCIe bus with requests and always prioritse safety over efficiency in the userspace. But efficiency over safety on the hardware level

how are children stored in a dir
just a list of inode numbers. Technically can be as long as it needs to be
have to search the inode table like k times for a file depth of k. It might be possible cache recently accessed inodes as well with LRU (on another thread)

a node has n levels
a node should be a single page at most with its data section. There can be at most 4096 - 1 levels

in memory structures are converted into in-struct structures
via the MMIO API and special convert() methods that extracts the needed info
and serialises it into bytes to be written to a new disk
On updates, write_update() methods target the specific page on disk and writes the new data to it
that is very different to the in memory API where we're mostly dealing with

each node should 'own' the next
its possible to just use references or ARC
but for now

How to write data to the in memory list:
like git, we have to make a commit that makes an atomic set of changes that can be referenced by a hash

when we save new data to a file, we are basically changing that file's VirtualFile.vec by pushing new data into it. This new data can be represented by a Patch struct, which is basically a diff file meant to be applied to a single file via the userspace API or something

you have to convert that patchfile into a per-block patch file I think. I think the SSD should be smart enough to not "change existing bytes if they are the same". But the smallest thing you can request is an entire page right. So the SSD has to figure out what bytes need to be changed and make the call to the microcontroller to change those specific bytes via byte addressing?

NOTES:

to find a specific block of a specific file: (logn)^2
to find k specific blocks of a specific file: k(logn)^2. Good if not as fragmented so we can allocate large cont sectors from the free area (free list)

assume you can only submit 4K read requests at a time
and receive a 4K block back, sometime later
for now its quite good

before attempting to read that block, check if its already in memory
IDK i think i should have an internal struct and an in memory struct

at most 2^64 clusters?
cluster number is used:
physical addr = physical_offset_of_parition + cluster_area_offset + cluster_number * cluster_size
