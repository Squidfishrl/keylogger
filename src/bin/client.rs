use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};

include!("../client_input.rs");

fn main() -> std::io::Result<()> {
    
    let cli = Cli::parse();

    println!("verbose: {:?}", cli.verbose);
    println!("socket: {:?}", cli.socket);
    println!("command: {:?}", cli.command);


    let socket_path = cli.socket;
    let mut stream = UnixStream::connect(socket_path)?;

    let _ = send_command(&mut stream, cli.command);
    println!("sent a message. Shutting down");
    stream.shutdown(std::net::Shutdown::Write)?;
    Ok(())
}

fn send_command(stream: &mut UnixStream, command: Commands) -> std::io::Result<()> {
    let encoded_cmd = match bincode::serialize(&command) {
        Ok(serialized) => serialized,
        Err(e) => panic!("Cannot serialize command: {}", e),
    };

    let len = encoded_cmd.len() as u32;

    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(&encoded_cmd)?;

    Ok(())
}
