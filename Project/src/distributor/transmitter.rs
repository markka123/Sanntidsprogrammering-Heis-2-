use crate::config::config;
use crate::elevator::state;
use crate::elevio::poll;
use crate::distributor::udp_message;

use crossbeam_channel as cbc;
use std::net;
use std::sync;
use serde_json;

pub fn transmitter(
    elevator_id: u8,
    new_state_rx: cbc::Receiver<state::State>,
    master_transmit_rx: cbc::Receiver<String>,
    order_message_rx: cbc::Receiver<(u8, poll::CallButton)>,
    socket: sync::Arc<net::UdpSocket>,
) {
    let mut state: state::State = state::State::init();

    let state_ticker = cbc::tick(config::STATE_TRANSMIT_PERIOD);

    loop {
        cbc::select! {
            recv(new_state_rx) -> state_message => {
                let new_state = state_message.unwrap();
                state = new_state;
            },
            recv(order_message_rx) -> order_message => {
                let (message_type, call) = order_message.unwrap();
                let message = udp_message::UdpMessage::Order((elevator_id, [message_type, call.floor, call.call]));
                udp_message::broadcast_udp_message(&socket, &message);
            },
            recv(state_ticker) -> _ => {
                let message = udp_message::UdpMessage::State((elevator_id, state.clone()));
                udp_message::broadcast_udp_message(&socket, &message);
            },
            recv(master_transmit_rx) -> assigned_orders_message => {
                let assigned_orders_string = assigned_orders_message.unwrap();
                let all_assigned_orders_string: serde_json::Value = serde_json::from_str(&assigned_orders_string).expect("Failed");
                let message = udp_message::UdpMessage::AllAssignedOrders((elevator_id, all_assigned_orders_string));
                udp_message::broadcast_udp_message(&socket, &message);
            }
        }
    }
}

