use std::collections::HashMap;
use serde_json::Value;


pub enum EventType {
    FileCreated,
    FileModified,
    FileDeleted,
}


pub struct Event {
    pub event_type: EventType,
    pub properties: HashMap<String, Value>,
}

