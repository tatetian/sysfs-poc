
pub struct SysFs {
    sb: SuperBlock,
    model: SysTree,
    root: Arc<dyn Inode>,
    this: Weak<Self>,
}

// These parameters are same as those of Linux.
const MAGIC_NUMBER: u64 = 0x62656572;
const BLOCK_SIZE: usize = 1024;

impl SysFs {
    pub(crate) fn new(model: Arc<SysTree>) -> Self {
        let new_self = Arc::new_cyclic(move |weak_fs| {
            let root = SysFsInode::new_root(model.root.clone(), weak_fs.clone());
            Self {
                sb: SuperBlock::new(MAGIC_NUMBER, BLOCK_SIZE, NAME_MAX),
                model,
                root,
                this: weak_fs.clone(),
            }
        });
        new_self
    }
}

impl FileSystem for SysFS {
    fn sync(&self) -> Result<()> {
        Ok(())
    }

    fn root_inode(&self) -> Arc<dyn Inode> {
        self.root.clone()
    }

    fn sb(&self) -> SuperBlock {
        self.sb.clone()
    }

    fn flags(&self) -> FsFlags {
        FsFlags::empty()
    }
}
