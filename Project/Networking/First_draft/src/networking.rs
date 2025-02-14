use std::net::UdpSocket;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use crate::config::TIMEOUT_SECS;

pub fn send_message<T: Serialize>(message: &T, ip: &str, port: &str) {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");

    if ip.ends_with(".255") {
        socket.set_broadcast(true).expect("Failed to enable broadcast mode");
    }

    let serialized = serde_json::to_string(message).expect("Failed to serialize message");
    let target = format!("{}:{}", ip, port);

    println!("[Debug] Sent successfully to {}", target);
    match socket.send_to(serialized.as_bytes(), &target) {
        Ok(_) => println!("[Debug] Sent successfully to {}", target),
        Err(e) => eprintln!("[Error] Failed to send UDP message to {}: {}", target, e),
    };

    socket.send_to(serialized.as_bytes(), target).expect("Failed to send UDP message");
}

pub fn receive_message<T: DeserializeOwned>(socket: &UdpSocket) -> Option<T> {
    let mut buffer = [0; 1024];

    socket.set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
        .expect("Failed to set read timeout");

    match socket.recv_from(&mut buffer) {
        Ok((size, _)) => {
            let message: T = serde_json::from_slice(&buffer[..size])
                .expect("Failed to deserialize message");
            Some(message)
        }
        Err(_) => None, // Return None if no message received
    }
}

