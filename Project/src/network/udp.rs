use crate::config::config;


use serde;
use socket2;
use std::sync;
use std::net;
use std::io;


pub fn create_socket() -> io::Result<sync::Arc<net::UdpSocket>> {
    let bind_address = format!("0.0.0.0:{}", config::UDP_PORT);
    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, Some(socket2::Protocol::UDP))?;
    
    socket.set_reuse_address(true)?;
    socket.bind(&bind_address.parse::<net::SocketAddr>().unwrap().into())?;
    

    let socket = sync::Arc::new(net::UdpSocket::from(socket));
    socket.set_broadcast(true)?;
    socket.set_nonblocking(true)?;

    Ok(socket)
}

pub fn broadcast_message<T: serde::Serialize>(socket: &sync::Arc<net::UdpSocket>, message: &T) -> io::Result<()> {
    let serialized = serde_json::to_string(message)?;
    let broadcast_address = format!("{}:{}", config::BROADCAST_IP, config::UDP_PORT);

    socket.send_to(serialized.as_bytes(), &broadcast_address)?;

    Ok(())
}

pub fn receive_message<T: serde::de::DeserializeOwned>(socket: &sync::Arc<net::UdpSocket>) -> Option<(T, String)> {
    let mut buffer = [0; 1024];

    match socket.recv_from(&mut buffer) {
        Ok((size, sender_address)) => {
            if let Ok(message) = serde_json::from_slice::<T>(&buffer[..size]) {
                return Some((message, sender_address.to_string()));
            }
        }
        Err(e) => {
            println!("Failed to receive message in network::udp and received this error: {:#?}", e);
        }
    }
    None
}