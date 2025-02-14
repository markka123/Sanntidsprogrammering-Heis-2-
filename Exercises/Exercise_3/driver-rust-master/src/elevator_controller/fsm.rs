#![allow(dead_code)]
use std::thread::*;
use std::time::*;
use crate::elevio::{self, elev as e};
use crate::elevio::elev::{HALL_UP, HALL_DOWN, CAB, DIRN_DOWN, DIRN_STOP, DIRN_UP};
use crate::elevio::poll::{CallButton};
use crate::elevator_controller::doors;
use crate::elevator_controller::direction;
use crate::elevator_controller::orders;
use crate::config::config;

use crossbeam_channel as cbc;

#[derive(Clone, Debug)]
pub enum Behaviour {
    Idle,
    Moving,
    DoorOpen,
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
        elevator: &e::Elevator,
        floor_sensor_rx: cbc::Receiver<u8>,
        stop_button_rx: cbc::Receiver<bool>, 
        obstruction_rx: cbc::Receiver<bool>,
        new_order_rx: cbc::Receiver<orders::Orders>,
        delivered_order_tx: cbc::Sender<elevio::poll::CallButton>
) {

    
    let mut state = State {
        obstructed: false,
        motorstop: false,
        behaviour: Behaviour::Idle,
        floor: 0,
        direction: e::HALL_UP,
    };
    let (door_open_tx, door_open_rx) = cbc::unbounded::<bool>();
    let (door_close_tx, door_close_rx) = cbc::unbounded::<bool>();
    let (obstructed_tx, obstructed_rx) = cbc::unbounded::<bool>();
    let (motor_tx, motor_rx) = cbc::unbounded::<()>();
    {
        let elevator = elevator.clone();
        spawn(move || doors::door(elevator, &door_open_rx, &door_close_tx, &obstruction_rx, &obstructed_tx));
    }

    let mut orders: orders::Orders = [[false; 3]; config::elev_num_floors as usize]; //hvordan skal denne bli oppdatert med alle ordre som heisen skal gjennomføre?
    
    loop {
        cbc::select! {
            recv(new_order_rx) -> a => {
                let new_orders = a.unwrap();
                orders = new_orders;
                // println!("{:#?}", new_order);
                match state.behaviour {
                    Behaviour::Idle => {

                        println!("Values: New_order = {:#?}, state.floor = {}, state.direction = {}", &orders, state.floor, state.direction);
                        match () {
                            _ if orders[state.floor as usize][(state.direction) as usize] || 
                                orders[state.floor as usize][CAB as usize] => {
                                door_open_tx.send(true).unwrap();
                                
                                orders[state.floor as usize][(state.direction) as usize] = false; // delete from orders
                                elevator.call_button_light(state.floor, state.direction, false);
                                elevator.call_button_light(state.floor, CAB, false);

                                orders::order_done(state.floor, state.direction, orders, &delivered_order_tx); //channel
                                state.behaviour = Behaviour::DoorOpen;
                                
                                // newState() // channelCAB as usize
                            },
                            _ if orders[state.floor as usize][direction::direction_opposite((state.direction)) as usize] => {
                                door_open_tx.send(true).unwrap();
                                
                                state.direction = direction::direction_opposite(state.direction);
                                orders[state.floor as usize][(state.direction) as usize]; // delete from orders

                                
                                orders::order_done(state.floor, state.direction, orders, &delivered_order_tx); //channel
                                state.behaviour = Behaviour::DoorOpen;  
                            },

                            _ if orders::order_in_direction(&orders, state.floor, state.direction) => {
                                elevator.motor_direction(direction::call_to_md(state.direction));
                                state.behaviour = Behaviour::Moving;
                                // newState = true
                                // handle motorstop
                            },
                            _ if orders::order_in_direction(&orders, state.floor, direction::direction_opposite(state.direction)) => {
                                state.direction = direction::direction_opposite(state.direction);
                                elevator.motor_direction(direction::call_to_md(state.direction));
                                state.behaviour = Behaviour::Moving;
                                // newState = true
                                // handle motorstop
                            }
                            () => {
                                println!("Handling new order in unexpected state.")
                            }
                            
                        }
                    },
                    Behaviour::DoorOpen => {
                        if orders[state.floor as usize][CAB as usize] || orders[state.floor as usize][state.direction as usize] {
                            door_open_tx.send(true).unwrap();

                            orders[state.floor as usize][CAB as usize] = false;
                            orders[state.floor as usize][state.direction as usize]= false;
                            orders::order_done(state.floor, state.direction, orders, &delivered_order_tx);
                        }
                    },
                    Behaviour::Moving => {
                        // no action
                    }
                }
            },

            recv(floor_sensor_rx) -> a => {
                let floor = a.unwrap();
                state.floor = floor;
                println!("Entered floor nr: {}", state.floor);
                elevator.floor_indicator(state.floor);
                //motorTime.Stop()
                motor_tx.send(()).unwrap();
                match state.behaviour {
                    Behaviour::Moving => {
                        match () {
                            _ if (orders[state.floor as usize][state.direction as usize]) ||
                                 (orders[state.floor as usize][CAB as usize] && orders::order_in_direction(&orders, state.floor, state.direction)) ||
                                 (orders[state.floor as usize][CAB as usize] && !orders[state.floor as usize][direction::direction_opposite(state.direction) as usize]) => {
                                elevator.motor_direction(DIRN_STOP);
                                door_open_tx.send(true).unwrap();
                                orders::order_done(floor, state.direction, orders, &delivered_order_tx);
                                state.behaviour = Behaviour::DoorOpen;
                            },
                            
                            _ if orders[state.floor as usize][direction::direction_opposite(state.direction) as usize] => {
                                elevator.motor_direction(DIRN_STOP);
                                door_open_tx.send(true).unwrap();
                                state.direction = direction::direction_opposite(state.direction);
                                orders::order_done(floor, state.direction, orders, &delivered_order_tx);
                                state.behaviour = Behaviour::DoorOpen;
                            },
                        
                            _ if orders::order_in_direction(&orders, floor, state.direction) => {
                                // motorTimer
                                // motor.c <- false
                            },
                        
                            _ if orders::order_in_direction(&orders, floor, direction::direction_opposite(state.direction)) => {
                                state.direction = direction::direction_opposite(state.direction);
                                elevator.motor_direction(direction::call_to_md(state.direction));
                                // motorTimer
                                // motor.c <- false
                            },
                        
                            _ => {
                                elevator.motor_direction(DIRN_STOP);
                                state.behaviour = Behaviour::Idle;
                            }
                        }
                        

                    },
                    _ => {
                        println!("Floor indicator received while in unexpected state")
                    }
                }
            },

            recv(motor_rx) -> _ => {
                if state.motorstop {
                    println!("Gained motor power");
                    state.motorstop = false;
                    //new_state
                }
            },
            recv(obstructed_rx) -> a => {
                let obsstructed = a.unwrap();
                if obsstructed != state.obstructed {
                    state.obstructed = obsstructed;
                    //new_state
                }
            },

            recv(door_close_rx) -> _ => {
                println!("Closing doors");
                match state.behaviour {
                    Behaviour::DoorOpen => {
                        match () {
                            _ if orders::order_in_direction(&orders, state.floor, state.direction) => {
                                elevator.motor_direction(direction::call_to_md(state.direction));
                                state.behaviour = Behaviour::Moving;
                                // motorTimer
                                // motor.c <- false
                                // new_state
                                println!("Case 1");
                            },
                            _ if orders[state.floor as usize][direction::direction_opposite(state.direction) as usize] => {
                                door_open_tx.send(true).unwrap(); // Sikker på dette?
                                state.direction = direction::direction_opposite(state.direction);
                                orders::order_done(state.floor, state.direction, orders, &delivered_order_tx);
                                println!("Case 2");
                            },
                            _ if orders::order_in_direction(&orders, state.floor, direction::direction_opposite(state.direction)) => {
                                door_open_tx.send(true).unwrap(); // Sikker på dette?
                                state.direction = direction::direction_opposite(state.direction);
                                elevator.motor_direction(direction::call_to_md(state.direction));
                                state.behaviour = Behaviour::Moving;
                                // motorTimer
                                // motor.c <- false
                                // new_state
                                println!("Case 3");
                            },
                            _ => {
                                state.behaviour = Behaviour::Idle;
                                println!("Case 4");

                                // new_state
                            }
                        }

                    },
                    _ => {
                        println!("Closing doors in unexpected state");
                    }
                }

            }

        }
    }
}
