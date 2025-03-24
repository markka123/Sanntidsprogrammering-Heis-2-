use crate::config::config;
use crate::elevio::elev;

use serde;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Behaviour {
    Idle,
    Moving,
    DoorOpen,
}

impl Behaviour {
    pub fn to_string(&self) -> String {
        match self {
            Behaviour::Idle => "idle".to_string(),
            Behaviour::Moving => "moving".to_string(),
            Behaviour::DoorOpen => "doorOpen".to_string(),
        }
    }
    
} 

#[repr(u8)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    Up = elev::HALL_UP,
    Down = elev::HALL_DOWN,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    pub fn to_motor_direction(&self) -> u8 {
        match self {
            Direction::Up => elev::DIRN_UP,
            Direction::Down => elev::DIRN_DOWN,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Direction::Up => "up".to_string(),
            Direction::Down => "down".to_string(),
        }
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
    pub direction: Direction,
}
impl State {
    pub fn init() -> Self {
        let obstructed = false;
        let motorstop = false;
        let offline = false;
        let emergency_stop = false;
        let behaviour = Behaviour::Idle;
        let floor = 0;
        let direction = Direction::Down;
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
    pub fn is_availible(&self) -> bool {
        !(self.motorstop || self.emergency_stop || (self.obstructed && self.behaviour == Behaviour::DoorOpen) || self.offline)
    }
}

pub type States = [State; config::ELEV_NUM_ELEVATORS as usize];