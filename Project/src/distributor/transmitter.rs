#![allow(dead_code)]
use crate::config::config;
use crate::elevio::elev;
use crate::elevator_controller::state;
use crate::elevio::poll;
use crate::network::udp;
use crate::distributor::distributor;

use crossbeam_channel as cbc;
use std::net;
use std::sync;
use serde_json;

pub fn transmitter(
    elevator_id: u8,
    new_state_rx: cbc::Receiver<state::State>,
    master_transmit_rx: cbc::Receiver<String>,
    call_msg_rx: cbc::Receiver<(u8, poll::CallButton)>,
    unconfirmed_orders: sync::Arc<sync::Mutex<Vec<(u8, poll::CallButton)>>>,
    socket: sync::Arc<net::UdpSocket>,
) {
    let mut state = state::State {
        obstructed: false,
        motorstop: false,
        offline: false,
        emergency_stop: false,
        behaviour: state::Behaviour::Idle,
        floor: 0,
        direction: elev::HALL_DOWN,
    };

    let state_ticker = cbc::tick(config::STATE_TRANSMIT_PERIOD);
    let unconfirmed_orders_ticker = cbc::tick(config::UNCONFIRMED_ORDERS_TRANSMIT_PERIOD);

    loop {
        cbc::select! {
            recv(new_state_rx) -> a => {
                let new_state = a.unwrap();
                state = new_state;
            },
            recv(call_msg_rx) -> a => {
                let (msg_type, call) = a.unwrap();
                let msg = distributor::Message::CallMsg((elevator_id, [msg_type, call.floor, call.call]));
                broadcast_message(&socket, &msg);
                unconfirmed_orders.lock().unwrap().push((msg_type, call));
            },
            recv(unconfirmed_orders_ticker) -> _ => {
                unconfirmed_orders.lock().unwrap().iter().for_each(|(msg_type, call)| {
                    let msg = distributor::Message::CallMsg((elevator_id, [*msg_type, call.floor, call.call]));
                    broadcast_message(&socket, &msg);
                });
            },
            recv(state_ticker) -> _ => {
                let msg = distributor::Message::StateMsg((elevator_id, state.clone()));
                broadcast_message(&socket, &msg);
            },
            recv(master_transmit_rx) -> a => {
                let assigned_orders_str = a.unwrap();
                let all_assigned_orders_str: serde_json::Value = serde_json::from_str(&assigned_orders_str).expect("Failed");
                let msg = distributor::Message::AllAssignedOrdersMsg((elevator_id, all_assigned_orders_str));

                broadcast_message(&socket, &msg);
            }
        }
    }
}

pub fn broadcast_message(socket: &sync::Arc<net::UdpSocket>, message: &distributor::Message) {
    let message_json = serde_json::to_string(message).unwrap();
    let _ = udp::broadcast_udp_message(&socket, &message_json);
}
