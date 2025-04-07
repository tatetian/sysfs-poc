//! A set of reference implementations for nodes in a `SysTree`.

use crate::{attr::SysAttrSet, node::SysNodeId};

pub struct SysObjFields {
    id: SysNodeId,
    name: SysStr,
}

impl SysObjFields {
    pub fn new(name: SysStr) -> Self {
        Self {
            id: SysNodeId::new(),
            name,
        }
    }

    pub fn id(&self) -> &SysNodeId {
        &self.id
    }

    pub fn name(&self) -> &str {
        self.name.deref()
    }
}

pub struct SysNormalNodeFields {
    base: StdObjFields,
    attr_set: SysAttrSet,
}

impl SysNormalNodeFields {
    pub fn new(name: SysStr, attr_set: SysAttrSet) -> Self {
        Self {
            base: StdObjFields::new(name),
            attr_set,
        }
    }

    pub fn id(&self) -> &SysNodeId {
        self.base.id()
    }

    pub fn name(&self) -> &str {
        self.base.name()
    }

    pub fn attr_set(&self) -> &SysAttrSet {
        &self.attr_set
    }
}

pub struct SysBranchNodeFields<C: ?Sized> {
    base: SysNormalNodeFields,
    pub children: RwMutex<BTreeMap<SysStr, Arc<C>>>,
}

impl<C: SysObj + ?Sized> SysBranchNodeFields<C> {
    pub fn contains(&self, child_name: &str) -> bool {
        let children = self.children.read();
        children.contains_key(child_name)
    }

    pub fn add_child(&self, new_child: Arc<C>) -> Result<()> {
        let mut children = self.children.write();

        let name = new_child.name();
        if children.contains_key(name) {
            return Err(...);
        }

        children.insert(name.clone(), new_child);
        Ok(())
    }

    pub fn remove_child(&self, child_name: &str) -> Option<Arc<C>> {
        let mut children = self.children.write();
        children.remove(child_name)
    }
}


/// A reference implementation for a symlink node.
pub struct SymlinkNode {
    base: StdObjFields,
    // The properties specific to a SysSymlink
    target_path: String,
    target_node: Weak<dyn SysNode>,
}

impl SysSymlinkImpl {

}
