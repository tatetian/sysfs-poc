use std::sync::Arc;

pub struct SysFsInode {
    // The corresponding node in the SysTree.
    inner_node: InnerNode,

    // The metadata of this inode.
    //
    // Most of the metadata (e.g., file size, timestamps) 
    // can be determined upon the creation of an inode,
    // and are thus kept intact inside the immutable `metadata` field.
    //
    // Currently, the only mutable metadata is `mode`,
    // which allows user space to `chmod` an inode on sysfs.
    metadata: Metadata,
    mode: RwLock<InodeMode>,

    parent: Weak<SysFsInode>,
    this: Weak<SysFsInode>,
}

impl Inode for SysfsInode {
    fn type_(&self) -> InodeType {
        self.metadata.type_
    }

    fn resize(&self, _new_size: usize) -> Result<()> {
        Err(..)
    }

    fn metadata(&self) -> Metadata {
        self.metadata
    }

    fn ino(&self) -> u64 {
        self.metadata.ino
    }

    fn mode(&self) -> Result<InodeMode> {
        Ok(*self.mode.read())
    }

    fn size(&self) -> usize {
        self.metadata.size
    }

    fn atime(&self) -> Duration {
        self.metadata.atime
    }

    fn set_atime(&self, _time: Duration) {
        // do not support modifying  timestamp
    }

    fn mtime(&self) -> Duration {
        self.metadata.mtime
    }

    fn set_mtime(&self, _time: Duration) {
        // do not support modifying  timestamp
    }

    fn ctime(&self) -> Duration {
        self.metadata.ctime
    }

    fn set_ctime(&self, _time: Duration) {
        // do not support modifying  timestamp
    }

    fn fs(&self) -> Arc<dyn FileSystem> {
        crate::singleton().clone()
    }

    fn set_mode(&self, mode: InodeMode) -> Result<()> {
        // TODO: check permissions
        self.mode.write().mode = mode;
        Ok(())
    }

    fn owner(&self) -> Result<Uid> {
        Ok(self.metadata.uid)
    }

    fn set_owner(&self, uid: Uid) -> Result<()> {
        Err(..)
    }

    fn group(&self) -> Result<Gid> {
        Ok(self.metadata.gid)
    }

    fn set_group(&self, gid: Gid) -> Result<()> {
        Err(..)
    }

    fn page_cache(&self) -> Option<crate::vm::vmo::Vmo<aster_rights::Full>> {
        None
    }

    fn read_at(&self, offset: usize, buf: &mut VmWriter) -> Result<usize> {
        self.read_direct_at(offset, buf)
    }

    fn read_direct_at(&self, _offset: usize, buf: &mut VmWriter) -> Result<usize> {
        // TODO: it is unclear whether we should simply igore the offset
        // or report errors if it is non-zero.

        let InnerNode::Attr(attr, leaf) = &self.inner_node else {
            return Err(Error::new(Errno::EINVAL));
        };

        // TODO: check read permission

        leaf.read_attr(attr.name(), buf)
    }

    fn write_at(&self, offset: usize, buf: &mut VmReader) -> Result<usize> {
        self.write_direct_at(offset, buf)
    }

    fn write_direct_at(&self, _offset: usize, buf: &mut VmReader) -> Result<usize> {
        let InnerNode::Attr(attr, leaf) = &self.inner_node else {
            return Err(Error::new(Errno::EINVAL));
        };

        // TODO: check write permission

        leaf.write_attr(attr.name(), buf)
    }

    fn create(&self, name: &str, type_: InodeType, mode: InodeMode) -> Result<Arc<dyn Inode>> {
        // The content of sysfs reflects that of systree,
        // so the user cannot create files.
        return_errno!(Errno::EOPNOTSUPP);
    }

    fn mknod(&self, _name: &str, _mode: InodeMode, _dev: MknodType) -> Result<Arc<dyn Inode>> {
        // The content of sysfs reflects that of systree,
        // so the user cannot create files.
        Err(Error::new(Errno::ENOTDIR))
    }

    fn as_device(&self) -> Option<Arc<dyn Device>> {
        None
    }

