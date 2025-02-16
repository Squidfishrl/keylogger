use super::keylogger::{KeyRecord, Keylogger};

use std::collections::HashMap;
use std::process::Command;
use std::str;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;

use x11rb::connection::Connection;
use x11rb::protocol::record::{self, ConnectionExt as _, Range8, CS};
use x11rb::protocol::xproto;
use x11rb::x11_utils::TryParse;

use crate::observers::pub_sub::BasicPublisher;
use crate::observers::pub_sub::Event;
use crate::observers::pub_sub::Publisher;
use crate::observers::pub_sub::Subscriber;

use crate::command_dispatcher::{CommandDispatcher, KeyloggerCommand};
use std::sync::mpsc::{Receiver, TryRecvError};

pub struct XKeylogger {
    handle: Option<thread::JoinHandle<Vec<KeyRecord>>>,
    keymap: HashMap<u8, Vec<String>>,
    publisher: Arc<Mutex<BasicPublisher<KeyRecord>>>,
    receiver: Arc<Mutex<Receiver<KeyloggerCommand>>>,
}

impl XKeylogger {
    pub fn new(receiver: Arc<Mutex<Receiver<KeyloggerCommand>>>) -> Result<Self, &'static str> {
        let keymap = match get_keycode_keysym_pairs() {
            Ok(map) => map,
            Err(_) => {
                log::error!("Cannot get keysym table");
                return Err("Failed to get keysym table");
            }
        };

        Ok(XKeylogger {
            handle: None,
            keymap,
            publisher: Arc::new(Mutex::new(BasicPublisher::new())),
            receiver,
        })
    }
}

impl Keylogger for XKeylogger {
    fn record_keystrokes(&mut self) -> Result<(), &'static str> {
        if self.handle.is_some() {
            log::warn!("Cannot record keystrokes. Already recording.");
            return Err("keylogger is already recording");
        }

        // let receiver = Arc::clone(&self.receiver);
        let receiver = self.receiver.clone();
        let keymap = self.keymap.clone();
        let publisher = Arc::clone(&self.publisher);
        let handle = thread::spawn(move || {
            //let con2 = XCBConnection::connect(None);
            let (conn, _) = match x11rb::connect(None) {
                Ok(val) => val,
                Err(e) => {
                    log::error!("Cannot connect to X server {:?}", e);
                    panic!();
                }
            };

            let rc = match conn.generate_id() {
                Ok(val) => val,
                Err(_) => {
                    log::error!("Cannot generate a new X identifier");
                    panic!();
                }
            };

            // setup record extension for keyboard events
            let range = record::Range {
                device_events: Range8 {
                    first: xproto::KEY_PRESS_EVENT,
                    last: xproto::KEY_RELEASE_EVENT,
                },
                ..record::Range::default()
            };

            match conn.record_create_context(rc, 0, &[CS::ALL_CLIENTS.into()], &[range]) {
                Ok(_) => (),
                Err(_) => {
                    log::error!("Cannot create record context");
                    panic!();
                }
            }

            let mut event_stream = match conn.record_enable_context(rc) {
                Ok(val) => val,
                Err(_) => {
                    log::error!("Cannot enable record context");
                    panic!();
                }
            };

            let mut keys: Vec<KeyRecord> = Vec::new();

            // we don't care if the receiver is locked, because we use it only here
            let mut receiver = match receiver.lock() {
                Ok(rx) => rx,
                Err(e) => {
                    log::error!("Cannot lock receiver: {e}");
                    panic!();
                }
            };

            let receiver = &mut *receiver;
            let mut is_paused = false;

            // do until exit_flag doesn't change (only changes through stop func)
            loop {
                match receiver.try_recv() {
                    Ok(cmd) => {
                        log::info!("Received keylogger command: {:?}", cmd);
                        match cmd {
                            KeyloggerCommand::StopRecording => {
                                break;
                            }
                            KeyloggerCommand::PauseRecording => {
                                is_paused = true;
                                //conn.record_disable_context(rc);
                            }
                            KeyloggerCommand::ResumeRecording => {
                                is_paused = false;
                                //event_stream = conn.record_enable_context(rc).unwrap();
                            }
                            _ => (),
                        }
                    }
                    Err(TryRecvError::Empty) => (),
                    Err(TryRecvError::Disconnected) => {
                        log::info!("Channel disconnected.");
                        break;
                    }
                }
                match event_stream.next() {
                    Some(Ok(reply)) => {
                        if reply.category == 0 {
                            // Core events
                            let data = &reply.data[..];
                            if let Ok((event, _)) = xproto::KeyPressEvent::try_parse(data) {
                                // Modifiers are automatically detected, we
                                // don't need to add a separate keyevent for
                                // them
                                if is_modifier_key(event.detail) {
                                    continue;
                                }

                                let key_name = keymap.get(&event.detail);

                                match key_name {
                                    Some(name) => {
                                        let key_record = KeyRecord {
                                            time: event.time,
                                            key_name: name[0].clone(),
                                            modifiers: format!("{:?}", event.state),
                                            press: event.response_type == xproto::KEY_PRESS_EVENT,
                                            key_code: event.detail,
                                        };

                                        if !is_paused {
                                            keys.push(key_record.clone());
                                        }

                                        match publisher.lock() {
                                            Ok(observer) => {
                                                // publisher is unlocked only after notify finishes
                                                // notify calls subscriber function, so they must
                                                // finish as well.
                                                observer.notify(&Event::KeyPress, &key_record)
                                            }
                                            Err(_) => {
                                                log::error!("Cannot send KeyPress notification. Publisher is already locked");
                                            }
                                        };
                                    }
                                    None => continue, // UNKNOWN KEY
                                };
                            }
                        }
                    }
                    Some(Err(e)) => {
                        println!("Error receiving event: {:?}", e);
                        break;
                    }
                    None => break,
                }

                thread::sleep(std::time::Duration::from_millis(20));
            }

            match conn.record_free_context(rc) {
                Ok(_) => (),
                Err(_) => log::error!("Cannot free context"),
            };

            keys
        });

