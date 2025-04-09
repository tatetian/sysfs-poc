//! This crate organizes the kernel information 
//! about the entire system in a tree structure called `SysTree`.
//!
//! This crate provides a singleton of `SysTree`,
//! which is the "model" part of Asterinas's 
//! model-view-controller (MVC) architecture
//! for organizing and managing device and kernel information. 
//! The "view" part is sysfs,
//! a file system that exposes the system information 
//! of the in-kernel `SysTree` to the user space.
//! The "controller" part consists of 
//! various subsystems, buses, drivers, and kernel modules.
//! The "view" part has read-only access to the "model",
//! whereas the "controller" part can make changes to the "model".
//! This MVC architecture achieves separation of concerns,
//! making the code more modular, maintainable, and easier to understand.

mod attr;
mod event;
mod node;
mod tree;

pub mod utils;

pub use self::attr:{SysAttr, SysAttrFlags, SysAttrSet, SysAttrSetBuilder};
pub use self::event::{SysEvent, SysEventKv, SysEventAction};
pub use self::node::{SysNodeType, SysBranchNode, SysNode, SysSymlink, SysObj, SysNodeId};
pub use self::tree::{SysTree};

static SYS_TREE: SysTree = SysTree::new();

/// Gets the singleton of the `SysTree`.
pub fn singleton() -> &'static SysTree {
    &SYS_TREE
}

// TODO: initialize the singleton in the component init function

/// An owned string or a static reference to string.
pub type SysStr = Cow<'static, str>;
