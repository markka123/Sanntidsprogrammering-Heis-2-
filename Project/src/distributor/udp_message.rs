use crate::elevator_controller::state;

use serde;
use serde_json;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum UdpMessage {
    Order((u8, [u8; 3])),
    State((u8, state::State)),
    AllAssignedOrders((u8, serde_json::Value)),
}