// use std::fs::{self, OpenOptions};
// use std::io::Write;
// use std::net::UdpSocket;
// use std::time::{Duration, Instant};
// use std::thread;

// const STATE_FILE: &str = "state.txt";
// const PRIMARY_PORT: &str = "127.0.0.1:8000"; // Primary broadcasts here
// const BACKUP_PORT: &str = "127.0.0.1:8001";  // Backup listens here
// const TIMEOUT: Duration = Duration::from_secs(5); // 5 seconds timeout


// fn read_last_number() -> u32 {
//     match fs::read_to_string(STATE_FILE) {
//         Ok(contents) => contents.trim().parse::<u32>().unwrap_or(0),
//         Err(_) => 0,
//     }
// }

// fn write_last_number(num: u32) {
//     let mut file = OpenOptions::new()
//         .write(true)
//         .create(true)
//         .truncate(true)
//         .open(STATE_FILE)
//         .unwrap();
//     writeln!(file, "{}", num).unwrap();
// }



// fn main() {
//     let args: Vec<String> = std::env::args().collect();

//     if args.len() > 1 && args[1] == "backup" {
//         backup_mode();
//     } else {
//         primary_mode();
//     }
// }


// fn primary_mode() {
//     let socket = UdpSocket::bind(PRIMARY_PORT).expect("Failed to bind primary socket");
//     socket.set_broadcast(true).expect("Failed to set broadcast mode");

//     let mut i = read_last_number();
//     println!("[Primary] Broadcasting on {}", PRIMARY_PORT);

//     loop {
//         i += 1;
//         let message = i.to_string();
//         socket.send_to(message.as_bytes(), BACKUP_PORT).expect("Failed to send message");

//         println!("[Primary] Count: {}", i);
//         write_last_number(i);
//         thread::sleep(Duration::from_secs(1)); // Broadcast every second
//     }
// }



// fn backup_mode() {
//     let socket = UdpSocket::bind(BACKUP_PORT).expect("Failed to bind backup socket");
//     println!("[Backup] Listening on {}", BACKUP_PORT);

//     let mut buffer = [0; 1024];
//     let mut latest_count = read_last_number(); // Start with the last known count
//     let mut last_message_time = Instant::now();

//     loop {
//         // Check for incoming messages with a timeout
//         socket.set_read_timeout(Some(Duration::from_secs(1))).expect("Failed to set timeout");

//         match socket.recv_from(&mut buffer) {
//             Ok((size, _)) => {
//                 // Parse the received message
//                 let message = String::from_utf8_lossy(&buffer[..size]);
//                 if let Ok(count) = message.parse::<u32>() {
//                     latest_count = count; // Update the latest count
//                     last_message_time = Instant::now();
//                     println!("[Backup] Received count: {}", latest_count);
//                 }
//             }
//             Err(_) => {
//                 // Timeout or no message received
//                 if last_message_time.elapsed() > TIMEOUT {
//                     println!("[Backup] Primary not responding. Taking over...");
//                     write_last_number(latest_count); // Save the latest count before taking over
//                     primary_mode(); // Become the new primary
//                     return;
//                 }
//             }
//         }
//     }
// }

use std::net::UdpSocket;
use std::time::{Duration, Instant};
use std::thread;

const PRIMARY_PORT: &str = "127.0.0.1:8000"; // Primary broadcasts here
const BACKUP_PORT: &str = "127.0.0.1:8001";  // Backup listens here
const TIMEOUT: Duration = Duration::from_secs(5); // 5 seconds timeout

fn primary_mode() {
    let socket = UdpSocket::bind(PRIMARY_PORT).expect("Failed to bind primary socket");
    socket.set_broadcast(true).expect("Failed to set broadcast mode");

    let mut i = 0; // Start counting from 0
    println!("[Primary] Broadcasting on {}", PRIMARY_PORT);

    loop {
        i += 1; // Increment the count
        let message = i.to_string();
        socket.send_to(message.as_bytes(), BACKUP_PORT).expect("Failed to send message");

        println!("[Primary] Count: {}", i);
        thread::sleep(Duration::from_secs(1)); // Broadcast every second
    }
}

fn backup_mode() {
    let socket = UdpSocket::bind(BACKUP_PORT).expect("Failed to bind backup socket");
    println!("[Backup] Listening on {}", BACKUP_PORT);

    let mut buffer = [0; 1024];
    let mut latest_count = 0;
    let mut last_message_time = Instant::now();

    loop {
        // Check for incoming messages with a timeout
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
                }
            }
            Err(_) => {
                if last_message_time.elapsed() > TIMEOUT {
                    println!("[Backup] Primary not responding. Taking over...");
                    
                    // Close the backup socket explicitly
                    drop(socket);

                    // Transition to primary mode, starting from the last count
                    primary_mode_with_start(latest_count);
                    return;
                }
            }
        }
    }
}

/// Primary mode with a starting count (used by backup during failover)
fn primary_mode_with_start(start_count: u32) {
    let socket = UdpSocket::bind(PRIMARY_PORT).expect("Failed to bind primary socket");
    socket.set_broadcast(true).expect("Failed to set broadcast mode");

    let mut i = start_count; // Start counting from the last received count
    println!("[Primary] Broadcasting on {}", PRIMARY_PORT);

    loop {
        i += 1; // Increment the count
        let message = i.to_string();
        socket.send_to(message.as_bytes(), BACKUP_PORT).expect("Failed to send message");

        println!("[Primary] Count: {}", i);
        thread::sleep(Duration::from_secs(1)); // Broadcast every second
    }
}

/// Check if a socket can be bound to the given address
fn can_bind(address: &str) -> bool {
    match UdpSocket::bind(address) {
        Ok(_) => true, // Address is free to bind
        Err(_) => false, // Address is already in use
    }
}

fn main() {
    if can_bind(PRIMARY_PORT) {
        primary_mode();
    } else if can_bind(BACKUP_PORT) {
        backup_mode();
    } else {
        panic!("Both primary and backup ports are in use. Unable to start!");
    }
}


