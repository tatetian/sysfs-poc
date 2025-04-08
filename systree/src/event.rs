
/// An event hub is where one can publish and subscribe events in a `SysTree`.
/// 
/// 
pub struct SysEventHub {
    subject: Subject<SysEvent, SysEventSelector>,
}

impl SysEventHub {
    pub const fn new() -> Self {
        Self {
            subject: Subject::new(),
        }
    }

    pub fn publish_event(&self,
        obj: &dyn SysObj,
        action: SysEventAction,
        details: Vec<SysEventKv>
    ) {
        let Some(path) = obj.path() else {
            // The object is not attached to the systree, yet.
            // We do not allow unattached object to publish events.
            return;
        };

        let event = SysEvent::new(action, path, details);
        self.subject.notify_observers(&event);
    }

    pub fn register_observer(&self,
        observer: Weak<dyn Observer<SysEvent>>,
        filter: SysEventSelector
    ) -> Option<> {
        self.subject.register_observer(observer, filter).unwrap()
    }

    pub fn unregister_observer(&self, observer: Weak<dyn Observer<SysEvent>>)
        -> Option<Weak<dyn Observer<SysEvent>>>
    {
        self.subject.unregister_observer(observer)
    }
}

/// A selector (i.e., a filter) for events that occur in the `SysTree`.
pub enum SysEventSelector {
    // Select all events.
    All,
    // Select only events of a specific action.
    Action(SysEventAction),
}

impl EventsFilter<SysEvent> for SysEventSelector {
    fn filter(&self, event: &SysEvent) -> bool {
        match self {
            Self::All => true,
            Self::Action(action) => action == event.action(),
        }
    }
}

/// An event happens in the `SysTree`.
/// 
/// An event consists of three components:
/// * Which _action_ triggers the event (`self.action()`);
/// * On which _path_ the event occurs (`self.path()`);
/// * More _details_ about the event, encoded as key-value pairs (`self.details`).
#[derive(Clone, Debug)]
pub struct SysEvent {
    // Mandatory info
    //
    // Which action happens
    action: SysEventAction,
    // Where the event originates from
    path: String,
    // Optional details
    details: Vec<SysEventKv>,
}

impl SysEvent {
    pub fn new(action: SysEventAction, path: String, details: Vec<SysEventKv>) -> Self {
        Self {
            action,
            path,
            details,
        }
    }

    pub fn action(&self) -> SysEventAction {
        self.action
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn details(&self) -> &[SysEventKv] {
        &self.details
    }
}

/// A key-value pair of strings, which encodes information about an `SysEvent`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SysEventKv {
    pub key: SysStr,
    pub value: SysStr,
}

/// The action of an `SysEvent`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SysEventAction {
    /// Add a new node in the `SysTree`.
    Add,
    /// Remove an existing node from the `SysTree`.
    Remove,
    /// Change a node in the `SysTree`.
    Change,
}