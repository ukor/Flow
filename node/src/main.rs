pub mod bootstrap;


use std::error::Error;


fn main() -> Result<(), Box<dyn Error>>{
    bootstrap::init::initialize()?;

    Ok(())
}


