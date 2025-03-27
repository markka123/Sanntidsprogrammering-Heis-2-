use crate::distributor::udp_message;
use crate::network::udp;

use crossbeam_channel as cbc;
use serde_json;
use std::net;
use std::sync;

/// Receive broadcasted UDP messages and pass them to the distributor.
pub fn receiver(
    udp_message_tx: cbc::Sender<udp_message::UdpMessage>,
    socket: sync::Arc<net::UdpSocket>,
) {
    loop {
        if let Some((received_message, _)) = udp::receive_message::<String>(&socket) {
            match serde_json::from_str::<udp_message::UdpMessage>(&received_message) {
                Ok(udp_message::UdpMessage::State((elevator_id, state))) => {
                    udp_message_tx.send(udp_message::UdpMessage::State((elevator_id, state))).unwrap();
                }
                Ok(udp_message::UdpMessage::Order((elevator_id, order))) => {
                    udp_message_tx.send(udp_message::UdpMessage::Order((elevator_id, order))).unwrap();
                }
                Ok(udp_message::UdpMessage::AllAssignedOrders((incoming_master_id, all_assigned_orders))) => {
                    udp_message_tx.send(udp_message::UdpMessage::AllAssignedOrders((incoming_master_id, all_assigned_orders))).unwrap();
                }
                Err(e) => {
                    println!("Failed to receive message in distributor::receiver and received this error: {:#?}", e);
                }
            }
        }
    }
}