        self.handle = Some(handle);
        Ok(())
    }

    fn stop(&mut self) -> Result<Vec<KeyRecord>, &'static str> {
        match CommandDispatcher::get().send_command(KeyloggerCommand::StopRecording) {
            Ok(_) => (),
            Err(e) => {
                let msg = "Failed to send keylogger stop command";
                log::error!("{msg}: {e}");
                return Err("{msg}");
            }
        }

        let handle = match self.handle.take() {
            Some(h) => h,
            None => {
                log::warn!("Cannot stop recording. Recording isn't started.");
                return Err("Not recording keystrokes");
            }
        };

        let result = handle.join();
        match result {
            Ok(res) => Ok(res),
            Err(_) => {
                log::error!("Failed receivig keylogger data from thread");
                Err("Failed to receive keylogger data")
            }
        }
    }
}

impl Publisher<KeyRecord> for XKeylogger {
    fn subscribe(&mut self, event: Event, listener: Arc<Mutex<dyn Subscriber<KeyRecord>>>) {
        let mut observer = match self.publisher.lock() {
            Ok(publisher) => publisher,
            Err(_) => {
                log::error!("Cannot add subscriber. Publisher is locked");
                return;
            }
        };

        observer.subscribe(event, listener)
    }

    fn unsubscribe(&mut self, event: &Event, listener: &Arc<Mutex<dyn Subscriber<KeyRecord>>>) {
        let mut observer = match self.publisher.lock() {
            Ok(publisher) => publisher,
            Err(_) => {
                log::error!("Cannot add subscriber. Publisher is locked");
                return;
            }
        };

        observer.unsubscribe(event, listener)
    }

    fn notify(&self, event: &Event, data: &KeyRecord) {
        let observer = match self.publisher.lock() {
            Ok(publisher) => publisher,
            Err(_) => {
                log::error!("Cannot add subscriber. Publisher is locked");
                return;
            }
        };

        observer.notify(event, data)
    }
}

fn get_keycode_keysym_pairs() -> Result<HashMap<u8, Vec<String>>, &'static str> {
    let xmodmap_output = match Command::new("xmodmap").arg("-pke").output() {
        Ok(res) => res,
        Err(_) => {
            log::error!("Cannot execute xmodmap");
            return Err("Cannot execute xmodmap");
        }
    };

    if !xmodmap_output.status.success() {
        log::error!("xmodmap command produced an error");
        return Err("xmodmap error");
    }

    let output_str = match str::from_utf8(&xmodmap_output.stdout) {
        Ok(res) => res,
        Err(_) => {
            log::error!("Cannot convert xmodmap output to str");
            return Err("Cannot convert xmodmap output to str");
        }
    };

    let mut keymap = HashMap::new();

    for line in output_str.lines() {
        // a line looks like this:
        // keycode <8-255> = <keysym> <keysym with shift modifier> <more keysyms..>
        let parts: Vec<&str> = line.split_whitespace().collect();

        let keycode = match parts[1].parse::<u8>() {
            Ok(code) => code,
            Err(_) => continue,
        };

        let mut keysyms = Vec::new();
        for keysym in &parts[3..] {
            keysyms.push(keysym.to_string());
        }
        keymap.insert(keycode, keysyms);
    }

    Ok(keymap)
}

// TODO: use something like this to make modifier enum and use that instead of string for modifiers
fn is_modifier_key(key_code: u8) -> bool {
    match key_code {
        0x32 => true, // Shift_L
        0x3e => true, // Shift_R
        0x25 => true, // Control_L
        0x69 => true, // Control_R
        0x40 => true, // ALT_L
        0x6c => true, // ALT_R
        0xcc => true, // ALT_L again
        0xcd => true, // Meta_L
        0x4d => true, // Num_Lock
        0xcb => true, // ISO_Level5_Shift
        0xcf => true, // Hyper_L
        0x85 => true, // Super_L
        0x86 => true, // Super_R
        0xce => true, // Super_L again
        0x5c => true, // ISO_Level3_Shift
        _ => false,
    }
}
