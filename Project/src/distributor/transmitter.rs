#![allow(dead_code)]
use crate::config::config;
use crate::elevio::poll::CallButton;
use crate::network::udp;
use crossbeam_channel as cbc;
use std::net::UdpSocket;
use std::sync::Arc;

pub fn transmitter(
    new_state_rx: cbc::Receiver<CallButton>,
    order_finished_rx: cbc::Receiver<CallButton>,
    master_activate_rx: cbc::Receiver<()>,
    socket: Arc<UdpSocket>,
) {
    let state = ""; //PLACEHOLDER: Replace with a state type
    let state_ticker = cbc::tick(config::STATE_TRANSMIT_PERIOD);

    let is_master = false;

    loop {
        // cbc::select! {
        //     recv(new_state_rx) -> a => {
        //         let new_state = a.unwrap();
        //         state = call;
        //     },
        //     recv(order_finished_rx) -> a => {
        //         let call = a.unwrap();
        //         type = "Completed";
        //         broadcast_order(call, type);
        //     },
        //     recv(call_button_rx) -> a => {
        //         let call = a.unwrap();
        //         type = "New_order";
        //         broadcast_order(call, type);
        //     },
        //     recv(state_ticker) -> _ => {
        //         broadcast_state(state);
        //     },
        //     recv(master_activate_rx) -> _ => {
        //         is_master = true;
        //     }
        // }

        // if(bcast_state) {
        //     let msg_state_bytes = bincode::serialize(&msg_state).unwrap();
        //     udp::broadcast_udp_message(socket, &msg_state_bytes);
        // }
    }
}

// let msg_call = "Hello World";
// udp::broadcast_udp_message(&socket, &msg_call);

// let msg_delivered = [1, delivered.floor, delivered.call];
// udp::broadcast_udp_message(&socket, &msg_delivered);
