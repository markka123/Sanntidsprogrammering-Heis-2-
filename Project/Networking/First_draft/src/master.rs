use crate::config::*;
use crate::message_variables::*;
use crate::networking::*;
use std::thread;
use std::time::Duration;
use std::net::{UdpSocket, SocketAddr};
use socket2::{Socket, Domain, Type};
use std::str::FromStr;

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



fn create_reusable_udp_socket(port: &str) -> UdpSocket {
    let addr = SocketAddr::from_str(&format!("0.0.0.0:{}", port))
        .expect("Invalid socket address");  // ✅ Explicit type annotation

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, None).expect("Failed to create socket");
    socket.set_reuse_address(true).expect("Failed to set SO_REUSEADDR"); // ✅ Only reuse_address
    socket.bind(&addr.into()).expect("Failed to bind UDP socket");

    socket.into()
}

fn listen_for_slaves() {
    let socket = create_reusable_udp_socket(MASTER_HEARTBEAT_PORT);

    loop {
        if let Some(state) = receive_message::<State>(&socket) {
            if state.id == MASTER_IP {
                continue;
            }
            println!("[Master] Received state update: {:?}", state);
        }
    }
}

fn listen_for_orders() {
    let socket = create_reusable_udp_socket(SLAVE_HEARTBEAT_PORT);

    loop {
        if let Some(order) = receive_message::<OrderMessage>(&socket) {
            println!("[Master] Received order request: {:?}", order);
            send_message(&order, &order.id, ORDER_ASSIGNMENT_PORT);
        }
    }
}
