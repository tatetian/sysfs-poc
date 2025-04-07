use core::any::Any;
use core::sync::{Arc, Weak};

/// The three types of nodes in a `SysTree`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SysNodeType {
    /// A branching node is one that may contain child nodes.
    Branch,
    /// A leaf node is one that may not contain child nodes.
    Leaf,
    /// A symlink node,
    /// which ia a special kind of leaf node that points to another node,
    /// similar to a symbolic link in file systems.
    Symlink,
}

/// A trait that represents a branching node in a `SysTree`.
pub trait SysBranchNode: SysNode {
    /// Visits a child node with the given name using a closure.
    /// 
    /// If the child with the given name exists, 
    /// a reference to the child will be provided to the closure.
    /// Otherwise, the closure will be given a `None`.
    /// 
    /// # Efficiency
    /// 
    /// This method is a more efficient, but less convenient version 
    /// of the `child` method.
    /// The method does not require taking the ownership of the child node.
    /// So use this method when efficiency is a primary concern,
    /// while using the `child` method for the sake of convenience. 
    /// 
    /// # Deadlock
    /// 
    /// The implementation of this method depends on the concrete type
    /// and probably will hold an internal lock.
    /// So the caller should do as little as possible inside the closure.
    /// In particular, the caller should _not_ invoke other methods
    /// on this object as this might cause deadlock.
    fn visit_child_with(&self,
        name: &str, 
        f: &mut dyn FnMut(Option<&dyn SysNode>)
    );

    /// Visits child nodes with a minimum ID using a closure.
    /// 
    /// This method iterates over the child nodes
    /// whose IDs are no less than a specified minimum value.
    /// and provide them to the given closure one at a time.
    /// 
    /// The iteration terminates until there are no unvisisted children
    /// or the closure returns a `None`.
    /// 
    /// # Efficiency
    /// 
    /// This method is a more efficient, but less convenient version 
    /// of the `children` method.
    /// The method require neither taking the ownership of the child nodes
    /// nor doing heap allocations.
    /// So use this method when efficiency is a primary concern,
    /// while using the `children` method for the sake of convenience. 
    /// 
    /// # Deadlock
    /// 
    /// Same as the `visit_child_with` method. 
    fn visit_children_with(&self, 
        min_id: u64,
        f: &mut dyn FnMut(&dyn SysObj) -> Option<()>,
    );

    /// Returns a child with a specified name.
    fn child(&self, name: &str) -> Option<Arc<dyn SysObj>> {
        let mut res;
        self.visit_child_with(name, &mut |child_opt| {
            res = child_opt.to_owned()
        });
        res
    }

    /// Collects all children into a `Vec`.
    fn children(&self) -> Vec<Arc<dyn SysObj>> {
        let mut children = Vec::new();
        self.visit_children_with(0, &mut |child| {
            children.push_back(child.clone());
            Some(())
        });
        children
    }

    /// Counts the number of children.
    fn count_children(&self) -> usize {
        let mut count = 0;
        self.visit_children_with(.., &mut |_| {
            *count += 1;
            Some(())
        });
        count
    }
}

/// The trait that abstracts a "normal" node in a `SysTree`.
/// 
/// The branching and leaf nodes are considered "normal",
/// whereas the symlink nodes are considered "special".
/// This trait abstracts the common interface of "normal" nodes.
/// In particular, every "normal" node may have associated attributes.
pub trait SysNode: SysObj {
    fn node_attrs(&self) -> &SysAttrSet;

    fn read_attr(&self, name: &str, writer: &mut VmWriter) -> Result<usize>;

    fn write_attr(&self, name: &str, reader: &mut VmReader) -> Result<()>;

    fn show_attr(&self, name: &str) -> Result<String> {
        let mut buf: Vec<u8> = vec![0; PAGE_SIZE];
        let mut writer = VmWriter::from(buf.as_mut_slice());
        let read_len = self.read_attr(name, &mut writer)?;
        let attr_val = String::from_utf(buf)?;
        Ok(attr_val)
    }

    fn store_attr(&self, name: &str, new_val: &str) -> Result<()> {
        let mut reader = VmReader::from(new_val.as_slice());
        self.write_attr(name, &mut reader)
    }
}

/// A trait that abstracts any symlink node in a `SysTree`.
pub trait SysSymlink: SysObj {
    /// A path that represents the target node of this symlink node.
    fn target_path(&self) -> &str;
}

/// The base trait for any node in a `SysTree`.
pub trait SysObj: Any + Send + Sync + Debug + 'static {
    /// Returns the unique and immutable ID of a node.
    fn id(&self) -> &SysNodeId;

    /// Returns the type of a node.
    fn type_(&self) -> SysNodeType;

    /// Returns the name of a node.
    /// 
    /// The name is guaranteed _not_ to contain two special characters:
    /// `'/'` and `'\0'`.
    /// 
    /// The root node of a `SysTree` has an empty name.
    /// All other inodes must have an non-empty name.
    fn name(&self) -> SysStr;

    /// Returns the parent of a node.
    /// 
    /// If the node has no 
    fn parent(&self) -> Weak<dyn SysBranchNode>;

    /// Returns whether a node is the root of a `SysTree`.
    fn is_root(&self) -> bool {
        return false;
    }

    /// Returns the path from the root to this node.
    /// 
    /// The path of a node is the names of all the ancestors concatenated
    /// with `/` as the separator.
    /// 
    /// If the node has been attached to a `SysTree`,
    /// then the returned path begins with `/`.
    /// Otherwise, the returned path does _not_ begin with `/`.
    fn path(&self) -> String {
        todo!("implement with the parent and name methods")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SysNodeId(u64);

impl SysNodeId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        let next_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        // Guard against integer overflow
        assert!(next_id <= u64::MAX / 2);

        Self(next_id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

pub type SysStr = Cow<'static, str>;