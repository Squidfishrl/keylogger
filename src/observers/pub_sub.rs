use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Event {
    KeyPress,
}

pub trait Subscriber<T: Clone>: Send + Sync {
    fn on_event(&mut self, event: &Event, data: &T);
}

pub trait Publisher<T: Clone> {
    fn subscribe(&mut self, event: Event, listener: Arc<Mutex<dyn Subscriber<T>>>);
    fn unsubscribe(&mut self, event: &Event, listener: &Arc<Mutex<dyn Subscriber<T>>>);
    fn notify(&self, event: &Event, data: &T);
}

#[derive(Default)]
pub struct BasicPublisher<T: Clone> {
    subscribers: HashMap<Event, Vec<Arc<Mutex<dyn Subscriber<T>>>>>,
}

impl<T: Clone> BasicPublisher<T> {
    pub fn new() -> Self {
        BasicPublisher {
            subscribers: HashMap::new(),
        }
    }
}

impl<T: Clone> Publisher<T> for BasicPublisher<T> {
    fn subscribe(&mut self, event: Event, listener: Arc<Mutex<dyn Subscriber<T>>>) {
        self.subscribers.entry(event).or_default().push(listener);
    }

    fn unsubscribe(&mut self, event: &Event, listener: &Arc<Mutex<dyn Subscriber<T>>>) {
        if let Some(event_subs) = self.subscribers.get_mut(&event) {
            event_subs.retain(|sub| !Arc::ptr_eq(sub, listener));
        }
    }

    fn notify(&self, event: &Event, data: &T) {
        if let Some(listeners) = self.subscribers.get(event) {
            for listener in listeners.iter() {
                match listener.lock() {
                    Ok(sub) => {
                        let mut sub = sub;
                        sub.on_event(event, data);
                    },
                    Err(_) =>  {
                        log::error!("Cannot send event to subscriber. Already busy")
                    }
                }
            }
        }
    }
}
