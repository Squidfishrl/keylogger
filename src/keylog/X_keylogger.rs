use super::keylogger::{KeyRecord, Keylogger};

use std::collections::HashMap;
use std::process::Command;
use std::str;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;

use x11rb::connection::Connection;
use x11rb::protocol::record::{self, ConnectionExt as _, Range8, CS};
use x11rb::protocol::xproto;
use x11rb::x11_utils::TryParse;

use super::super::observers::password_protection;

pub struct XKeylogger {
    exit_flag: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<Vec<KeyRecord>>>,
    keymap: HashMap<u8, Vec<String>>,
}

impl XKeylogger {
    pub fn new() -> Result<Self, &'static str> {
        let keymap = match get_keycode_keysym_pairs() {
            Ok(map) => map,
            Err(_) => {
                log::error!("Cannot get keysym table");
                return Err("Failed to get keysym table");
            }
        };

        Ok(XKeylogger {
            exit_flag: Arc::new(AtomicBool::new(false)),
            handle: None,
            keymap,
        })
    }
}

impl Keylogger for XKeylogger {
    fn record_keystrokes(&mut self) -> Result<(), &'static str> {
        if self.handle.is_some() {
            log::warn!("Cannot record keystrokes. Already recording.");
            return Err("keylogger is already recording");
        }

        self.exit_flag.store(false, Ordering::Relaxed);

        let exit_flag = Arc::clone(&self.exit_flag);
        let keymap = self.keymap.clone();
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

            // do until exit_flag doesn't change (only changes through stop func)
            while !exit_flag.load(Ordering::SeqCst) {
                match event_stream.next() {
                    Some(Ok(reply)) => {
                        if reply.category == 0 {
                            // Core events
                            let data = &reply.data[..];
                            if let Ok((event, _)) = xproto::KeyPressEvent::try_parse(data) {
                                let key_name = keymap.get(&event.detail);

                                match key_name {
                                    Some(name) => {
                                        keys.push(KeyRecord {
                                            time: event.time,
                                            key_name: name[0].clone(),
                                            modifiers: format!("{:?}", event.state),
                                            press: event.response_type == xproto::KEY_PRESS_EVENT,
                                            key_code: event.detail,
                                        });
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
        self.exit_flag.store(true, Ordering::Relaxed);
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
