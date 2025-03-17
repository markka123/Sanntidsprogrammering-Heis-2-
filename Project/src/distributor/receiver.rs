#![allow(dead_code)]
use crate::distributor::distributor;
use crate::config::config;
use crate::network::udp;

use crossbeam_channel as cbc;
use std::net;
use std::sync;
use serde_json;

pub fn receiver(
    message_tx: cbc::Sender<distributor::Message>,
    master_activate_tx: cbc::Sender<bool>,
    is_online_tx: cbc::Sender<bool>,
    socket: sync::Arc<net::UdpSocket>,
    elevator_id: u8
) {

    let mut network_timer = cbc::after(config::NETWORK_TIMER_DURATION);
    let mut master_id = config::ELEV_NUM_ELEVATORS;
    let mut master_timer = cbc::after(config::MASTER_TIMER_DURATION);

    loop {
        cbc::select! {
            recv(master_timer) -> _ => {
                if elevator_id == master_id + 1 || (elevator_id == 0 && master_id >= config::ELEV_NUM_ELEVATORS - 1) {
                    master_activate_tx.send(true).unwrap();
                    println!("Id {} is taking over as master because master_id {} died!", elevator_id, master_id);
                }
            },
            recv(network_timer) -> _ =>{
                is_online_tx.send(false).unwrap();
                master_activate_tx.send(false).unwrap();
                network_timer = cbc::never();
            },
            default(config::UDP_POLL_PERIOD) => {
                if let Some((received_message, sender_addr)) = udp::receive_udp_message::<String>(&socket) {
                    match serde_json::from_str::<distributor::Message>(&received_message) {
                        Ok(distributor::Message::StateMsg((elevator_id, state))) => {
                            network_timer = cbc::after(config::NETWORK_TIMER_DURATION);
                            is_online_tx.send(true).unwrap();
                            message_tx.send(distributor::Message::StateMsg((elevator_id, state))).unwrap();

                        }
                        Ok(distributor::Message::CallMsg(call)) => {
                            message_tx.send(distributor::Message::CallMsg(call)).unwrap();
                        }
                        Ok(distributor::Message::AllAssignedOrdersMsg((incoming_master_id, all_assigned_orders))) => {
                            master_id = incoming_master_id;
                            master_timer = cbc::after(config::MASTER_TIMER_DURATION);
                            message_tx.send(distributor::Message::AllAssignedOrdersMsg((master_id, all_assigned_orders))).unwrap();
   
                        }
                        Err(e) => {
                            // TODO: HANDLE ERROR
                        }
                    }
                }
            }
        }
    }
}
