use std::collections::VecDeque;

use super::pub_sub::{Subscriber, Event};
use super::super::keylog::keylogger::KeyRecord;

pub struct PasswordProtection {
    recent_key_presses: VecDeque<KeyRecord>
}

impl Subscriber<KeyRecord> for PasswordProtection {
    fn on_event(&self, event: &Event, key: &KeyRecord) {
        // add to recentKeyPresses if not full
        // if full rotate to the right and change first element to new key
        // see if the list of all events forms what we're listening for
        // for example if the first two keysyms are s and u
    }
}


