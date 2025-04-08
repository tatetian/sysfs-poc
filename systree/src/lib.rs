mod attr;
mod event;
mod node;
mod utils;
mod tree;

static SYS_TREE: SysTree = SysTree::new();

/// Gets the singleton of the `SysTree`.
pub fn singleton() -> &'static SysTree {
    &SYS_TREE
}

// TODO: initialize the singleton in the component init function

/// An owned string or a static reference to string.
pub type SysStr = Cow<'static, str>;