use std::collections::HashMap;



pub enum QueryTarget {
    KV_STORE,
    SQLITE,
    GRAPH,
}


pub struct Query {
    target: QueryTarget,
    map: HashMap<String, String>,
}


impl Query {

    pub fn from_target (target: QueryTarget,) -> Query {
        Query {
            target,
            map: HashMap::new(),
        }
    }

    pub fn from(target: QueryTarget, map: HashMap<String, String>) -> Query {
        Query {
            target, map,
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) -> &mut Query {
        self.map.insert(key.to_string(), value.to_string());
        self
    }

}

/// Response encapsulates a response from the storage layer

pub struct Response<T> {
    data: T,
}


