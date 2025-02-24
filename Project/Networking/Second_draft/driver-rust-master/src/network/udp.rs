use std::net::UdpSocket;
use std::io;
use crate::config::config;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;


/// Creates and configures a UDP socket for sending and receiving messages.
///
/// ## Functionality:
/// - Binds the socket to `0.0.0.0:<udp_port>`, allowing it to listen on all network interfaces.
/// - Enables **broadcasting**, allowing messages to be sent to `255.255.255.255`.
/// - Sets the socket to **non-blocking mode**, so `recv_from()` does not block execution.
///
/// ## Returns:
/// - `Ok(UdpSocket)`: A configured UDP socket ready for use.
/// - `Err(io::Error)`: If socket creation fails.
///
/// ## Errors:
/// - Panics if binding to the port fails (`expect("Failed to bind socket")`).
/// ```
pub fn create_udp_socket() -> io::Result<Arc<UdpSocket>> {
    let bind_addr = format!("0.0.0.0:{}", config::udp_port); 
    let socket = Arc::new(UdpSocket::bind(&bind_addr).expect("Failed to bind socket"));

    socket.set_broadcast(true)?; 
    socket.set_nonblocking(true)?;

    //println!("[UDP] Socket created on {}", bind_addr);
    Ok(socket)
}




pub fn broadcast_udp_message<T: Serialize>(socket: &Arc<UdpSocket>, message: &T,) -> io::Result<()> {
    let serialized = serde_json::to_string(message)?;
    let broadcast_addr = format!("{}:{}", config::broadcast_ip, config::udp_port);

    socket.send_to(serialized.as_bytes(), &broadcast_addr)?;

    Ok(())
}



pub fn send_udp_message<T: Serialize>(socket: &Arc<UdpSocket>, message: &T, target_ip: &str,) -> io::Result<()> {
    let serialized = serde_json::to_string(message)?;
    let target_addr = format!("{}:{}", target_ip, config::udp_port);

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
                println!("[UDP] Failed to deserialize message from {}", sender_addr);
            }
        }
        Err(e) => {
            if e.kind() != io::ErrorKind::WouldBlock {
                println!("[UDP] Error receiving message: {:?}", e);
            }
        }
    }

    None
}



