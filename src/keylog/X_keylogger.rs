include!("./keylogger.rs");

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;

pub struct XKeylogger {
    exit_flag: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<u32>>,
}

impl XKeylogger {
    fn new() -> Self {
        XKeylogger {
            exit_flag: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }
}


impl Keylogger for XKeylogger {
    fn record_keystrokes(&mut self) -> Result<(), &'static str>{
        if self.handle.is_some(){
            return Err("keylogger is already recording");
        }

        self.exit_flag.store(false, Ordering::SeqCst);

        let exit_flag = Arc::clone(&self.exit_flag);
        let handle = thread::spawn(move || {
            while !exit_flag.load(Ordering::SeqCst) {
                println!("Recording keystrokes");
                thread::sleep(std::time::Duration::from_millis(20));
            }

            42
        });

        self.handle = Some(handle);

        //self.handle = Some(thread::spawn(|| {
        //    let k1 = KeyRecord{keycode: 1};
        //    let k2 = KeyRecord{keycode: 2};
        //    while !self.exit_flag.load(Ordering::Relaxed) {
        //        // do something
        //    }
        //    42 as u32
        //    //Box::new(vec![k1, k2])
        //}));

        Ok(())
    }

    fn stop(&mut self) -> Result<Vec<KeyRecord>, &'static str> {
        self.exit_flag.store(true, Ordering::Relaxed);
        let handle = match self.handle.take() {
            Some(h) => h,
            None => {return Err("Not recording keystrokes");}
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
