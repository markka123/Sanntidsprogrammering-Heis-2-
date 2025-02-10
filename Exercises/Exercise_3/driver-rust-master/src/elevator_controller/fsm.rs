#![allow(dead_code)]
use std::thread::*;
use crate::elevio::elev as e;
use crate::elevio::elev::{HALL_UP, HALL_DOWN, CAB, DIRN_DOWN, DIRN_STOP, DIRN_UP};
use crate::elevio::poll::{CallButton};
use crate::elevator_controller::doors;
use crate::elevator_controller::direction;
use crate::elevator_controller::orders;

use crossbeam_channel as cbc;

#[derive(Clone, Debug)]
pub enum Behaviour {
    Idle,
    Moving,
    DoorsOpen,
}

#[derive(Clone, Debug)]
pub struct State {
    pub obstructed: bool,
	pub motorstop: bool,
	pub behaviour: Behaviour,
	pub floor: u8,
    pub direction: u8,
}

pub fn fsm_elevator(
        elevator: e::Elevator,
        floor_sensor_rx: cbc::Receiver<u8>,
        stop_button_rx: cbc::Receiver<bool>, 
        obstruction_rx: cbc::Receiver<bool>,
        new_order_rx: cbc::Receiver<orders::Orders>,

) {

    
    let mut state = State {
        obstructed: false,
        motorstop: false,
        behaviour: Behaviour::Idle,
        floor: 1,
        direction: e::DIRN_UP,
    };
    let (door_open_tx, door_open_rx) = cbc::unbounded::<bool>();
    let (door_close_tx, door_close_rx) = cbc::unbounded::<bool>();
    let (obstructed_tx, obstructed_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || doors::door(elevator, door_open_rx, door_close_rx, obstruction_rx, obstructed_tx));
    }
    
    loop {
        cbc::select! {
            recv(new_order_rx) -> a => {
                let new_order = a.unwrap();
                // println!("{:#?}", new_order);
                match state.behaviour {
                    Behaviour::Idle => {

                        println!("Values: New_order = {:#?}, state.floor = {}, state.direction = {}", &new_order, state.floor, state.direction);
                        match () {
                            // _ if new_order[state.floor as usize][state.direction as usize] || new_order[state.floor as usize][CAB as usize] => {
                            //     if let Err(e) = door_open_tx.send(true) {
                            //         eprintln!("Failed to send door open signal: {}", e);
                            //     }
                            //     // orderCompleted(); //channel
                            // //     state.behaviour = Behaviour::DoorsOpen;
                                
                            // //     // newState() // channel
                            // },
                            // _ if new_order[state.floor as usize][direction::direction_opposite(state.direction) as usize] => {
                                
                            // },

                            _ if orders::order_in_direction(&new_order, state.floor, state.direction) => {
                                elevator.motor_direction(state.direction);
                                state.behaviour = Behaviour::Moving;
                                // newState = true
                                // handle motorstop
                            },
                            _ if orders::order_in_direction(&new_order, state.floor, direction::direction_opposite(state.direction)) => {
                                state.direction = direction::direction_opposite(state.direction);
                                elevator.motor_direction(state.direction);
                                state.behaviour = Behaviour::Moving
                                // newState = true
                                // handle motorstop
                            }
                            () => todo!()
                            
                        }
                    }
                    Behaviour::DoorsOpen => {
                        
                    }
                    Behaviour::Moving => {
            
                    }
                }
            },
            recv(obstructed_rx) -> a => {
                if a.unwrap() != state.obstructed {
                    state.obstructed = a.unwrap();
                    //mangler en newstatec, for å skal sende til nettverket
                }
            }
        }
    }
}
