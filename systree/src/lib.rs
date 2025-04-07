mod attr;
mod event;
mod node;
mod utils;
mod tree;

static SYS_TREE: SysTree = SysTree::new();

pub fn singleton() -> &'static SysTree {
    &SYS_TREE
}

