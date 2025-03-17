use crate::config::config;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io;
use std::net::UdpSocket;
use std::sync::Arc;
use socket2::{Socket, Domain, Type, Protocol};


pub fn create_udp_socket() -> io::Result<Arc<UdpSocket>> {
    let bind_addr = format!("0.0.0.0:{}", config::UDP_PORT);
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    
    socket.set_reuse_address(true)?;
    use std::net::SocketAddr;
    socket.bind(&bind_addr.parse::<SocketAddr>().unwrap().into())?;
    

    let socket = Arc::new(UdpSocket::from(socket));
    socket.set_broadcast(true)?;
    socket.set_nonblocking(true)?;

    Ok(socket)
}


pub fn broadcast_udp_message<T: Serialize>(socket: &Arc<UdpSocket>, message: &T) -> io::Result<()> {
    let serialized = serde_json::to_string(message)?;
    let broadcast_addr = format!("{}:{}", config::BROADCAST_IP, config::UDP_PORT);

    socket.send_to(serialized.as_bytes(), &broadcast_addr)?;

    Ok(())
}

pub fn send_udp_message<T: Serialize>(
    socket: &Arc<UdpSocket>,
    message: &T,
    target_ip: &str,
) -> io::Result<()> {
    let serialized = serde_json::to_string(message)?;
    let target_addr = format!("{}:{}", target_ip, config::UDP_PORT);

    socket.send_to(serialized.as_bytes(), &target_addr)?;

    Ok(())
}

pub fn receive_udp_message<T: DeserializeOwned>(socket: &Arc<UdpSocket>) -> Option<(T, String)> {
    let mut buf = [0; 1024];

    match socket.recv_from(&mut buf) {
        Ok((size, sender_addr)) => {
            if let Ok(message) = serde_json::from_slice::<T>(&buf[..size]) {
                return Some((message, sender_addr.to_string()));
            } else {

            }
        }
        Err(e) => {
            if e.kind() != io::ErrorKind::WouldBlock {

            }
        }
    }
    
    None
}
