use crate::config::*;
use crate::message_variables::*;
use crate::networking::*;
use crate::master::start_master;
use std::thread;
use std::time::Duration;
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
            id: my_id.clone(), // Added ID field
            obstructed: false,
            motorstop: false,
            behaviour: Behaviour::Idle,
            floor: 2, // Example floor
            direction: 0, // Stationary
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


fn listen_for_master_heartbeat(my_id: String) {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", MASTER_HEARTBEAT_PORT))
        .expect("Failed to bind to master heartbeat port");

    let mut last_heartbeat = std::time::Instant::now();

    loop {
        if let Some(state) = receive_message::<State>(&socket) {
            if state.id == MASTER_IP {
                last_heartbeat = std::time::Instant::now();
                println!("[Slave] Master is alive.");
            }
        }

        // Check if no heartbeat received within timeout
        if last_heartbeat.elapsed().as_secs() > TIMEOUT_SECS {
            println!("[Slave] Master is unresponsive! Taking over as master...");
            become_master(my_id.clone());
            return; // Exit loop since it becomes master
        }
    }
}



fn become_master(my_id: String) {
    println!("[Backup] Promoting self to master!");

    // Update config to act as master
    let mut new_master = MASTER_IP.to_string();
    new_master.clone_from(&my_id); // Update MASTER_IP to the new master

    // Call master functions
    start_master();
}

