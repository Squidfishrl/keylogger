use crate::client_input::Commands;
use crate::keylog::keylogger::{write_keylog_to_file, Keylogger};

pub trait State {
    fn transition(
        self: Box<Self>,
        cmd: Commands,
        keylogger: &mut Box<dyn Keylogger>,
    ) -> Box<dyn State>;
    fn get_id(&self) -> u8;
}

#[derive(PartialEq, Debug)]
pub struct IdleState {
    id: u8,
}

impl Default for IdleState {
    fn default() -> IdleState {
        IdleState { id: 0 }
    }
}

#[derive(PartialEq, Debug)]
pub struct RecordingState {
    id: u8,
}

impl Default for RecordingState {
    fn default() -> RecordingState {
        RecordingState { id: 1 }
    }
}

#[derive(PartialEq, Debug)]
pub struct PausedState {
    id: u8,
}

impl Default for PausedState {
    fn default() -> PausedState {
        PausedState { id: 2 }
    }
}

pub struct KeyLoggerFSM {
    pub state: Box<dyn State>,
}

impl KeyLoggerFSM {
    pub fn new() -> Self {
        KeyLoggerFSM {
            state: Box::new(IdleState::default()),
        }
    }
}

impl State for IdleState {
    fn transition(
        self: Box<Self>,
        cmd: Commands,
        keylogger: &mut Box<dyn Keylogger>,
    ) -> Box<dyn State> {
        match cmd {
            Commands::Record {} => {
                log::info!("Recording.");
                match keylogger.record_keystrokes() {
                    Ok(_) => (),
                    Err(e) => {
                        log::error!("Keylogger failed to record: {e}")
                    }
                }
                Box::new(RecordingState::default())
            }
            _ => {
                log::warn!("Invalid transition: {:?}", cmd);
                self
            }
        }
    }

    fn get_id(&self) -> u8 {
        self.id
    }
}

impl State for RecordingState {
    fn transition(
        self: Box<Self>,
        cmd: Commands,
        keylogger: &mut Box<dyn Keylogger>,
    ) -> Box<dyn State> {
        match cmd {
            Commands::Save { file } => {
                match keylogger.stop() {
                    Ok(keys) => {
                        log::info!("Saving recording to file {file}.");
                        match write_keylog_to_file(&file, &keys) {
                            Ok(_) => (),
                            Err(e) => log::error!("Cannot save to file: {e}"),
                        };
                    }
                    Err(e) => log::error!("Error stopping recording: {e}"),
                }

                Box::new(IdleState::default())
            }
            Commands::Pause {} => {
                log::info!("Pausing recording.");
                Box::new(PausedState::default())
            }
            _ => {
                log::warn!("Invalid transition: {:?}", cmd);
                self
            }
        }
    }

    fn get_id(&self) -> u8 {
        self.id
    }
}

impl State for PausedState {
    fn transition(
        self: Box<Self>,
        cmd: Commands,
        keylogger: &mut Box<dyn Keylogger>,
    ) -> Box<dyn State> {
        match cmd {
            Commands::Save { file } => {
                log::info!("Saving recording to file {file}.");
                Box::new(IdleState::default())
            }
            Commands::Resume {} => {
                log::info!("Resuming recording.");
                Box::new(RecordingState::default())
            }
            _ => {
                log::warn!("Invalid transition: {:?}", cmd);
                self
            }
        }
    }

    fn get_id(&self) -> u8 {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::super::keylog::empty_keylogger::EmptyKeylogger;
    use super::super::keylog::keylog_factory::{KeyloggerFact, KeyloggerFactory, KeyloggerTypes};
    use super::State;
    use super::*;

    #[test]
    fn correct_state_transitions_test() {
        let idle_state = Box::new(IdleState::default());
        let mut keylogger = KeyloggerFactory
            .create_keylogger(KeyloggerTypes::Empty)
            .unwrap();

        assert_eq!(idle_state.get_id(), 0);

        let idle_state = idle_state.transition(Commands::Resume {}, &mut keylogger);
        assert_eq!(idle_state.get_id(), 0);
        let idle_state = idle_state.transition(Commands::Pause {}, &mut keylogger);
        assert_eq!(idle_state.get_id(), 0);
        let idle_state = idle_state.transition(
            Commands::Save {
                file: "".to_string(),
            },
            &mut keylogger,
        );
        assert_eq!(idle_state.get_id(), 0);
        let rec_state = idle_state.transition(Commands::Record {}, &mut keylogger);
        assert_eq!(rec_state.get_id(), 1);

        let rec_state = rec_state.transition(Commands::Record {}, &mut keylogger);
        assert_eq!(rec_state.get_id(), 1);
        let rec_state = rec_state.transition(Commands::Resume {}, &mut keylogger);
        assert_eq!(rec_state.get_id(), 1);

        let pause_state = rec_state.transition(Commands::Pause {}, &mut keylogger);
        assert_eq!(pause_state.get_id(), 2);
        let pause_state = pause_state.transition(Commands::Record {}, &mut keylogger);
        assert_eq!(pause_state.get_id(), 2);
        let pause_state = pause_state.transition(Commands::Pause {}, &mut keylogger);
        assert_eq!(pause_state.get_id(), 2);

        let idle_state = pause_state.transition(
            Commands::Save {
                file: "".to_string(),
            },
            &mut keylogger,
        );
        assert_eq!(idle_state.get_id(), 0);
        let rec_state = idle_state.transition(Commands::Record {}, &mut keylogger);
        assert_eq!(rec_state.get_id(), 1);
        let pause_state = rec_state.transition(Commands::Pause {}, &mut keylogger);
        assert_eq!(pause_state.get_id(), 2);
        let rec_state = pause_state.transition(Commands::Resume {}, &mut keylogger);
        assert_eq!(rec_state.get_id(), 1);
        let idle_state = rec_state.transition(
            Commands::Save {
                file: "".to_string(),
            },
            &mut keylogger,
        );
        assert_eq!(idle_state.get_id(), 0);
    }
}
