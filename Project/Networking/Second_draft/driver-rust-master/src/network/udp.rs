use std::net::UdpSocket;
use std::io;
use crate::config::config;
use serde::Serialize;



pub fn create_udp_socket() -> io::Result<UdpSocket> {
    let bind_addr = format!("0.0.0.0:{}", udp_port); 
    let socket = UdpSocket::bind(&bind_addr).expect("Failed to bind socket");

    socket.set_broadcast(true)?; 
    socket.set_nonblocking(true)?;

    //println!("[UDP] Socket created on {}", bind_addr);
    Ok(socket)
}




pub fn broadcast_udp_message<T: Serialize>(socket: &UdpSocket, message: &T,) -> io::Result<()> {
    let serialized = serde_json::to_string(message)?;
    let broadcast_addr = format!("{}:{}", broadcast_ip, udp_port);

    socket.send_to(serialized.as_bytes(), &broadcast_addr)?;

    Ok(())
}



pub fn send_udp_message<T: Serialize>(socket: &UdpSocket, message: &T, target_ip: &str,) -> io::Result<()> {
    let serialized = serde_json::to_string(message)?;
    let target_addr = format!("{}:{}", target_ip, udp_port);

    socket.send_to(serialized.as_bytes(), &target_addr)?;

    Ok(())
}






