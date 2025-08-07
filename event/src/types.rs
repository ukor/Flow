use std::collections::HashMap;


pub enum EventType {
    FILE_CREATED,
    FILE_MODIFIED,
    FILE_DELETED,
}


pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}


pub struct Event {
    eventType: EventType,
    properties: HashMap<String, Value>,
}

