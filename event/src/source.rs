use std::error::Error;

use crate::types::Event;


pub trait EventListener {
    fn handle(&self, event: &Event) -> Result<(), Box<dyn Error>>;
}


pub trait EventSource {

    // Implementor must provide access to listeners storage
    fn get_listeners(&self) -> &Vec<Box<dyn EventListener>>;
    fn get_listeners_mut(&mut self) -> &mut Vec<Box<dyn EventListener>>;

    
    // Default implementations that use the storage
    fn subscribe(&mut self, listener: Box<dyn EventListener>) {
        self.get_listeners_mut().push(listener);
    }

    
    fn publish(&self, event: &Event) -> Result<(), Box<dyn Error>> {
        for listener in self.get_listeners() {
            listener.handle(event)?;
        }
        Ok(())
    }

    
    fn unsubscribe_all(&mut self) {
        self.get_listeners_mut().clear();
    }

    
    fn listener_count(&self) -> usize {
        self.get_listeners().len()
    }
    

}
