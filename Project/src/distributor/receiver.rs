#![allow(dead_code)]
use crate::distributor::distributor::{COMPLETED_ORDER, NEW_ORDER};
use crate::elevator_controller::orders;
use crate::elevator_controller::orders::{AllOrders, Orders};
use crate::elevio::poll::CallButton;
use crate::network::udp;
use crate::network::udp::*;
use crossbeam_channel as cbc;
use serde::{Deserialize, Serialize};
use crate::elevator_controller::elevator_fsm::{State, Behaviour};
use std::net::UdpSocket;
use std::sync::Arc;
use serde_json;

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
                    Message::State(state_json) => {
                        match serde_json::from_str::<State>(&state_json) { // Need to find out how the state of all elevators should be stored and how to infer an elevators ID - the task description states that id should be passed as a commad line argument
                            Ok(state) => {
                                println!("Received state: {:?} from {}", state, sender_addr);
                            }
                            Err(e) => {
                                println!("Failed to deserialize state: {:?} from {}", e, sender_addr);
                            }
                        }
                    }
                }
            } else {
                println!("Failed to deserialize message from {}", sender_addr);
            }
        }
    }
}
