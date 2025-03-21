use crate::distributor::udp_message;
use crate::config::config;
use crate::network::udp;

use crossbeam_channel as cbc;
use std::net;
use std::sync;
use serde_json;

pub fn receiver(
    message_tx: cbc::Sender<udp_message::UdpMessage>,
    master_activate_tx: cbc::Sender<bool>,
    socket: sync::Arc<net::UdpSocket>,
    elevator_id: u8
) {
    let mut master_id = config::ELEV_NUM_ELEVATORS-1;
    let mut master_timer = cbc::after(config::MASTER_TIMER_DURATION);

    loop {
        cbc::select! {
            recv(master_timer) -> _ => {
                master_id = (master_id + 1) % config::ELEV_NUM_ELEVATORS;
                if elevator_id == master_id {
                    master_activate_tx.send(true).unwrap();
                }
                master_timer = cbc::after(config::MASTER_TIMER_DURATION);
            },
            default(config::UDP_POLL_PERIOD) => {
                if let Some((received_message, _)) = udp::receive_udp_message::<String>(&socket) {
                    match serde_json::from_str::<udp_message::UdpMessage>(&received_message) {
                        Ok(udp_message::UdpMessage::State((elevator_id, state))) => {
                            message_tx.send(udp_message::UdpMessage::State((elevator_id, state))).unwrap();
                        }
                        Ok(udp_message::UdpMessage::Order(call)) => {
                            message_tx.send(udp_message::UdpMessage::Order(call)).unwrap();
                        }
                        Ok(udp_message::UdpMessage::AllAssignedOrders((incoming_master_id, all_assigned_orders))) => {
                            master_id = incoming_master_id;
                            master_timer = cbc::after(config::MASTER_TIMER_DURATION);
                            message_tx.send(udp_message::UdpMessage::AllAssignedOrders((master_id, all_assigned_orders))).unwrap();
   
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }
            }
        }
    }
}
