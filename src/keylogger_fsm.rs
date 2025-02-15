include!("./keylog/keylog_factory.rs");

pub trait State {
    fn transition(self: Box<Self>, cmd: Commands, keylogger: &mut Box<dyn Keylogger>) -> Box<dyn State>;
}

pub struct IdleState;
pub struct RecordingState;
pub struct PausedState;

impl State for IdleState {
    fn transition(self: Box<Self>, cmd: Commands, keylogger: &mut Box<dyn Keylogger>) -> Box<dyn State> {
        match cmd {
            Commands::Record {} => { 
                log::info!("Recording.");
                match keylogger.record_keystrokes() {
                    Ok(_) => (),
                    Err(e) => {
                        log::error!("Keylogger failed to record: {e}")
                    }
                }
                Box::new(RecordingState)
            }
            _ => {
                log::warn!("Invalid transition: {:?}", cmd);
                self
            }
        }
    }
}

impl State for RecordingState {
    fn transition(self: Box<Self>, cmd: Commands, keylogger: &mut Box<dyn Keylogger>) -> Box<dyn State> {
        match cmd {
            Commands::Save {file} => { 
                match keylogger.stop() {
                    Ok(keys) => {
                        log::info!("Saving recording to file {file}.");
                        match write_keylog_to_file(&file, &keys) {
                            Ok(_) => (),
                            Err(e) => log::error!("Cannot save to file: {e}")
                        };
                    },
                    Err(e) => log::error!("Error stopping recording: {e}")
                }

                Box::new(IdleState)
            }
            Commands::Pause {  } => {
                log::info!("Pausing recording.");
                Box::new(PausedState)
            }
            _ => {
                log::warn!("Invalid transition: {:?}", cmd);
                self
            }
        }
    }
}

impl State for PausedState {
    fn transition(self: Box<Self>, cmd: Commands, keylogger: &mut Box<dyn Keylogger>) -> Box<dyn State> {
        match cmd {
            Commands::Save {file} => { 
                log::info!("Saving recording to file {file}.");
                Box::new(IdleState)
            }
            Commands::Resume {  } => {
                log::info!("Resuming recording.");
                Box::new(RecordingState)
            }
            _ => {
                log::warn!("Invalid transition: {:?}", cmd);
                self
            }
        }
    }
}
