use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};

use clap::Parser;

use keylogger::client_input::{Commands, Cli};


fn main() -> std::io::Result<()> {
    
    let cli = Cli::parse();

    let socket_path = cli.socket;
    let mut stream = match UnixStream::connect(socket_path) {
        Ok(s) => s,
        Err(e) => {
            println!("Error: Socket is missing. Server uses a different socket or is offline");
            return Err(e);
        }
    };
    match send_command(&mut stream, cli.command) {
        Ok(_) => if cli.verbose {println!("Successfully sent command.")},
        Err(e) => {
            if cli.verbose {
                println!("Failed to send command")
            } else {
                println!("Failed to send command: {:?}", e)
            }
        }
    }
    stream.shutdown(std::net::Shutdown::Write)?;
    Ok(())
}

fn send_command(stream: &mut UnixStream, command: Commands) -> Result<(), &'static str> {
    let encoded_cmd = match bincode::serialize(&command) {
        Ok(serialized) => serialized,
        Err(e) => return Err("Cannot serialize command"),
    };

    let len = encoded_cmd.len() as u32;

    match stream.write_all(&len.to_be_bytes()) {
        Ok(_) => (),
        Err(e) => return Err("Failed to write command header")
    };

    match stream.write_all(&encoded_cmd) {
        Ok(_) => (),
        Err(e) => return Err("Failed to write command payload")
    };

    Ok(())
}
