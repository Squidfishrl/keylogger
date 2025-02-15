use crate::keylog::keylogger::KeyRecord;


#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Event {
    KeyPress,
}

trait Subscriber<T> {
    fn on_event(self, event: &Event, data: &T);
}

trait Publisher<T> {
    fn subscribe(&mut self, event: Event, listener: Box<dyn Subscriber<T>>);
    fn unsubscribe(&mut self, event: Event, listener: &dyn Subscriber<T>);
    fn notify(&self, event: Event, data: &T);
}


#[derive(Default)]
pub struct BasicPublisher<T>{
    subscribers: HashMap<Event, Vec<Box<dyn Subscriber<T>>>>
}

impl<T> Publisher<T> for BasicPublisher<T> where dyn Subscriber<T>: PartialEq<dyn Subscriber<T>> {
    fn subscribe(&mut self, event: Event, listener: Box<dyn Subscriber<T>>) {
        self.subscribers.entry(event).or_insert_with(Vec::new).push(listener);
        //self.subscribers.entry(event.clone()).or_default();
        //self.subscribers.get_mut(&event).unwrap().push(listener);
    }

    fn unsubscribe(&mut self, event: Event, listener: &dyn Subscriber<T>) {
        if let Some(event_subs) = self.subscribers.get_mut(&event) {
            if let Some(index) = event_subs.iter().position(|sub| sub.as_ref() == listener) {
                event_subs.remove(index);
            }
        }
    }

    fn notify(&self, event_type: Event, data: &T) {
        if let Some(listeners) = self.subscribers.get(&event_type) {
            for listener in listeners.iter() {
                listener.on_event(&event_type, data.clone());
            }
        }
    }
}
