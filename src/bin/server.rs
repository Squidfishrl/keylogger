use std::io::prelude::*;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
include!("../client_input.rs");

fn main() -> std::io::Result<()> {

    let listener = create_socket("/tmp/keylog.socket")?;

    // accept connections
    loop {
        let (mut unix_stream, _socket_address) = listener.accept()?;
        match handle_stream(unix_stream) {
            Ok(()) => (),
            Err(e) => println!("{e}")
        }
    }

    Ok(())
}

fn handle_stream(mut stream: UnixStream) -> Result<(), &'static str> {
    let mut client_command = String::new();
    let command = match receive_command(&mut stream) {
        Ok(cmd) => cmd,
        Err(_) => return Err("Unknown command or bad serialization"),
    };

    println!("{:?}", command);

    Ok(())
}

fn create_socket(socket_path: &str) -> std::io::Result<UnixListener> {
    // delete socket file, if it exists
    if std::fs::metadata(socket_path).is_ok() {
        println!("Socket already exists, deleting it.");
        std::fs::remove_file(socket_path)?;
    }

    // create new socket
    UnixListener::bind(socket_path)
}

fn receive_command(stream: &mut UnixStream) -> Result<Commands, Box<dyn std::error::Error>> {
    // read length
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    // read payload
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer)?;

    // deserialize
    let command: Commands = bincode::deserialize(&buffer)?;
    Ok(command)
}
