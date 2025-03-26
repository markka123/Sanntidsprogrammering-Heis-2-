use crate::config::config;
use crate::elevator::state;
use crate::elevio::poll;
use crate::distributor::udp_message;

use crossbeam_channel as cbc;
use serde_json;
use std::net;
use std::sync;

pub fn transmitter(
    new_state_rx: cbc::Receiver<state::State>,
    master_transmit_rx: cbc::Receiver<String>,
    order_message_rx: cbc::Receiver<(u8, poll::CallButton)>,
    socket: sync::Arc<net::UdpSocket>,
    elevator_id: u8,
) {
    let mut elevator_state: state::State = state::State::init();
    let state_ticker = cbc::tick(config::STATE_TRANSMIT_PERIOD);

    loop {
        cbc::select! {
            recv(new_state_rx) -> state_message => {
                let new_state = state_message.unwrap();
                elevator_state = new_state;
            },
            recv(order_message_rx) -> order_message => {
                let (order_status, order) = order_message.unwrap();
                let message = udp_message::UdpMessage::Order((elevator_id, [order_status, order.floor, order.call]));
                udp_message::broadcast_udp_message(&socket, &message);
            },
            recv(state_ticker) -> _ => {
                let message = udp_message::UdpMessage::State((elevator_id, elevator_state.clone()));
                udp_message::broadcast_udp_message(&socket, &message);
            },
            recv(master_transmit_rx) -> assigned_orders_message => {
                let all_assigned_orders_string: serde_json::Value = serde_json::from_str(&assigned_orders_message.unwrap()).expect("Failed");
                let message = udp_message::UdpMessage::AllAssignedOrders((elevator_id, all_assigned_orders_string));
                udp_message::broadcast_udp_message(&socket, &message);
            }
        }
    }
}

