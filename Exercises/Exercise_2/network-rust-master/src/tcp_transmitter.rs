// Sender script
use std::net::TcpStream;
use std::io::{self, Write};

pub fn send_message(addr: &str, message: &str) -> io::Result<()> {
    let mut stream = TcpStream::connect(addr)?;
    stream.write_all(message.as_bytes())?;
    Ok(())
}

fn main() {
    // Server IP and TCP port
    let server_addr = "10.100.23.204:33546";

    // Computer IP and port of reciever
    let message = "Connect to: 10.100.23.13:7878\0";

    if let Err(e) = send_message(server_addr, message) {
        eprintln!("Error sending message: {}", e);
    }
}