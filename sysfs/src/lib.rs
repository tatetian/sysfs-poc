mod inode;
mod fs;

pub use self::inode::SysFsInode;
pub use self::fs::SysFs;

static SINGLETON: Once<Arc<SysFs>> = Once::new();

pub fn singleton() -> &'static Arc<SysFs> {
    SINGLETON.get()
}

pub fn init() {
    todo!("init the FS singleton")
}