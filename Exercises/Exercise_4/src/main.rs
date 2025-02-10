use std::net::UdpSocket;
use std::time::{Duration, Instant};
use std::thread;

const BROADCAST_IP: &str = "10.100.23.255"; // Correct subnet broadcast
const PRIMARY_PORT: &str = "0.0.0.0:8000";  // Listen on all interfaces
const BACKUP_PORT: &str = "0.0.0.0:8001";   
const TIMEOUT: Duration = Duration::from_secs(5); // 5-second timeout


fn main() {
    println!("[System] Starting...");
    
    // Try to detect if a primary is already active
    if detect_primary() {
        println!("[System] Primary detected, acting as backup.");
        backup_mode();
    } else {
        println!("[System] No primary detected, becoming primary.");
        primary_mode(0);
    }
}

/// Tries to detect if an active primary exists by listening for UDP messages
fn detect_primary() -> bool {
    let socket = UdpSocket::bind(BACKUP_PORT).expect("Failed to bind detection socket");
    socket.set_read_timeout(Some(Duration::from_secs(3))).expect("Failed to set timeout");

    let mut buffer = [0; 1024];

    match socket.recv_from(&mut buffer) {
        Ok(_) => {
            println!("[System] Primary detected!");
            true // Primary is active
        }
        Err(_) => {
            println!("[System] No primary detected.");
            false // No primary detected
        }
    }
}

/// Primary mode: Sends UDP messages to announce itself
fn primary_mode(start_count: u32) {
    let socket = UdpSocket::bind(PRIMARY_PORT).expect("Failed to bind primary socket");
    socket.set_broadcast(true).expect("Failed to enable broadcast");

    let mut i = start_count;
    println!("[Primary] Broadcasting on {}", PRIMARY_PORT);

    loop {
        i += 1;
        let message = i.to_string();
        socket.send_to(message.as_bytes(), (BROADCAST_IP, 8001)).expect("Failed to send message");

        println!("[Primary] Count: {}", i);
        thread::sleep(Duration::from_secs(1));
    }
}


/// Backup mode: Listens for primary messages and takes over if needed
fn backup_mode() {
    let socket = UdpSocket::bind(BACKUP_PORT).expect("Failed to bind backup socket");
    println!("[Backup] Listening on {}", BACKUP_PORT);

    let mut buffer = [0; 1024];
    let mut latest_count = 0;
    let mut last_message_time = Instant::now();

    loop {
        socket
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("Failed to set timeout");

        match socket.recv_from(&mut buffer) {
            Ok((size, _)) => {
                let message = String::from_utf8_lossy(&buffer[..size]);
                if let Ok(count) = message.parse::<u32>() {
                    latest_count = count;
                    last_message_time = Instant::now();
                    println!("[Backup] Received count: {}", latest_count);
                }10.100.23.255
            }
            Err(_) => {
                if last_message_time.elapsed() > TIMEOUT {
                    println!("[Backup] Primary not responding. Taking over...");
                    
                    drop(socket);
                    primary_mode(latest_count);
                    return;
                }
            }
        }
    }
}