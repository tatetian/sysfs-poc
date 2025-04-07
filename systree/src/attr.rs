mod super::SysStr;

/// An immutable set of attributes associated with a node in `SysTree`.
pub struct SysAttrSet {
    this_set: Option<Box<[SysAttr]>>,
    parent_set: Option<Arc<SysAttr>>,
}

impl SysAttrSet {
    pub const CAPACITY: usize = 256;

    pub const fn new_empty() -> Self {
        Self {
            this_set: None,
            parent_set: None,
        }
    }

    pub fn get(&self, attr_name: &str) -> Option<&SysAttr> {
        todo!()
    }

    pub fn contains(&self, attr_name: &str) -> bool {
        self.iter().find(|name| ).is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = SysAttr>  {
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        let this_set_len = self.this_set.map_or(0, |set| set.len());
        let parent_set_len = self.parent_set.map_or(0, |set| set.len());
        this_set_len + paretn_set_len
    }
}

pub struct SysAttrSetBuidler {
    total_attrs: u8,
    this_set: Vec<SysStr>,
    parent_set: Option<Arc<SysAttrSet>>,
}

impl SysAttrSetBuidler {
    pub fn new() -> Self {
        Self {
            total_attrs: 0,
            this_set: Vec::new(),
            parent_set: None,
        }
    }

    pub fn with_parent(parent: Arc<SysAttrSet>) -> Self {
        Self {
            total_attrs: parent.len() as u8,
            this_set: Vec::new(),
            parent_set: Some(parent),
        }    
    }

    pub fn add(&mut self, name: SysStr, flags: SysAttrFlags) -> &mut Self {
        debug_assert!(self.total_attrs < u8::MAX);

        // Ignore the attribute if it is already contained in parent_set
        if let Some(parent_set) = self.parent_set {
            if parent_set.contains(name) {
                return;
            }
        };

        // Ignore the attribute if it is already contained in this_set
        let already_added = this_set.iter().find(|old_attr| {
            old_attr.name() == name
        }).is_some();
        if already_added {
            return;
        }

        let new_attr = SysAttr {
            id: self.total_attrs,
            name,
            flags,
        };
        self.this_set.push_back(new_attr);
        self.total_attrs += 1;
    }

    pub fn build(mut self) -> SysAttrSet {
        let Self {this_set, parent_set, ..} = self;
        let new_self = SysAttrSet {
            this_set: this_set.into_boxed_slice(),
            parent_set,
        };
        new_self
    }
}

/// An attribute of a node in a `SysTree`.
#[derive(Copy, Clone, Debug)]
pub struct SysAttr {
    id: u8,
    name: SysStr,
    flags: SysAttrFlags,
}

impl SysAttr {
    pub fn new(id: u8, name: SysStr, flags: SysAttrFlags) -> Self {
        Self {
            id,
            name,
            flags,
        }
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn name(&self) -> &SysStr {
        &self.name
    }

    pub fn flags(&self) -> SysAttrFlags {
        self.flags
    }
}

bitflags! {
    /// The flags of an attribute of a node in a `SysTree`.
    pub struct SysAttrFlags: u8 {
        /// Indicates whether an attribute can be shown or read.
        const CAN_READ: u8      = 1 << 0;
        /// Indicates whether an attribute can be stored or written.
        const CAN_WRITE: u8     = 1 << 1;
        /// Indicates whether an attribute is a binary one
        /// (rather than a textual one).
        const IS_BINARY: u8     = 1 << 4;
    }
}

impl Default for SysAttrFlags {
    fn default() -> Self {
        Self::CAN_READ
    }
}
