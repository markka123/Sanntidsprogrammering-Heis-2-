use crate::config::config;

use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Behaviour {
    Idle,
    Moving,
    DoorOpen,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    pub obstructed: bool,
    pub motorstop: bool,
    pub offline: bool,
    pub emergency_stop: bool,
    pub behaviour: Behaviour,
    pub floor: u8,
    pub direction: u8,
}
// functions
// init, is_availible

pub type States = [State; config::ELEV_NUM_ELEVATORS as usize];

pub fn behaviour_to_string(behaviour: Behaviour) -> String {
    match behaviour {
        Behaviour::Idle => "idle".to_string(),
        Behaviour::Moving => "moving".to_string(),
        Behaviour::DoorOpen => "doorOpen".to_string(),
    }
}