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
    elevator_id: u8
) {

    let mut master_id = config::ELEV_NUM_ELEVATORS;
    let mut master_timer = cbc::after(config::MASTER_TIMER_DURATION);

    loop {
        cbc::select! {
            recv(master_timer) -> _ => {
                if elevator_id == master_id + 1 || (elevator_id == 0 && master_id >= config::ELEV_NUM_ELEVATORS - 1) {
                    master_activate_tx.send(()).unwrap();
                    println!("Id {} is taking over as master because master_id {} died!", elevator_id, master_id);
                }
            }
            default(config::UDP_POLL_PERIOD) => {
                if let Some((received_message, sender_addr)) = udp::receive_udp_message::<String>(&socket) {
                    //let message = serde_json::from_value::<Message>(received_message);
                    match serde_json::from_str::<Message>(&received_message) {
                        Ok(Message::StateMsg((elevator_id, state))) => {
                            message_tx.send(Message::StateMsg((elevator_id, state))).unwrap();
                        }
                        Ok(Message::CallMsg(call)) => {
                            message_tx.send(Message::CallMsg(call)).unwrap();
                        }
                        Ok(Message::AllAssignedOrdersMsg((incoming_master_id, all_assigned_orders))) => {
                            master_id = incoming_master_id;
                            master_timer = cbc::after(config::MASTER_TIMER_DURATION);
                            message_tx.send(Message::AllAssignedOrdersMsg((master_id, all_assigned_orders))).unwrap();
   
                        }
                        Err(e) => {
                            println!("ERROR: Received message with unexpected format.");
                            println!("Received: {:#?}", received_message);
                            println!("Deserialization Error: {:#?}", e);
                        }
                    }
                }
            }
        }
    }
}
