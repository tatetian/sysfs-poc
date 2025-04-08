use crate::utils::SysBranchNodeFields;

/// A tree structure to expose the system state.
pub struct SysTree {
    root: Arc<SysTreeRoot>,
    event_hub: SysEventHub,
}

impl SysTree {
    pub(crate) fn new() -> Self {
        Self {
            root: SysTreeRoot::new(),
            event_hub: SysEventHub::new(),
        }
    }

    pub fn root(&self) -> &Arc<SysTreeRoot> {
        &self.root
    }

    pub fn register_observer(&self, observer: Weak<dyn Observer<SysEvent>>, filter: SysEventSelector)
    {
        self.event_hub.register_observer(observer, filter)
    }
    
    pub fn unregister_observer(&self, observer: Weak<dyn Observer<SysEvent>>) -> Option<Weak<dyn Observer<SysEvent>>>
    {
        self.event_hub.unregister_observer(observer)
    }
    
    pub fn publish_event(&self, node: &dyn SysNode, action: SysEventAction, details: Vec<SysEventKv>) {
        self.event_hub.publish_event(node, action, details)
    }
}

struct SysTreeRoot(SysBranchNodeFields<dyn SysNode>);

impl SysTreeRoot {
    pub fn new() -> Arc<Self> {
        let name = ""; // Only the root has an empty name
        let attr_set = SysAttrSet::new_empty(); // The root has no attributes
        let inner = SysBranchNodeFields::new(name, attr_set);
        Arc::new(Self(inner))
    }
}

#[inherit_methods(from = "self.0")]
impl SysTreeRoot {
    pub fn contains(&self, child_name: &str) -> bool;
    pub fn add_child(&self, new_child: Arc<C>) -> Result<()>;
    pub fn remove_child(&self, child_name: &str) -> Option<Arc<C>>;
}

#[inherit_methods(from = "self.0")]
impl SysBranchNode for SysTreeRoot {
    fn visit_child_with(&self,
        name: &str, 
        f: &mut dyn FnMut(Option<&dyn SysNode>)
    );
    fn visit_children_with(&self, 
        min_id: u64,
        f: &mut dyn FnMut(&dyn SysObj) -> Option<()>,
    );
    fn child(&self, name: &str) -> Option<Arc<dyn SysObj>>;
    fn children(&self) -> Vec<Arc<dyn SysObj>>;
    fn count_children(&self) -> usize;
}

#[inherit_methods(from = "self.0")]
impl SysNode for SysTreeRoot {
    fn node_attrs(&self) -> &SysAttrSet;
    fn show_attr(&self, name: &str) -> SysStr;
    fn store_attr(&self, name: &str, new_val: &str) -> Result<()>;
    fn read_attr(&self, name: &str, offset: usize, writer: VmWriter<Falliable>) -> Result<()>;
    fn write_attr(&self, name: &str, offset: usize, reader: VmReader<Falliable>) -> Result<()>;
}

#[inherit_methods(from = "self.0")]
impl SysObj for SysTreeRoot {
    fn id(&self) -> &SysNodeId;
    fn type_(&self) -> SysNodeType;
    fn name(&self) -> SysStr;
}

