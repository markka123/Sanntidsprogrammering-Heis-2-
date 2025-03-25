use crate::elevator::state;
use crate::network::udp;

use serde;
use serde_json;
use std::net;
use std::sync;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum UdpMessage {
    Order((u8, [u8; 3])),
    State((u8, state::State)),
    AllAssignedOrders((u8, serde_json::Value)),
}

pub fn broadcast_udp_message(socket: &sync::Arc<net::UdpSocket>, message: &UdpMessage) {
    let message_json = serde_json::to_string(message).unwrap();
    let _ = udp::broadcast_udp_message(&socket, &message_json);
}
