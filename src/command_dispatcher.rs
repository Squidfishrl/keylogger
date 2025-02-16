use std::option::Option;
use std::sync::{
    mpsc::{self, Receiver, Sender},
    OnceLock,
};

#[derive(Debug, Clone)]
pub enum KeyloggerCommand {
    StopRecording,
    PauseRecording,
    ResumeRecording,
    TimeoutRecording(u16),
}

pub struct CommandDispatcher {
    tx: Sender<KeyloggerCommand>,
}

impl CommandDispatcher {
    pub fn get_or_init(
        transmitter: Option<Sender<KeyloggerCommand>>,
    ) -> &'static CommandDispatcher {
        static INSTANCE: OnceLock<CommandDispatcher> = OnceLock::new();

        INSTANCE.get_or_init(|| match transmitter {
            Some(tx) => CommandDispatcher { tx },
            None => {
                log::error!("transmission is none on init call");
                panic!();
            }
        })
    }

    pub fn get() -> &'static CommandDispatcher {
        Self::get_or_init(None)
    }

    pub fn send_command(&self, cmd: KeyloggerCommand) -> Result<(), &'static str> {
        match self.tx.send(cmd) {
            Ok(_) => Ok(()),
            Err(_) => {
                let msg = "Failed to send command to keylogger";
                log::error!("{msg}");
                Err("{msg}")
            }
        }
    }
}
