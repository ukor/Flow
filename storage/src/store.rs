use std::error::Error;

use crate::types::{Query, Response};



pub trait Store<T> {

    fn process(&self, query: Query) -> Result<Response<T>, Box<dyn Error>>;

}


