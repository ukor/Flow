use std::error::Error;
use node::bootstrap;


fn main() -> Result<(), Box<dyn Error>>{
    bootstrap::init::initialize()?;

    Ok(())
}


