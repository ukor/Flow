use std::error::Error;

use crate::types::Event;

pub trait EventListener {
    fn handle(&self, event: &Event) -> Result<(), Box<dyn Error>>;
}

#[derive(Default)]
pub struct EventListenerManager {
    listeners: Vec<Box<dyn EventListener>>,
}

impl EventListenerManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&mut self, listener: Box<dyn EventListener>) {
        self.listeners.push(listener);
    }

    pub fn publish(&self, event: &Event) -> Result<(), Box<dyn Error>> {
        for listener in &self.listeners {
            listener.handle(event)?;
        }
        Ok(())
    }

    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }

    pub fn unsubscribe_all(&mut self) {
        self.listeners.clear();
    }
}

pub trait EventSource {
    fn event_manager(&self) -> &EventListenerManager;

    fn event_manager_mut(&mut self) -> &mut EventListenerManager;

    fn subscribe(&mut self, listener: Box<dyn EventListener>) {
        self.event_manager_mut().subscribe(listener);
    }

    fn publish(&self, event: &Event) -> Result<(), Box<dyn Error>> {
        self.event_manager().publish(event)
    }

    fn unsubscribe_all(&mut self) {
        self.event_manager_mut().unsubscribe_all();
    }

    fn listener_count(&self) -> usize {
        self.event_manager().listener_count()
    }
}
