use crate::config::*;
use crate::message_variables::*;
use crate::networking::*;
use std::thread;
use std::time::Duration;

pub fn start_slave(my_id: &str) {
    println!("[Slave] Running with ID: {}", my_id);

    let my_id_string = my_id.to_string(); // Convert to String once

    thread::spawn({
        let my_id_clone = my_id_string.clone();
        move || send_heartbeat(my_id_clone)
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

    listen_for_orders(my_id_string); // Now it's safely used
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
    loop {
        if let Some(order) = receive_message::<OrderMessage>(ORDER_ASSIGNMENT_PORT) {
            println!("[Slave] Received assigned order: {:?}", order);
        }
    }
}
