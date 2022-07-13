# NeutronFS

NeutronFS B (neutron skiplist fs)

Assumes:

- a relatively fast secondary storage drive with sector size == page size (4K)
- pretty fast random reads, no seek time

Design:
Superblock => stores an offset to every other substructure

Free List => fixed LIFO list of free sectors

Root List => stores an index to the main fs list

Kernel Bookkeeping:
/sys/users => stores permissions for each user on the system. And their names and passwords. If enabled. By default, non existent. Can be used with software to determine whether a user can read/write a specific vnode number

/sys/fs/rootfs_meta => TOML that stores extra metadata for the rootfs. A list of pairs of [inode: <times>, <etc>]. Used by `ls`. And usually a memory mapped file. The [free_inode] stores a list of free inodes LIFO. In a serialised state .serde or a readable state yml for quick viewing just in case

File Handles:
Instead of dealing with pointers and references, which can get messy, we address everything by their 'cluster number' or 'sector number'. Its P(1) to then go to that cluster

The structs themselves dont know how that works, even in memory. In memory you basically map the entire fs tree aligned to page size

Stack Based:
Its hard to do a stack based in memory structure. So instead we dont. We use the heap and grow and shrink as needed, some overhead yes but at least we dont have to write it to the write_queue and jam the bus

We can convert the heap based memory struct to a stack based one somehow. I think its possible since all nodes are the same size so we could pop one of them out and put a certain one in. And just reference the stack addr of it

But the actual memory mapped data will def have to use a 'heap' like structure. Thats prob the bigger thing. Though very low latency file indexing is also good

All a data node is is some reference to a contiguous sector of data
It stores the cluster number of that data (in the cluster data area)
And the cont size of that data
Its the lower level driver's job to find that cluster in the physical media and access its data in a safe way

```bash
/
    .
    ..
    sys/
    dev/
```

NOTE: this doesnt handle any other fs. So you have a mounted QFS, that will actually call the QFS driver Readable/Writable trait impls
This uses the RAM module's internal structures
/dev/ should be mounted by udev
