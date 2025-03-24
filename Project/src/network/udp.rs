use crate::config::config;

use std::sync;
use std::net;
use std::io;
use serde;
use socket2;


pub fn create_udp_socket() -> io::Result<sync::Arc<net::UdpSocket>> {
    let bind_address = format!("0.0.0.0:{}", config::UDP_PORT);
    let socket = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, Some(socket2::Protocol::UDP))?;
    
    socket.set_reuse_address(true)?;
    socket.bind(&bind_address.parse::<net::SocketAddr>().unwrap().into())?;
    

    let socket = sync::Arc::new(net::UdpSocket::from(socket));
    socket.set_broadcast(true)?;
    socket.set_nonblocking(true)?;

    Ok(socket)
}

pub fn broadcast_udp_message<T: serde::Serialize>(socket: &sync::Arc<net::UdpSocket>, message: &T) -> io::Result<()> {
    let serialized = serde_json::to_string(message)?;
    let broadcast_address = format!("{}:{}", config::BROADCAST_IP, config::UDP_PORT);

    socket.send_to(serialized.as_bytes(), &broadcast_address)?;

    Ok(())
}

pub fn send_udp_message<T: serde::Serialize>(
    socket: &sync::Arc<net::UdpSocket>,
    message: &T,
    target_ip: &str,
) -> io::Result<()> {
    let serialized = serde_json::to_string(message)?;
    let target_address = format!("{}:{}", target_ip, config::UDP_PORT);

    socket.send_to(serialized.as_bytes(), &target_address)?;

    Ok(())
}

pub fn receive_udp_message<T: serde::de::DeserializeOwned>(socket: &sync::Arc<net::UdpSocket>) -> Option<(T, String)> {
    let mut buffer = [0; 1024];

    match socket.recv_from(&mut buffer) {
        Ok((size, sender_address)) => {
            if let Ok(message) = serde_json::from_slice::<T>(&buffer[..size]) {
                return Some((message, sender_address.to_string()));
            } else {

            }
        }
        Err(e) => {
            
        }
    }
    None
}