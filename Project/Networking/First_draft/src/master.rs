use crate::config::*;
use crate::message_variables::*;
use crate::networking::*;
use std::thread;
use std::time::Duration;
use std::net::UdpSocket;


pub fn start_master() {
    println!("[Master] Running...");

    thread::spawn(send_heartbeat);
    thread::spawn(listen_for_slaves);
    listen_for_orders();
}

// Master sends heartbeat every 2 seconds
fn send_heartbeat() {
    loop {
        let master_state = State {
            id: MASTER_IP.to_string(),  // Added ID field
            obstructed: false,
            motorstop: false,
            behaviour: Behaviour::Idle,
            floor: 0,
            direction: 0,
        };        
        send_message(&master_state, BROADCAST_IP, MASTER_HEARTBEAT_PORT);
        thread::sleep(Duration::from_secs(2));
    }
}

// Master receives updates from slaves
fn listen_for_slaves() {
    let socket = UdpSocket::bind(format!("{}:{}", BROADCAST_IP, MASTER_HEARTBEAT_PORT)).expect("Failed to bind UDP socket");


    loop {
        if let Some(state) = receive_message::<State>(&socket) {
            if state.id == MASTER_IP {
                continue;
            }
            println!("[Master] Received state update: {:?}", state);
        }
    }
}


// Master receives order requests & assigns orders
fn listen_for_orders() {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", SLAVE_HEARTBEAT_PORT))
        .expect("Failed to bind UDP socket");
    
    loop {
        if let Some(order) = receive_message::<OrderMessage>(&socket) {        
            println!("[Master] Received order request: {:?}", order);
            send_message(&order, &order.id, ORDER_ASSIGNMENT_PORT);
        }
    }
}