    fn readdir_at(&self, mut offset: usize, visitor: &mut dyn DirentVisitor) -> Result<usize> {
        if self.type_() != InodeType::DIR {
            return Err(Error::new(Errno::ENOTDIR));
        }

        // Why interpreting the `offset` argument as an inode number?
        //
        // It may take multiple `getdents` system calls
        // -- and thus multiple calls to this method --
        // to list a large directory when the syscall is provided a small buffer.
        // Between these calls,
        // the directory may have new entries added or existing ones removed
        // by some concurrent users that are working on the directory.
        // In such situations,
        // missing some of the concurrently-added entries is inevitable,
        // but reporting the same entry multiple times would be
        // very confusing to the user. 
        //
        // To address this issue,
        // the `readdir_at` method reports entries starting from a user-given `offset`
        // and returns an increment that the next call should be put on the `offset` argument
        // to avoid getting duplicated entries. 
        // 
        // Different file systems may interpret the meaning of 
        // the `offset` argument differently:
        // one may take it as a _byte_ offset,
        // while the other may treat it as an _index_.
        // This freedom is guaranteed by Linux as documented in
        // [the man page of getdents](https://man7.org/linux/man-pages/man2/getdents.2.html).
        //
        // Our implementation of sysfs interprets the `offset`
        // as an _inode number_.
        // By inode numbers, directory entries will have a _stable_ order
        // across different calls to `readdir_at`.
        let min_ino = offset as Ino;
        let mut dentry_iter = self.new_dentry_iter(min_ino);

        // Dump the dentries returned by the iterator into the output visitor
        let mut last_dentry_ino = min_ino;
        let mut count = 0;
        while let Some(dentry) = dentry_iter.next() {
            let res = visitor.visit(dentry.name(), dentry.ino(), dentry.type_(), dentry.ino());
            if res.is_err() {
                if count == 0 {
                    return Err(Error::new(Errno::EINVAL));
                } else {
                    break;
                }
            }

            count += 1;
            last_dentry_ino = dentry.ino();
        }

        if count == 0 {
            return Ok(0);
        }

        let next_call_min_ino = last_dentry_ino + 1;
        Ok(next_call_min_ino - min_ino)
    }

    fn link(&self, _old: &Arc<dyn Inode>, _name: &str) -> Result<()> {
        // TODO: is the errno correct?
        Err(Error::new(Errno::EPERM))
    }

    fn unlink(&self, _name: &str) -> Result<()> {
        // TODO: is the errno correct?
        Err(Error::new(Errno::EPERM))
    }

    fn lookup(&self, name: &str) -> Result<Arc<dyn Inode>> {
        if self.type_() != InodeType::DIR {
            return Err(Error::new(Errno::ENOTDIR));
        }

        // TODO: check permissions

        if name == "." {
            return self.this();
        } else if name == ".." {
            return self.parent.upgrade().unwrap_or_else(|| self.this());
        };

        match &self.inner_node {
            InnerNode::Branch(branch_sysnode) => {
                self.lookup_node_or_attr(name, branch_sysnode)
            }
            InnerNode::Leaf(leaf_sysnode) => {
                self.lookup_attr(name, leaf_sysnode)
            }
            _ => unreachable!()
        }
    }

    fn rename(&self, _old_name: &str, _target: &Arc<dyn Inode>, _new_name: &str) -> Result<()> {
        Err(Error::new(Errno::EPERM))
    }

    fn read_link(&self) -> Result<String> {
        let InnerNode::Symlink(symlink_node) = &self.inner_node else {
            return Err(Error::new(Errno::EINVAL));
        };

        Ok(symlink_node.target_path().to_string())
    }

    fn write_link(&self, target: &str) -> Result<()> {
        Err(Error::new(Errno::EPERM))
    }

    fn ioctl(&self, _cmd: IoctlCmd, _arg: usize) -> Result<i32> {
        Err(Error::new(Errno::EPERM))
    }

    fn sync_all(&self) -> Result<()> {
        Ok(())
    }

    fn sync_data(&self) -> Result<()> {
        Ok(())
    }

    fn fallocate(&self, _mode: FallocMode, _offset: usize, _len: usize) -> Result<()> {
        Err(Error::new(Errno::EOPNOTSUPP))
    }

    fn poll(&self, mask: IoEvents, _poller: Option<&mut PollHandle>) -> IoEvents {
        let events = IoEvents::IN | IoEvents::OUT;
        events & mask
    }

