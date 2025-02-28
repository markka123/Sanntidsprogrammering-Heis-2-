#![allow(dead_code)]
use crate::config::config;
use crate::elevator_controller::direction;
use crate::elevator_controller::doors;
use crate::elevator_controller::orders;
use crate::elevio::elev::{CAB, DIRN_STOP, HALL_DOWN};
use crate::elevio::{self, elev as e};
use std::thread::*;

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
    pub emergency_stop: bool,
    pub behaviour: Behaviour,
    pub floor: u8,
    pub direction: u8,
}

pub fn elevator_fsm(
    elevator: &e::Elevator,
    new_order_rx: cbc::Receiver<orders::Orders>,
    delivered_order_tx: cbc::Sender<elevio::poll::CallButton>,
    // new_state_tx: &cbc::Sender<State>,
    emergency_reset_tx: cbc::Sender<bool>,
) {
    let mut state = State {
        obstructed: false,
        motorstop: false,
        emergency_stop: false,
        behaviour: Behaviour::Idle,
        floor: 0,
        direction: HALL_DOWN,
    };

    let mut orders: orders::Orders = [[false; 3]; config::ELEV_NUM_FLOORS as usize];

    let (door_open_tx, door_open_rx) = cbc::unbounded::<bool>();
    let (door_close_tx, door_close_rx) = cbc::unbounded::<bool>();
    let (obstructed_tx, obstructed_rx) = cbc::unbounded::<bool>();
    let (motorstop_tx, motorstop_rx) = cbc::unbounded::<bool>();
    let mut motor_timer = cbc::never();

    let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
    {
        let elevator = elevator.clone();
        spawn(move || elevio::poll::floor_sensor(elevator, floor_sensor_tx, config::POLL_PERIOD));
    }

    let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || elevio::poll::stop_button(elevator, stop_button_tx, config::POLL_PERIOD));
    }

    let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || elevio::poll::obstruction(elevator, obstruction_tx, config::POLL_PERIOD));
    }

    {
        let elevator = elevator.clone();
        spawn(move || {
            doors::door(
                elevator,
                &door_open_rx,
                &door_close_tx,
                &obstruction_rx,
                &obstructed_tx,
            )
        });
    }

    state.direction = HALL_DOWN;
    state.behaviour = Behaviour::Moving;
    elevator.motor_direction(direction::call_to_md(state.direction));

    loop {
        cbc::select! {
            recv(new_order_rx) -> a => {
                let new_orders = a.unwrap();
                orders = new_orders;
                // println!("New order received:\n {:#?}", orders);
                // println!("\nCurrent states are:\n {:#?}\n", state);

                if state.emergency_stop {
                    continue;
                }

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
                            _ if orders[state.floor as usize][direction::direction_opposite(state.direction) as usize] => {
                                door_open_tx.send(true).unwrap();

                                state.direction = direction::direction_opposite(state.direction);
                                orders[state.floor as usize][(state.direction) as usize]; // delete from orders


                                orders::order_done(state.floor, state.direction, orders, &delivered_order_tx); //channel
                                state.behaviour = Behaviour::DoorOpen;
                            },

                            _ if orders::order_in_direction(&orders, state.floor, state.direction) => {
                                elevator.motor_direction(direction::call_to_md(state.direction));
                                state.behaviour = Behaviour::Moving;
                                motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
                                // newState = true
                            },
                            _ if orders::order_in_direction(&orders, state.floor, direction::direction_opposite(state.direction)) => {
                                state.direction = direction::direction_opposite(state.direction);
                                elevator.motor_direction(direction::call_to_md(state.direction));
                                state.behaviour = Behaviour::Moving;
                                motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
                                // newState = true
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
                motor_timer = cbc::never();
                motorstop_tx.send(false).unwrap();
                state.floor = floor;
                // println!("\n\nEntered floor nr: {},\n with current states:\n {:#?}\n", state.floor, state);
                elevator.floor_indicator(state.floor);

                if state.emergency_stop {
                    continue;
                }

                match state.behaviour {
                    Behaviour::Moving => {
                        match () {
                            _ if (orders[state.floor as usize][state.direction as usize]) => {
                                elevator.motor_direction(DIRN_STOP);
                                door_open_tx.send(true).unwrap();
                                orders::order_done(floor, state.direction, orders, &delivered_order_tx);
                                state.behaviour = Behaviour::DoorOpen;
                            },

                            _ if (orders[state.floor as usize][CAB as usize] && orders::order_in_direction(&orders, state.floor, state.direction)) => {
                                elevator.motor_direction(DIRN_STOP);
                                door_open_tx.send(true).unwrap();
                                orders::order_done(floor, state.direction, orders, &delivered_order_tx);
                                state.behaviour = Behaviour::DoorOpen;
                            },

                            _ if (orders[state.floor as usize][CAB as usize] && !orders[state.floor as usize][direction::direction_opposite(state.direction) as usize]) => {
                                elevator.motor_direction(DIRN_STOP);
                                door_open_tx.send(true).unwrap();
                                orders::order_done(floor, state.direction, orders, &delivered_order_tx);
                                state.behaviour = Behaviour::DoorOpen;
                            }

                            _ if orders[state.floor as usize][direction::direction_opposite(state.direction) as usize] => {
                                elevator.motor_direction(DIRN_STOP);
                                door_open_tx.send(true).unwrap();
                                state.direction = direction::direction_opposite(state.direction);
                                orders::order_done(floor, state.direction, orders, &delivered_order_tx);
                                state.behaviour = Behaviour::DoorOpen;
                            },

                            _ if orders::order_in_direction(&orders, floor, state.direction) => {
                                motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
                            },

                            _ if orders::order_in_direction(&orders, floor, direction::direction_opposite(state.direction)) => {
                                state.direction = direction::direction_opposite(state.direction);
                                elevator.motor_direction(direction::call_to_md(state.direction));
                                motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
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
                                // new_state_tx.send(state.clone()).unwrap();
                                println!("Case 1");
                                motor_timer = cbc::never();
                                // new_state
                            },
                            _ if orders[state.floor as usize][direction::direction_opposite(state.direction) as usize] => {
                                door_open_tx.send(true).unwrap();
                                state.direction = direction::direction_opposite(state.direction);
                                orders::order_done(state.floor, state.direction, orders, &delivered_order_tx);
                            },
                            _ if orders::order_in_direction(&orders, state.floor, direction::direction_opposite(state.direction)) => {
                                door_open_tx.send(true).unwrap();
                                state.direction = direction::direction_opposite(state.direction);
                                elevator.motor_direction(direction::call_to_md(state.direction));
                                state.behaviour = Behaviour::Moving;
                                // motorTimer
                                // motor.c <- false
                                // new_state_tx.send(state.clone()).unwrap();
                                println!("Case 3");
                                motor_timer = cbc::never();
                                // new_state
                            },
                            _ => {
                                state.behaviour = Behaviour::Idle;
                                println!("Case 4");

                                // new_state_tx.send(state.clone()).unwrap();
                                motor_timer = cbc::never();
                                // new_state
                            }
                        }

                    },
                    Behaviour::Idle => {
                        if state.emergency_stop {
                            door_open_tx.send(true).unwrap(); // Sikker på dette?
                        }
                    }
                    _ => {
                        println!("Closing doors in unexpected state");
                    }
                }

            },

            recv(motor_timer) -> _ => {
                motorstop_tx.send(true).unwrap();
            },

            recv(motorstop_rx) -> a => {
                let is_motorstop = a.unwrap();
                if state.motorstop != is_motorstop {
                    state.motorstop = is_motorstop;
                    println!("{}", if state.motorstop { "Lost motor power!" } else { "Regained motor power!" } );
                }
            },

            recv(obstructed_rx) -> a => {
                let is_obstructed = a.unwrap();
                println!("\nObstructed: {:#?}\n\n", is_obstructed);
                if is_obstructed != state.obstructed {
                    state.obstructed = is_obstructed;
                }
            },

            // Må bestemme oss for stop-knapp funksjonalitet
            recv(stop_button_rx) -> a => {
                let is_emergency_stop = a.unwrap();

                if is_emergency_stop && !state.emergency_stop {
                    state.emergency_stop = true;
                    elevator.motor_direction(DIRN_STOP);
                    motor_timer = cbc::never();
                    state.behaviour = Behaviour::Idle;
                }
                else if is_emergency_stop && state.emergency_stop {
                    state.emergency_stop = false;
                    emergency_reset_tx.send(true).unwrap();
                }
                elevator.stop_button_light(state.emergency_stop);
            }
        }
    }
}
