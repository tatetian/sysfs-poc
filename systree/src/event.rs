
/// An event hub is where one can publish and subscribe events in a `SysTree`.
/// 
/// 
pub struct SysEventHub {
    subject: Subject<SysEvent, SysEventFilter>,
}

impl SysEventHub {
    pub const fn new() -> Self {
        Self {
            sub: Subject::new(),
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
        }

        let event = SysEvent::new(action, path, details);
        self.subject.notify_observers(&event);
    }

    pub fn register_observer(&self,
        observer: Weak<dyn Observer<SysEvent>>,
        filter: SysEventFilter
    ) -> Option<> {
        self.subject.register_observer(observer, filter).unwrap()
    }

    pub fn unregister_observer(&self, observer: Weak<dyn Observer<SysEvent>>)
        -> Option<Weak<dyn Observer<SysEvent>>>
    {
        self.subject.unregister_observer(observer)
    }
}


pub enum SysEventFilter {
    // Select all events.
    All,
    // Select only events of a specific action.
    Action(SysEventAction),
}

pub struct SysEvent {
    // Mandatory info
    //
    // Which action happens
    action: SysEventAction,
    // Where the event originates from
    path: SysEventStr,
    // Optional details
    details: Vec<SysEventKv>,
}

pub struct SysEventKv {
    pub key: SysEventStr,
    pub value: SysEventStr,
}

pub enum SysEventAction {
    Add,
    Remove,
    Change,
}