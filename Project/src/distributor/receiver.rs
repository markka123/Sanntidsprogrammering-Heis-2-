#![allow(dead_code)]
use crate::elevator_controller::orders;
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
    new_order_tx: cbc::Sender<orders::Orders>,
    master_activate_tx: cbc::Sender<()>,
    socket: Arc<UdpSocket>,
) {
    loop {
        if let Some((received_message, sender_addr)) =
            udp::receive_udp_message::<serde_json::Value>(&socket)
        {
            if let Ok(message) = serde_json::from_value::<Message>(received_message) {
                match message {
                    Message::Call(order) => {
                        println!("Received order: {:?} from {}", order, sender_addr);
                    }
                    Message::State(state) => {
                        println!("Received state: {} from {}", state, sender_addr);
                    }
                }
            } else {
                println!("Failed to deserialize message from {}", sender_addr);
            }
        }
    }
}
