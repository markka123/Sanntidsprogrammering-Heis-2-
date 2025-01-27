// Receiver script
use std::net::TcpListener;
use std::io::{self, Read, Write};

pub fn start_server(port: u16) -> io::Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", port))?;
    println!("Server listening on port {}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("New connection from {}", stream.peer_addr()?);
                let mut buffer = [0; 512];
                let bytes_read = stream.read(&mut buffer)?;
                let received_message = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("Received message: {}", received_message);

                // Echo the message back to the client
                stream.write_all(b"Message received")?;
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn main() {
    let port = 7878;
    if let Err(e) = start_server(port) {
        eprintln!("Error starting server: {}", e);
    }
}