    fn is_dentry_cacheable(&self) -> bool {
        true
    }
}

impl SysFsInode {
    pub(crate) fn new_root() -> Self {
        let root_inner_node = {
            let sysnode = systree::singleton().root().clone();
            InnerNode::Branch(sysnode)
        };
        let none_parent = Weak::new();
        Self::do_new_dir(root_inner_node, none_parent)
    }

    pub fn this(&self) -> Arc<SysFsInode> {
        self.this.upgrade().unwrap()
    }

    fn lookup_node_or_attr(&self, name: &str, sysnode: &Arc<dyn SysBranchNode>) -> Result<Arc<SysInode>> {
        if let Some(child_sysnode) = sysnode.child(name) {
            let child_type = child_sysnode.type_(); 
            let child_inode = match child_type {
                SysNodeType::Branch => {
                    let child_branch = child_sysnode.dyn_cast();
                    self.new_branch_dir(child_branch)
                }
                SysNodeType::Leaf => {
                    let child_leaf = child_sysnode.dyn_cast();
                    self.new_leaf_dir(child_leaf)
                }
                SysNodeType::Symlink => {
                    let child_symlink = child_sysnode.dyn_cast();
                    self.new_symlink(child_symlink)
                }
            };
            return Ok(child_inode);
        }

        let Some(attr) = sysnode.node_attrs.get(name) else {
            return Err(Error::new(Errno::ENOENT));
        };
        let new_attr_file = self.new_attr_file(attr, sysnode.clone());
        Ok(new_attr_file)
    }

    fn lookup_attr(&self, name: &str, sysnode: &Arc<dyn SysNode>) -> Result<Arc<SysInode>> {
        let Some(attr) = sysnode.node_attrs.get(name) else {
            return Err(Error::new(Errno::ENOENT));
        };
        let new_attr_file = self.new_attr_file(attr, sysnode.clone());
        Ok(new_attr_file)
    }

    fn new_branch_dir(&self, sysnode: Arc<dyn SysBranchNode>) -> Arc<SysFsInode> {
        let inner_node = InnerNode::Branch(sysnode);
        let parent = self.this.clone();
        Self::do_new_dir(inner_node, parent)
    }

    fn new_leaf_dir(&self, sysnode: Arc<dyn SysNode>) -> Arc<SysFsInode> {
        let inner_node = InnerNode::Leaf(sysnode);
        let parent = self.this.clone();
        Self::do_new_dir(inner_node, parent)
    }

    fn do_new_dir(inner_node: InnerNode, parent: Weak<SysFsInode>) -> Arc<SysFsInode> {
        let metadata = {
            let ino = ino::from_inner_node(&inner_node);
            let inode_type = InodeType::Dir;
            Self::new_metadata(ino, inode_type)
        };
        let mode = InodeMode::from_bits_truncate(0o555); // Everyone is allowed to read and list the dir
        Arc::new_cyclic(|this| {
            SysFsInode {
                inner_node,
                metadata,
                mode,
                parent,
                this,
            }
        })
    }

    fn new_attr_file(&self, attr: &SysAttr, sysnode: Arc<dyn SysNode>) -> Arc<SysInode> {
        let inner_node = InnerNode::Attr(attr.clone(), sysnode);
        let metadata = {
            let ino = ino::from_inner_node(&inner_node);
            let inode_type = InodeType::File;
            Self::new_metadata(ino, inode_type)
        };
        let mode = Self::flags_to_inode_mode(attr.flags());
        let parent = self.this.clone();
        Arc::new_cyclic(|this| {
            SysFsInode {
                inner_node,
                metadata,
                mode,
                parent,
                this,
            }
        })
    }

    fn new_symlink(&self, sysnode: Arc<dyn SysSymlink>) -> Arc<SysFsInode> {
        let inner_node = InnerNode::Symlink(sysnode);
        let metadata = {
            let ino = ino::from_inner_node(&inner_node);
            let inode_type = InodeType::Symlink;
            Self::new_metadata(ino, inode_type)
        };
        let mode = InodeMode::from_bits_truncate(0o0444); // Everyone is allowed to read the link 
        let parent = self.this.clone();
        Arc::new_cyclic(|this| {
            SysFsInode {
                inner_node,
                metadata,
                mode,
                parent,
                this,
            }
        })
    }

