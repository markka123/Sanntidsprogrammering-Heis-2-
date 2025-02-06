#![allow(dead_code)]

use crate::elevio::elev as e;
use crate::elevio::elev::{HALL_UP, HALL_DOWN, CAB, DIRN_DOWN, DIRN_STOP, DIRN_UP};
use crate::elevio::poll::{CallButton};

#[derive(Clone, Debug)]
pub struct State {
    pub obstructed: bool,
	pub motorstop: bool,
	pub behaviour: Behaviour,
	pub floor: u8,
	pub Direction,
}

pub enum Behaviour {
    Idle,
    Moving,
    DoorsOpen,
}

#[derive(Debug)]
pub struct ElevatorVariables {
    pub current_floor: u8,
    pub direction: u8,
    pub doors: Doors,
    pub state: State,
}

fsm_floor_detected()

fsm_new_order()

fsm_doors_closed()

fsm_obstruction()

fsm_emergency_stop()

fsm_motor_stop()

