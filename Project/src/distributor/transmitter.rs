#![allow(dead_code)]
use crate::config::config;
use crate::distributor::distributor::{COMPLETED_ORDER, NEW_ORDER};
use crate::elevator_controller::elevator_fsm::State;
use crate::elevio::poll::CallButton;
use crate::network::udp;
use crossbeam_channel as cbc;
use std::net::UdpSocket;
use std::sync::Arc;

use super::receiver::Message;

pub fn transmitter(
    call_button_rx: cbc::Receiver<CallButton>,
    new_state_rx: cbc::Receiver<State>,
    order_completed_rx: cbc::Receiver<CallButton>,
    master_activate_rx: cbc::Receiver<()>,
    socket: Arc<UdpSocket>,
    master_ip: &str,
) {
    let state = Message::State(String::from("hey")); //PLACEHOLDER: Replace with a state type
    let state_ticker = cbc::tick(config::STATE_TRANSMIT_PERIOD);

    let is_master = false;

    loop {
        cbc::select! {
            // recv(new_state_rx) -> a => {
            //     let new_state = a.unwrap();
            //     state = call;
            // },
            recv(order_completed_rx) -> a => {
                let call = a.unwrap();
                let msg_type = COMPLETED_ORDER;
                broadcast_order(&socket, call, msg_type, &master_ip);
            },
            recv(call_button_rx) -> a => {
                let call = a.unwrap();
                let msg_type = NEW_ORDER;
                broadcast_order(&socket, call, msg_type, &master_ip);
                // println!("Sendt message!");
            },
            recv(state_ticker) -> _ => {
                // println!("Broadcasting state: {:?}", state);
                broadcast_state(&socket, &state, &master_ip);
            },
            // recv(master_activate_rx) -> _ => {
            //     is_master = true;
            // }
        }
    }
}

// let msg_call = "Hello World";
// udp::broadcast_udp_message(&socket, &msg_call);

// let msg_delivered = [1, delivered.floor, delivered.call];
// udp::broadcast_udp_message(&socket, &msg_delivered);

pub fn broadcast_order(socket: &Arc<UdpSocket>, call: CallButton, msg_type: u8, master_ip: &str) {
    let msg = Message::Call([msg_type, call.floor, call.call]);
    let _ = udp::broadcast_udp_message(&socket, &msg);
}
pub fn broadcast_state(socket: &Arc<UdpSocket>, state: &Message, master_ip: &str) {
    let _ = udp::send_udp_message(&socket, &state, &master_ip);
}
