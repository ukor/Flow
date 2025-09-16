use serde_json::Value;
use std::collections::HashMap;

pub enum EventType {
    FileCreated,
    FileModified,
    FileDeleted,
}

pub struct Event {
    pub event_type: EventType,
    pub properties: HashMap<String, Value>,
}
