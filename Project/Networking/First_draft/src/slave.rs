use crate::config::*;
use crate::message_variables::*;
use crate::networking::*;
use crate::master::start_master;
use std::thread;
use std::time::{Duration, Instant};
use std::net::UdpSocket;

pub fn start_slave(my_id: &str) {
    println!("[Slave] Running with ID: {}", my_id);

    let my_id_string = my_id.to_string();

    thread::spawn({
        let my_id_clone = my_id_string.clone();
        move || send_heartbeat(my_id_clone)
    });

    thread::spawn({
        let my_id_clone = my_id_string.clone();
        move || listen_for_master_heartbeat(my_id_clone)
    });

    thread::spawn({
        let my_id_clone = my_id_string.clone();
        let current_state = State {
            id: my_id_clone.clone(),
            obstructed: false,
            motorstop: false,
            behaviour: Behaviour::Idle,
            floor: 2, 
            direction: 0,
        };
        move || send_order_request(my_id_clone, current_state)
    });

    listen_for_orders(my_id_string);
}
    
// Slave sends heartbeat to the master every 2 seconds
fn send_heartbeat(my_id: String) {
    loop {
        let state = State {
            id: my_id.clone(),
            obstructed: false,
            motorstop: false,
            behaviour: Behaviour::Idle,
            floor: 2, 
            direction: 0, 
        };        
        send_message(&state, MASTER_IP, MASTER_HEARTBEAT_PORT);
        thread::sleep(Duration::from_secs(TIMEOUT_SECS));
    }
}

// Slave sends order request when needed
fn send_order_request(my_id: String, state: State) {
    let order = OrderMessage {
        id: my_id,
        state,
        master_id: MASTER_IP.to_string(),
    };
    send_message(&order, MASTER_IP, SLAVE_HEARTBEAT_PORT);
}

// Slave listens for assigned orders
fn listen_for_orders(_my_id: String) {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", ORDER_ASSIGNMENT_PORT))
        .expect("Failed to bind to order assignment port");

    loop {
        if let Some(order) = receive_message::<OrderMessage>(&socket) {
            println!("[Slave] Received assigned order: {:?}", order);
        }
    }
}

// Slave listens for master's heartbeat & detects failure
fn listen_for_master_heartbeat(_my_id: String) {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", SLAVE_HEARTBEAT_PORT))
        .expect("Failed to bind to master heartbeat port");

    let mut last_heartbeat = Instant::now();

    loop {
        socket.set_read_timeout(Some(Duration::from_secs(TIMEOUT_SECS)))
            .expect("Failed to set read timeout");

        if let Some(state) = receive_message::<State>(&socket) {
            if state.id == MASTER_IP {
                last_heartbeat = Instant::now();
                println!("[Slave] Master is alive.");
            }
        }

        // Check if master has been silent for too long
        if last_heartbeat.elapsed().as_secs() > TIMEOUT_SECS * 2 {
            println!("[Slave] Master is unresponsive! Taking over as master...");
            start_master();
            return;
        }
    }
}