pub struct KeyRecord {
    keycode: u64 
}

pub trait Keylogger {
    // records keystrokes in a different thread
    fn record_keystrokes(&mut self) -> Result<(), &'static str>;

    // Stop recording keystrokes and return result
    fn stop(&mut self) -> Result<Vec<KeyRecord>, &'static str>;
}
