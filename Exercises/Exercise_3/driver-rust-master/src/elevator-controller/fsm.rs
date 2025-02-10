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
    pub direction: u8,
}

pub enum Behaviour {
    Idle,
    Moving,
    DoorsOpen,
}

fn fsm_floor_entered() {}

fn fsm_new_order(state: State, orders: Vec<Vec<bool>>) {
    match state.behaviour {
        Behaviour::Idle => {
            match () {
                _ if orders[state.floor][state.direction] || orders[state.floor][CAB] {
                    openDoors();
                    orderCompleted(state.floor, CAB);
                    state.behaviour = Behaviour::DoorsOpen;
                    newState()
                }
            }
        }
        Behaviour::DoorsOpen => {

        }
        Behaviour::Moving => {

        }
    }
}

fsm_doors_closed()

fsm_obstruction()

fsm_emergency_stop()

fsm_motor_stop()

