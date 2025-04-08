use systree::SysTree;

use crate::inode::{SysFsInode};

/// A file system for exposing kernel information to the user space.
pub struct SysFs {
    sb: SuperBlock,
    systree: &'static SysTree,
    root: Arc<dyn Inode>,
}

// These parameters are same as those of Linux.
const MAGIC_NUMBER: u64 = 0x62656572;
const BLOCK_SIZE: usize = 1024;
const NAME_MAX: usize = 255;

impl SysFs {
    pub(crate) fn new() -> Self {
        let sb = SuperBlock::new(MAGIC_NUMBER, BLOCK_SIZE, NAME_MAX);
        let systree = systree::singleton();
        let root = SysFsInode::new_root();
        let new_self = Self {
            sb, 
            systree, 
            root,
        };
        Arc::new(new_self)
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