    /// Creates an iterator for the dentries in this dir.
    fn new_dentry_iter(&self, min_ino: Ino) -> impl Iterator<Item = Dentry> {
        // Creates an iterator that returns dentries in the following order:
        //
        // 1. Dentries of the attributes;
        // 2. Dentries of the child nodes;
        // 3. The dentry of this inode;
        // 4. The dentry of the parent inode.
        //
        // and satisfies the bound that
        // their inode numbers are greater or equal to `min_ino`.
        return match self.inner_node {
            InnerNode::Branch(sysnode) => {
                let attr_dentry_iter = AttrDentryIter::new(
                    sysnode.attr_set(), self.ino(), min_ino);
                let node_dentry_iter = NodeDentryIter::new({
                    let mut children = Vec::new();
                    sysnode.visit_children_with(min_ino, &mut |child| {
                        if child.id() >= min_ino {
                            children.push_back(child.clone());
                        }
                        Some(())
                    });
                    children
                });
                let special_dentry_iter = ThisAndParentDentryIter::new(
                    self, min_ino);
                attr_dentry_iter
                    .chain(node_dentry_iter)
                    .chain(special_dentry_iter) 
            }
            InnerNode::Leaf(sysnode) => {
                let attr_dentry_iter = AttrDentryIter::new(sysnode.attr_set(), min_ino);
                let node_dentry_iter = NodeDentryIter::new(Vec::new());
                let special_dentry_iter = ThisAndParentDentryIter::new(self, min_ino);
                attr_dentry_iter
                    .chain(node_dentry_iter)
                    .chain(special_dentry_iter) 
            }
            _ => {
                unreachable!("the caller should not call this method for non-dir")
            }
        };

        // Helper iterator types

        struct AttrDentryIter<'a> {
            attr_iter: SysAttrIter<'a>,
            dir_ino: Ino,
            min_ino: Ino,
        }
        impl<'a> AttrDentryIter<'a> {
            pub fn new(attr_set: &'a SysAttrSet, dir_ino: Ino, min_ino: Ino) -> Self {
                Self {
                    attr_iter,
                    dir_ino,
                    min_ino,
                }
            }
        }
        impl<'a> Iterator for AttrDentryIter {
            type Item = Dentry;

