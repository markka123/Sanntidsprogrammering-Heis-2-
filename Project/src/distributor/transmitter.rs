#![allow(dead_code)]
use crate::config::config;
use crate::distributor::distributor::{COMPLETED_ORDER, NEW_ORDER};
use crate::elevio::elev::{CAB, DIRN_STOP, HALL_DOWN};
use crate::elevator_controller::elevator_fsm::{State, Behaviour};
use crate::elevio::poll::CallButton;
use crate::network::udp;
use crossbeam_channel as cbc;
use std::net::UdpSocket;
use std::sync::Arc;
use serde_json;

use super::receiver::Message;

pub fn transmitter(
    call_button_rx: cbc::Receiver<CallButton>,
    new_state_rx: cbc::Receiver<State>,
    order_completed_rx: cbc::Receiver<CallButton>,
    master_activate_rx: cbc::Receiver<()>,
    socket: Arc<UdpSocket>,
    master_ip: &str,
) {
    let mut state = State {
        obstructed: false,
        motorstop: false,
        emergency_stop: false,
        behaviour: Behaviour::Idle,
        floor: 0,
        direction: HALL_DOWN,
    };

    let state_ticker = cbc::tick(config::STATE_TRANSMIT_PERIOD);

    let master_ticker = cbc::never();

    loop {
        cbc::select! {
            recv(new_state_rx) -> a => {
                let new_state = a.unwrap();
                state = new_state;
                println!("State updated!");
            },
            recv(order_completed_rx) -> a => {
                let call = a.unwrap();
                let msg_type = COMPLETED_ORDER;
                broadcast_order(&socket, call, msg_type, &master_ip);
            },
            recv(call_button_rx) -> a => {
                let call = a.unwrap();
                let msg_type = NEW_ORDER;
                broadcast_order(&socket, call, msg_type, &master_ip);
            },
            recv(state_ticker) -> _ => {
                broadcast_state(&socket, &state, &master_ip);
            },
            recv(master_activate_rx) -> _ => {
                master_ticker = cbc::ticker(config::MASTER_TRANSMIT_PERIOD);
            }
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
pub fn broadcast_state(socket: &Arc<UdpSocket>, state: &State, master_ip: &str) {
    let state_json = serde_json::to_string(state).unwrap();
    let msg = Message::State(state_json);
    let _ = udp::broadcast_udp_message(&socket, &msg);
}
