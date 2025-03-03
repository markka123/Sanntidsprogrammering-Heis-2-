#![allow(dead_code)]
use crate::distributor::distributor::{COMPLETED_ORDER, NEW_ORDER};
use crate::elevator_controller::orders;
use crate::elevator_controller::orders::{AllOrders, Orders};
use crate::elevio::poll::CallButton;
use crate::config::config;
use crate::distributor::distributor::{Message};
use crate::network::udp;
use crossbeam_channel as cbc;
use serde::{Deserialize, Serialize};
use crate::elevator_controller::elevator_fsm::{State, Behaviour};
use std::net::UdpSocket;
use std::sync::Arc;
use serde_json;

pub fn receiver(
    message_tx: cbc::Sender<Message>,
    master_activate_tx: cbc::Sender<()>,
    socket: Arc<UdpSocket>,
) {
    loop {
        if let Some((received_message, sender_addr)) = 
        udp::receive_udp_message::<String>(&socket) 
        {
            //let message = serde_json::from_value::<Message>(received_message);
            match serde_json::from_str::<Message>(&received_message) {
                Ok(Message::StateMsg((elevator_id, state))) => {
                    message_tx.send(Message::StateMsg((elevator_id, state))).unwrap();
                }
                Ok(Message::CallMsg(call)) => {
                    message_tx.send(Message::CallMsg(call)).unwrap();
                }
                Ok(Message::AllAssignedOrdersMsg(assigned_orders)) => {
                    message_tx.send(Message::AllAssignedOrdersMsg(assigned_orders)).unwrap();
                }
                Err(e) => {
                    println!("ERROR: Received message with unexpected format.");
                    println!("{:#?}", e);
                }
            }
        }
    }
}
