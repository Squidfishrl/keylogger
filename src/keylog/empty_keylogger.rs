use std::sync::{Arc, Mutex};

use super::keylogger::{KeyRecord, Keylogger};
use crate::observers::pub_sub::{Event, Publisher, Subscriber};

pub struct EmptyKeylogger {}

impl EmptyKeylogger {
    pub fn new() -> Result<Self, &'static str> {
        Ok(EmptyKeylogger {})
    }
}

impl Keylogger for EmptyKeylogger {
    fn record_keystrokes(&mut self) -> Result<(), &'static str> {
        Ok(())
    }

    fn stop(&mut self) -> Result<Vec<KeyRecord>, &'static str> {
        Err("No err, just simpler to return")
    }
}

impl Publisher<KeyRecord> for EmptyKeylogger {
    fn subscribe(&mut self, _event: Event, _listener: Arc<Mutex<dyn Subscriber<KeyRecord>>>) {
        return;
    }

    fn unsubscribe(&mut self, _event: &Event, _listener: &Arc<Mutex<dyn Subscriber<KeyRecord>>>) {
        return;
    }

    fn notify(&self, _event: &Event, _data: &KeyRecord) {
        return;
    }
}
