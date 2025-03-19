use crate::config::config;
use crate::elevator_controller::direction;
use crate::elevator_controller::doors;
use crate::elevator_controller::orders;
use crate::elevator_controller::lights;
use crate::elevator_controller::state;
use crate::elevio::elev;
use crate::elevio::poll;

use std::thread::*;
use crossbeam_channel as cbc;

pub fn elevator_fsm(
    elevator: &elev::Elevator,
    new_order_rx: cbc::Receiver<(orders::Orders, orders::HallOrders)>,
    order_completed_tx: cbc::Sender<poll::CallButton>,
    new_state_tx: cbc::Sender<state::State>,
) {
    let mut state = state::State::init();

    let mut elevator_orders = orders::ElevatorOrders::init();

    let mut motor_timer = cbc::never();

    let (door_open_tx, door_open_rx) = cbc::unbounded::<bool>();
    let (door_close_tx, door_close_rx) = cbc::unbounded::<bool>();
    let (obstructed_tx, obstructed_rx) = cbc::unbounded::<bool>();
    let (motorstop_tx, motorstop_rx) = cbc::unbounded::<bool>();
    
    let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::floor_sensor(elevator, floor_sensor_tx, config::POLL_PERIOD));
    }

    let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::stop_button(elevator, stop_button_tx, config::POLL_PERIOD));
    }

    let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::obstruction(elevator, obstruction_tx, config::POLL_PERIOD));
    }

    {
        let elevator = elevator.clone();
        spawn(move || {
            doors::door(
                elevator,
                door_open_rx,
                door_close_tx,
                obstruction_rx,
                obstructed_tx,
            )
        });
    }

    state.direction = elev::HALL_DOWN;
    state.behaviour = state::Behaviour::Moving;
    elevator.motor_direction(direction::call_to_md(state.direction));

    lights::set_lights(&elevator_orders, elevator.clone());
    elevator.stop_button_light(state.emergency_stop);

    loop {
        cbc::select! {
            recv(new_order_rx) -> new_order_tuple => {
                println!("Received new order");
                (elevator_orders.orders, elevator_orders.hall_orders) = new_order_tuple.unwrap();
            
                lights::set_lights(&elevator_orders, elevator.clone());

                if state.emergency_stop {
                    continue;
                }

                match state.behaviour {
                    state::Behaviour::Idle => {
                        match () {
                            _ if elevator_orders.order_at_floor_in_direction(state.floor, state.direction) => {
                                state.behaviour = state::Behaviour::DoorOpen;
                                elevator_orders.order_done(state.floor, state.direction, &order_completed_tx);
                                door_open_tx.send(true).unwrap();
                                new_state_tx.send(state.clone()).unwrap();
                            },
                            _ if elevator_orders.order_at_floor_in_direction(state.floor, direction::direction_opposite(state.direction)) => {
                                state.behaviour = state::Behaviour::DoorOpen;
                                state.direction = direction::direction_opposite(state.direction);
                                elevator_orders.order_done(state.floor, state.direction, &order_completed_tx);
                                door_open_tx.send(true).unwrap();
                                new_state_tx.send(state.clone()).unwrap();
                            },
                            _ if elevator_orders.order_in_direction(state.floor, state.direction) => {
                                state.behaviour = state::Behaviour::Moving;
                                elevator.motor_direction(direction::call_to_md(state.direction));
                                new_state_tx.send(state.clone()).unwrap();
                                motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
                            },
                            _ if elevator_orders.order_in_direction(state.floor, direction::direction_opposite(state.direction)) => {
                                state.behaviour = state::Behaviour::Moving;
                                state.direction = direction::direction_opposite(state.direction);
                                elevator.motor_direction(direction::call_to_md(state.direction));
                                new_state_tx.send(state.clone()).unwrap();
                                motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
                            }
                            _ if elevator_orders.orders[state.floor as usize].iter().all(|&x| x == false) => {
                                continue;
                                
                            }
                            () => {
                                println!("Handling new order in unexpected state.")
                            }
                        }
                    },
                    state::Behaviour::DoorOpen => {
                        if elevator_orders.order_at_floor_in_direction(state.floor, state.direction)  {
                            door_open_tx.send(true).unwrap();
                            elevator_orders.order_done(state.floor, state.direction, &order_completed_tx);
                        }
                    },
                    state::Behaviour::Moving => {
                    }
                }
            },

            recv(floor_sensor_rx) -> floor_message => {
                let floor = floor_message.unwrap();
                motor_timer = cbc::never();
                if state.motorstop {
                    motorstop_tx.send(false).unwrap();
                }
                state.floor = floor;
                elevator.floor_indicator(state.floor);

                if state.emergency_stop {
                    continue;
                }

                match state.behaviour {
                    state::Behaviour::Moving => {
                        match () {
                            _ if elevator_orders.order_at_floor_in_direction(state.floor, state.direction)=> {
                                state.behaviour = state::Behaviour::DoorOpen;
                                elevator.motor_direction(elev::DIRN_STOP);
                                door_open_tx.send(true).unwrap();
                                elevator_orders.order_done(floor, state.direction, &order_completed_tx);
                                new_state_tx.send(state.clone()).unwrap();
                            },

                            _ if elevator_orders.order_in_direction(state.floor, state.direction) => {
                                motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
                            },

                            _ if elevator_orders.order_at_floor_in_direction(state.floor, direction::direction_opposite(state.direction)) => {
                                state.behaviour = state::Behaviour::DoorOpen;
                                state.direction = direction::direction_opposite(state.direction);
                                elevator.motor_direction(elev::DIRN_STOP);
                                door_open_tx.send(true).unwrap(); 
                                elevator_orders.order_done(floor, state.direction, &order_completed_tx);
                                new_state_tx.send(state.clone()).unwrap();
                            },

                            _ if elevator_orders.order_in_direction(state.floor, direction::direction_opposite(state.direction)) => {
                                state.direction = direction::direction_opposite(state.direction);
                                new_state_tx.send(state.clone()).unwrap();
                                elevator.motor_direction(direction::call_to_md(state.direction));
                                motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
                            },

                            _ => {
                                state.behaviour = state::Behaviour::Idle;
                                elevator.motor_direction(elev::DIRN_STOP);
                                new_state_tx.send(state.clone()).unwrap();
                            }
                        }
                    },
                    _ => {
                        println!("Floor indicator received while in unexpected state")
                    }
                }
            },

            recv(door_close_rx) -> _ => {
                match state.behaviour {
                    state::Behaviour::DoorOpen => {
                        match () {
                            _ if elevator_orders.order_in_direction(state.floor, state.direction) => {
                                state.behaviour = state::Behaviour::Moving;
                                elevator.motor_direction(direction::call_to_md(state.direction));
                                new_state_tx.send(state.clone()).unwrap();
                                motor_timer = cbc::never();
                            },
                            _ if elevator_orders.order_at_floor_in_direction(state.floor, direction::direction_opposite(state.direction)) => {
                                state.direction = direction::direction_opposite(state.direction);
                                door_open_tx.send(true).unwrap();
                                new_state_tx.send(state.clone()).unwrap();
                                elevator_orders.order_done(state.floor, state.direction, &order_completed_tx);
                            },
                            _ if elevator_orders.order_in_direction(state.floor, direction::direction_opposite(state.direction)) => {
                                state.behaviour = state::Behaviour::Moving;
                                state.direction = direction::direction_opposite(state.direction);
                                elevator.motor_direction(direction::call_to_md(state.direction));
                                new_state_tx.send(state.clone()).unwrap();
                                motor_timer = cbc::never();
                            },
                            _ => {
                                state.behaviour = state::Behaviour::Idle;
                                new_state_tx.send(state.clone()).unwrap();
                                motor_timer = cbc::never();
                            }
                        }

                    },
                    state::Behaviour::Idle => {
                        /* if state.emergency_stop {
                            door_open_tx.send(true).unwrap(); // Sikker på dette?
                        } */
                    }
                    _ => {
                        println!("Closing doors in unexpected state");
                    }
                }

            },

            recv(motor_timer) -> _ => {
                motorstop_tx.send(true).unwrap();
            },

            recv(motorstop_rx) -> motorstop_message => {
                state.motorstop = motorstop_message.unwrap();
                new_state_tx.send(state.clone()).unwrap();
                println!("{}", if state.motorstop { "Lost motor power!" } else { "Regained motor power!" } );
            },

            recv(obstructed_rx) -> obstructed_message => {
                state.obstructed = obstructed_message.unwrap();
                new_state_tx.send(state.clone()).unwrap();
                println!("{}", if state.obstructed { "Doors are obstructed." } else { "Doors are no longer obstructed." } );
            },

            // Må bestemme oss for stop-knapp funksjonalitet
            recv(stop_button_rx) -> stop_button_message => {
                let is_emergency_stop = stop_button_message.unwrap();

                if is_emergency_stop && !state.emergency_stop {
                    state.emergency_stop = true;
                    elevator.motor_direction(elev::DIRN_STOP);
                    motor_timer = cbc::never();
                    new_state_tx.send(state.clone()).unwrap();
                    println!("Emergency stop activated");
                }
                else if is_emergency_stop && state.emergency_stop {
                    state.emergency_stop = false;
                    if state.behaviour == state::Behaviour::Moving {
                        elevator.motor_direction(direction::call_to_md(state.direction));
                    }
                    new_state_tx.send(state.clone()).unwrap();
                    println!("Emergency stop deactivated");
                }
                elevator.stop_button_light(state.emergency_stop);
            }
        }
    }
}