#![allow(dead_code)]
use crate::config::config;
use crate::distributor::distributor::{COMPLETED_ORDER, NEW_ORDER};
use crate::elevio::elev::{CAB, DIRN_STOP, HALL_DOWN};
use crate::elevator_controller::elevator_fsm::{State, Behaviour};
use crate::elevio::poll::CallButton;
use crate::network::udp;
use crate::distributor::distributor::Message;
use crossbeam_channel as cbc;
use std::net::UdpSocket;
use std::sync::Arc;
use serde_json;
use std::sync::Mutex;

pub fn transmitter(
    elevator_id: u8,
    new_state_rx: cbc::Receiver<State>,
    master_transmit_rx: cbc::Receiver<String>,
    call_msg_rx: cbc::Receiver<(u8, CallButton)>,
    pending_orders: Arc<Mutex<Vec<(u8, CallButton)>>>,
    socket: Arc<UdpSocket>,
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
    let pending_orders_ticker = cbc::tick(config::PENDING_ORDERS_TRANSMIT_PERIOD);

    loop {
        cbc::select! {
            recv(new_state_rx) -> a => {
                let new_state = a.unwrap();
                state = new_state;
            },
            recv(call_msg_rx) -> a => {
                let (msg_type, call) = a.unwrap();
                let msg = Message::CallMsg((elevator_id, [msg_type, call.floor, call.call]));
                broadcast_message(&socket, &msg);
                pending_orders.lock().unwrap().push((msg_type, call));
            },
            recv(pending_orders_ticker) -> _ => {
                pending_orders.lock().unwrap().iter().for_each(|(msg_type, call)| {
                    let msg = Message::CallMsg((elevator_id, [*msg_type, call.floor, call.call]));
                    broadcast_message(&socket, &msg);
                });
            },
            recv(state_ticker) -> _ => {
                let msg = Message::StateMsg((elevator_id, state.clone()));
                broadcast_message(&socket, &msg);
            },
            recv(master_transmit_rx) -> a => {
                let assigned_orders_str = a.unwrap();
                let all_assigned_orders_str: serde_json::Value = serde_json::from_str(&assigned_orders_str).expect("Failed");
                let msg = Message::AllAssignedOrdersMsg((elevator_id, all_assigned_orders_str));

                broadcast_message(&socket, &msg);
            }
        }
    }
}

pub fn broadcast_message(socket: &Arc<UdpSocket>, message: &Message) {
    let message_json = serde_json::to_string(message).unwrap();
    let _ = udp::broadcast_udp_message(&socket, &message_json);
}
