#![allow(dead_code)]
use crate::config::config;
use crate::elevator_controller::orders;
use crate::elevio::poll::CallButton;
use crossbeam_channel as cbc;
use std::sync::Arc;
use std::thread::*;
use std::time::*;

pub fn distributor(
    new_state_rx: cbc::Receiver<State>,
    order_finished_rx: cbc::Receiver<CallButton>,
    new_order_tx: cbc::Sender<orders::Orders>,
) {
    let socket = udp::create_udp_socket().expect("Failed to create UDP socket");
    let socket_receiver = Arc::clone(&socket);
    let socket_transmitter = Arc::clone(&socket);

    let (master_activate_tx, master_activate_rx) = cbc::unbounded::<()>();

    {
        spawn(move || receiver::receiver(new_order_tx, master_activate_tx, socket_receiver));
    }
    {
        spawn(move || {
            transmitter::transmitter(
                new_state_rx,
                order_finished_rx,
                master_activate_rx,
                socket_transmitter,
            )
        });
    }
}
