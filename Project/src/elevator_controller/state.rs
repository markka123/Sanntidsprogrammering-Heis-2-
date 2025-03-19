use crate::config::config;
use crate::elevio::elev;

use serde;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub enum Behaviour {
    Idle,
    Moving,
    DoorOpen,
}

pub fn behaviour_to_string(behaviour: Behaviour) -> String {
    match behaviour {
        Behaviour::Idle => "idle".to_string(),
        Behaviour::Moving => "moving".to_string(),
        Behaviour::DoorOpen => "doorOpen".to_string(),
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct State {
    pub obstructed: bool,
    pub motorstop: bool,
    pub offline: bool,
    pub emergency_stop: bool,
    pub behaviour: Behaviour,
    pub floor: u8,
    pub direction: u8,
}
impl State {
    pub fn init() -> Self {
        let obstructed = false;
        let motorstop = false;
        let offline = false;
        let emergency_stop = false;
        let behaviour = Behaviour::Idle;
        let floor = 0;
        let direction = elev::HALL_DOWN;
        Self {
            obstructed,
            motorstop,
            offline,
            emergency_stop,
            behaviour,
            floor,
            direction,
        }
    }
    pub fn is_availible(self) -> bool {
        !(self.motorstop || self.emergency_stop || self.obstructed || self.offline)
    }
}

pub type States = [State; config::ELEV_NUM_ELEVATORS as usize];