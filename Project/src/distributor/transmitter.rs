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

pub fn transmitter(
    elevator_id: u8,
    call_button_rx: cbc::Receiver<CallButton>,
    new_state_rx: cbc::Receiver<State>,
    order_completed_rx: cbc::Receiver<CallButton>,
    master_transmit_rx: cbc::Receiver<String>,
    socket: Arc<UdpSocket>
) {
    let mut state = State {
        obstructed: false,
        motorstop: true,
        emergency_stop: false,
        behaviour: Behaviour::Idle,
        floor: 0,
        direction: HALL_DOWN,
    };

    let state_ticker = cbc::tick(config::STATE_TRANSMIT_PERIOD);

    loop {
        cbc::select! {
            recv(new_state_rx) -> a => {
                let new_state = a.unwrap();
                state = new_state;
                //println!("State updated!");
            },
            recv(order_completed_rx) -> a => {
                let call = a.unwrap();
                let msg_type = COMPLETED_ORDER;
                let msg = Message::CallMsg((elevator_id, [msg_type, call.floor, call.call]));
                broadcast_message(&socket, &msg);
            },
            recv(call_button_rx) -> a => {
                let call = a.unwrap();
                let msg_type = NEW_ORDER;
                let msg = Message::CallMsg((elevator_id, [msg_type, call.floor, call.call]));
                broadcast_message(&socket, &msg);
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
                // println!("Hei");
            }
        }
    }
}

pub fn broadcast_message(socket: &Arc<UdpSocket>, message: &Message) {
    let message_json = serde_json::to_string(message).unwrap();
    let _ = udp::broadcast_udp_message(&socket, &message_json);
}
