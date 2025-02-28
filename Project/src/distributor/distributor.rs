#![allow(dead_code)]
use crate::config::config;
use crate::distributor::receiver;
use crate::distributor::transmitter;
use crate::elevator_controller::elevator_fsm::State;
use crate::elevator_controller::orders;
use crate::elevio::elev as e;
use crate::elevio::poll;
use crate::elevio::poll::CallButton;
use crate::network::udp;
use crossbeam_channel as cbc;
use std::sync::Arc;
use std::thread::*;
use std::time::*;

pub const NEW_ORDER: u8 = 0;
pub const COMPLETED_ORDER: u8 = 1;

pub fn distributor(
    elevator: &e::Elevator,
    new_state_rx: cbc::Receiver<State>,
    order_completed_rx: cbc::Receiver<CallButton>,
    new_order_tx: cbc::Sender<orders::Orders>,
) {
    let socket = udp::create_udp_socket().expect("Failed to create UDP socket");
    let socket_receiver = Arc::clone(&socket);
    let socket_transmitter = Arc::clone(&socket);

    let (master_activate_tx, master_activate_rx) = cbc::unbounded::<()>();

    let master_ip = config::BROADCAST_IP;

    let (call_button_tx, call_button_rx) = cbc::unbounded::<CallButton>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::call_buttons(elevator, call_button_tx, config::POLL_PERIOD));
    }

    {
        spawn(move || receiver::receiver(new_order_tx, master_activate_tx, socket_receiver));
    }
    {
        spawn(move || {
            transmitter::transmitter(
                call_button_rx,
                new_state_rx,
                order_completed_rx,
                master_activate_rx,
                socket_transmitter,
                &master_ip,
            )
        });
    }

    loop {
        // sleep(Duration::from_millis(100));
    }
}
