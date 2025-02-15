use super::keylogger::{KeyRecord, Keylogger};

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
