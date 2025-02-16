use std::collections::VecDeque;

use super::pub_sub::{Event, Subscriber};
use crate::command_dispatcher::{CommandDispatcher, KeyloggerCommand};
use crate::keylog::keylogger::KeyRecord;

#[derive(Debug, PartialEq)]
pub enum Hotkeys {
    SU,
    CTRL_A_W,
}

pub struct HotkeyManager {
    recent_key_presses: VecDeque<KeyRecord>,
    max_hotkey_size: u8,
}

impl HotkeyManager {
    pub fn new(max_hotkey_size: u8) -> HotkeyManager {
        HotkeyManager {
            recent_key_presses: VecDeque::new(),
            max_hotkey_size,
        }
    }

    fn execute_hotkey_action(&self) {
        let hotkey = Hotkeys::get_hotkey(&self.recent_key_presses);

        match hotkey {
            Some(Hotkeys::SU) => {
                log::debug!("SU hotkey received");
                CommandDispatcher::get().send_command(KeyloggerCommand::PauseRecording);
            }
            Some(Hotkeys::CTRL_A_W) => {
                log::debug!("CTRL-A-W hotkey received");
                CommandDispatcher::get().send_command(KeyloggerCommand::ResumeRecording);
            }
            Some(other) => log::warn!("Cannot handle hotkey type {:?}", other),
            None => log::trace!("No hotkey detected"), // No hotkey received
        }
    }
}

impl Subscriber<KeyRecord> for HotkeyManager {
    fn on_event(&mut self, event: &Event, key: &KeyRecord) {
        log::trace!("hotkey manager event received: {:?}", key);

        if *event != Event::KeyPress {
            log::trace!("event received is not from keyboard - discarding");
            return;
        }

        // only care about button press (not release) events
        if key.press == false {
            log::trace!("event received is not a key press - discarding");
            return;
        }

        if self.recent_key_presses.len() == self.max_hotkey_size.into() {
            self.recent_key_presses.pop_back();
            self.recent_key_presses.push_front(key.clone());
        } else {
            self.recent_key_presses.push_front(key.clone());
        }

        log::trace!("Current key buffer is {:?}", self.recent_key_presses);
        log::trace!("Searching for hotkey...");
        self.execute_hotkey_action();
    }
}

impl Hotkeys {
    pub fn get_hotkey(keys: &VecDeque<KeyRecord>) -> Option<Hotkeys> {
        if Hotkeys::is_su(keys) {
            Some(Hotkeys::SU)
        } else if Hotkeys::is_ctrl_a_w(keys) {
            Some(Hotkeys::CTRL_A_W)
        } else {
            None
        }
    }

    fn is_su(keys: &VecDeque<KeyRecord>) -> bool {
        // last key pressed is u, so it's the first element, second is s
        log::trace!("Checking for su hotkey...");

        let first_key = match keys.get(0) {
            Some(key) => key,
            None => return false,
        };

        log::trace!("Verified that first key exists");

        if first_key.key_name != "u" {
            return false;
        }

        log::trace!("Verified that first key is u");

        let second_key = match keys.get(1) {
            Some(key) => key,
            None => return false,
        };

        log::trace!("Verified that second key exists");

        if second_key.key_name != "s" {
            return false;
        }

        log::trace!("Verified that second key is ");

        true
    }

    fn is_ctrl_a_w(keys: &VecDeque<KeyRecord>) -> bool {
        log::trace!("Checking for ctrl-a-w hotkey...");

        let first_key = match keys.get(0) {
            Some(key) => key,
            None => return false,
        };

        log::trace!("Verified that first key exists");

        if first_key.key_name != "w" || first_key.modifiers != "CONTROL" {
            return false;
        }

        log::trace!("Verified that first key is w and has a control modifier");

        let second_key = match keys.get(1) {
            Some(key) => key,
            None => return false,
        };

        log::trace!("Verified that second key exists");

        if second_key.key_name != "a" || second_key.modifiers != "CONTROL" {
            return false;
        }

        log::trace!("Verified that second key is a and has a control modifier");

        true
    }
}
