#![allow(dead_code)]
use crate::distributor::distributor::{COMPLETED_ORDER, NEW_ORDER};
use crate::elevator_controller::orders;
use crate::elevator_controller::orders::{AllOrders, Orders};
use crate::elevio::poll::CallButton;
use crate::network::udp;
use crate::network::udp::*;
use crossbeam_channel as cbc;
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Call([u8; 3]),
    State(String),
}

pub fn receiver(
    order_msg_tx: cbc::Sender<[u8; 3]>,
    master_activate_tx: cbc::Sender<()>,
    socket: Arc<UdpSocket>,
) {
    loop {
        if let Some((received_message, sender_addr)) =
            udp::receive_udp_message::<serde_json::Value>(&socket)
        {
            if let Ok(message) = serde_json::from_value::<Message>(received_message) {
                match message {
                    Message::Call(order_msg) => order_msg_tx.send(order_msg).unwrap(),
                    Message::State(state) => {
                        // println!("Received state: {} from {}", state, sender_addr);
                    }
                }
            } else {
                println!("Failed to deserialize message from {}", sender_addr);
            }
        }
    }
}
