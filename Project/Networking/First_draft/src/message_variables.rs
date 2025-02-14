use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    pub id: String,
    pub obstructed: bool,
	pub motorstop: bool,
	pub behaviour: Behaviour,
	pub floor: u8,
    pub direction: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Behaviour {
    Idle,
    Moving,
    DoorsOpen,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderMessage {
    pub id: String,       
    pub state: State,  
    pub master_id: String,
}