            fn next(&mut self) -> Option<Dentry> {
                loop {
                    let attr = self.attr_iter().next()?;

                    let ino = ino::from_dir_ino_and_attr_id(self.attr.dir_ino, attr.id());
                    if ino < self.min_ino {
                        continue;
                    }

                    let next = Dentry {
                        ino,
                        name: self.attr.name().to_owned(),
                        type_: InodeType::File,
                    };
                    return Some(next);
                }
            }
        }

        struct NodeDentryIter {
            node_iter: alloc::vec::IntoIter<Arc<dyn SysObj>>,
        }
        impl NodeDentryIter {
            pub fn new(child_nodes: Vec<Arc<dyn SysObj>) -> Self {
                Self {
                    node_iter: child_nodes.into_iter(),
                }
            }
        }
        impl<'a> Iterator for NodeDentryIter {
            type Item = Dentry;

            fn next(&mut self) -> Option<Dentry> {
                let next_node = self.node_iter().next()?;

                let ino = ino::from_sysnode_id(next_node.id());
                let next_dentry= Dentry {
                    ino,
                    name: next_node.name().to_owned(),
                    type_: match next_node.type_() {
                        SysNodeType::Branch | SysNodeType::Leaf => InodeType::DIR,
                        SysNodeType::Symlink => InodeType::SYMLINK,
                    },
                };
                Some(next_dentry)
            }
        }

        struct ThisAndParentDentryIter<'a> {
            this_dir: &'a SysFsInode, 
            min_ino: Ino,
        }
        impl<'a> ThisAndParentDentryIter<'a> {
            pub fn new(this_dir: &'a SysFsInode, min_ino: Ino) -> Self {
                Self {
                    this_dir,
                    min_ino,
                }
            }

            /// This iterator returns the dentry of this dir 
            /// if `min_ino <= THIS_DENTRY_INO`.
            const THIS_DENTRY_INO: Ino = u64::MAX - 2;

            /// This iterator returns the dentry of the parent dir 
            /// if `min_ino <= PARENT_DENTRY_INO`.
            const PARENT_DENTRY_INO: Ino = u64::MAX - 1;
        }
        impl<'a> Iterator for ThisAndParentDentryIter<'a> {
            type Item = Dentry;

            fn next(&mut self) -> Option<Dentry> {
                if self.min_ino <= Self::THIS_DENTRY_INO {
                    let next_dentry = Dentry {
                        ino: self.this_dir.ino(),
                        name: ".".into(),
                        type_: InodeType::DIR,
                    };
                    self.min_ino = Self::PARENT_DENTRY_INO; 
                    Some(next_dentry)
                } else if self.min_ino <= Self::PARENT_DENTRY_INO {
                    let next_dentry = Dentry {
                        ino: {
                            let this_dir_ino = self.this_dir.ino();
                            let parent_dir_ino = self.this_dir
                                .parent
                                .upgrade()
                                .map_or(this_dir_ino, |parent| parent.ino());
                            parent_dir_ino
                        },
                        name: "..".into(),
                        type_: InodeType::DIR,
                    };
                    self.min_ino = u64::MAX;
                    Some(next_dentry)
                } else {
                    None
                }
            }
        }
    }

    fn flags_to_inode_mode(attr_flags: SysAttrFlags) -> InodeMode {
        let mut inode_mode = InodeMode::empty();
        if attr_flags.contains(SysAttrFlags::CAN_READ) {
            inode_mode |= InodeMode::S_IRUSR;
        }
        if attr_flags.contains(SysAttrFlags::CAN_WRITE) {
            inode_mode |= InodeMode::S_IWUSR;
        }
        inode_mode
    }

    fn new_metadata(ino: u64, type_: InodeType) -> Metadata {
        // Experiments on Linux show that the timestamps of inodes 
        // are determined at the time when the inode is first visisted
        // and won't be changed afterwards.
        let now = crate::time::clocks::RealTimeCoarseClock::get().read_time();
        Metadata {
            ino,
            type_,
            atime: now,
            mtime: now,
            ctime: now,
            ..Default::default()
        }
    }

}

impl PartialEq for SysFsInode {
    fn eq(&self, other: &Self) -> bool {
        self.metadata.id == other.metadata.id
    }
}
impl Eq for SysFsInode {}

#[derive(Debug)]
enum InnerNode {
    Branch(Arc<dyn SysBranchNode>),
    Leaf(Arc<dyn SysNode>),
    Attr(SysAttr, Arc<dyn SysNode>),
    Symlink(Arc<dyn SysSymlink>),
}

/// A directory entry of sysfs.
struct Dentry {
    pub ino: Ino,
    pub name: SysStr,
    pub type_: InodeType,
}

mod ino {
    //! Calculating the inode numbers for sysfs inodes _deterministically_.

    // The least significant 8 bits are used to encode the attribute ID.
    const ATTR_INO_SHIFT: u8 = 8;
    const_assert!(SysAttrSet::CAPACITY == (1_usize << ATTR_INO_SHIFT));

    pub fn from_sysnode_id(node_id: &SysNodeId) -> Ino {
        node_id.as_u64() << ATTR_INO_SHIFT
    }

    pub fn from_dir_ino_and_attr_id(dir_ino: Ino, attr_id: u8) -> Ino {
        dir_ino + (attr_id as Ino)
    }

    pub fn from_inner_node(inner_node: &InnerNode) -> Ino {
        match inner_node {
            InnerNode::Branch(sysnode) => from_sysnode_id(sysnode.id()),
            InnerNode::Leaf(sysnode) => from_sysnode_id(sysnode.id()),
            InnerNode::Symlink(sysnode) => from_sysnode_id(sysnode.id()),
            InnerNode::Attr(name, sysnode) => {
                let dir_ino = from_sysnode_id(sysnode.id());
                let attr_id = sysnode
                    .attr_set()
                    .iter()
                    .find(|attr| attr.name() == name)
                    .map(|attr| attr.id())
                    .unwrap();
                from_dir_ino_and_attr_id(dir_ino, attr_id)
            }
        };
    }
}
