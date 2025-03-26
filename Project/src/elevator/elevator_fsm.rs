use crate::config::config;
use crate::elevator::doors;
use crate::elevator::orders;
use crate::elevator::state;
use crate::elevio::elev;
use crate::elevio::poll;

use std::thread;
use crossbeam_channel as cbc;

pub fn elevator_fsm(
    elevator: &elev::Elevator,
    elevator_orders_rx: cbc::Receiver<(orders::Orders, orders::HallOrders)>,
    completed_order_tx: cbc::Sender<poll::CallButton>,
    new_order_tx: cbc::Sender<poll::CallButton>,
    new_state_tx: cbc::Sender<state::State>,
) {

    let (call_button_tx, call_button_rx) = cbc::unbounded::<poll::CallButton>();
    {
        let elevator = elevator.clone();
        thread::spawn(move || poll::call_buttons(elevator, call_button_tx, config::POLL_PERIOD));
    }
    
    let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
    {
        let elevator = elevator.clone();
        thread::spawn(move || poll::floor_sensor(elevator, floor_sensor_tx, config::POLL_PERIOD));
    }

    let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        thread::spawn(move || poll::stop_button(elevator, stop_button_tx, config::POLL_PERIOD));
    }

    let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        thread::spawn(move || poll::obstruction(elevator, obstruction_tx, config::POLL_PERIOD));
    }

    let (open_doors_tx, open_doors_rx) = cbc::unbounded::<bool>();
    let (close_doors_tx, close_doors_rx) = cbc::unbounded::<bool>();
    let (obstruct_doors_tx, obstruct_doors_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        thread::spawn(move || {
            doors::doors(
                elevator,
                open_doors_rx,
                close_doors_tx,
                obstruct_doors_rx,
            )
        });
    }

    let mut state = state::State::init();
    let mut elevator_orders = orders::ElevatorOrders::init();
    let mut motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);

    state.behaviour = state::Behaviour::Moving;
    elevator.motor_direction(state.direction.to_motor_direction());
    
    elevator_orders.set_lights(elevator.clone());
    elevator.stop_button_light(state.emergency_stop);

    loop {   
        cbc::select! {
            recv(elevator_orders_rx) -> elevator_orders_message => {
                (elevator_orders.orders, elevator_orders.hall_orders) = elevator_orders_message.unwrap();
            
                elevator_orders.set_lights(elevator.clone());
                
                match state.behaviour {
                    state::Behaviour::Idle => {
                        if elevator_orders.cab_at_floor(state.floor) || elevator_orders.hall_at_floor_in_direction(state.floor, state.direction) {
                            state.behaviour = state::Behaviour::DoorOpen;
                            new_state_tx.send(state.clone()).unwrap();
                            elevator_orders.order_done(state.floor, state.direction, &completed_order_tx);
                            open_doors_tx.send(true).unwrap();
                        }
                        else if elevator_orders.hall_at_floor_in_direction(state.floor, state.direction.opposite()) {
                            state.behaviour = state::Behaviour::DoorOpen;
                            state.direction = state.direction.opposite();
                            new_state_tx.send(state.clone()).unwrap();
                            elevator_orders.order_done(state.floor, state.direction, &completed_order_tx);
                            open_doors_tx.send(true).unwrap();
                        }
                        else if elevator_orders.order_in_direction(state.floor, state.direction) {
                            state.behaviour = state::Behaviour::Moving;
                            new_state_tx.send(state.clone()).unwrap();
                            elevator.motor_direction(state.direction.to_motor_direction());
                            motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
                        }
                        else if elevator_orders.order_in_direction(state.floor, state.direction.opposite()) {
                            state.behaviour = state::Behaviour::Moving;
                            state.direction = state.direction.opposite();
                            new_state_tx.send(state.clone()).unwrap();
                            elevator.motor_direction(state.direction.to_motor_direction());
                            motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
                        }
                    },
                    state::Behaviour::DoorOpen => {
                        if (elevator_orders.cab_at_floor(state.floor) || elevator_orders.hall_at_floor_in_direction(state.floor, state.direction))
                        && state.is_availible() {
                            elevator_orders.order_done(state.floor, state.direction, &completed_order_tx);
                            open_doors_tx.send(true).unwrap();
                        }
                    },
                    state::Behaviour::Moving => {
                        continue;
                    }
                }
            },
            recv(floor_sensor_rx) -> floor_message => {
                let floor = floor_message.unwrap();
                state.floor = floor;
                elevator.floor_indicator(state.floor);

                motor_timer = cbc::never();
                if state.motorstop {
                    state.motorstop = false;
                    println!("Regained motor power.");
                }

                if state.emergency_stop {
                    continue;
                }

                match state.behaviour {
                    state::Behaviour::Moving => {
                        if elevator_orders.hall_at_floor_in_direction(state.floor, state.direction) {
                            state.behaviour = state::Behaviour::DoorOpen;
                            elevator.motor_direction(elev::DIRN_STOP);
                            elevator_orders.order_done(floor, state.direction, &completed_order_tx);
                            open_doors_tx.send(true).unwrap();
                        }
                        else if elevator_orders.cab_at_floor(state.floor) && elevator_orders.order_in_direction(state.floor, state.direction) {
                            state.behaviour = state::Behaviour::DoorOpen;
                            elevator.motor_direction(elev::DIRN_STOP);
                            open_doors_tx.send(true).unwrap();
                            elevator_orders.order_done(floor, state.direction, &completed_order_tx);
                        }
                        else if elevator_orders.cab_at_floor(state.floor) && !elevator_orders.hall_at_floor_in_direction(state.floor, state.direction.opposite()) {
                            state.behaviour = state::Behaviour::DoorOpen;
                            elevator.motor_direction(elev::DIRN_STOP);
                            elevator_orders.order_done(floor, state.direction, &completed_order_tx);
                            open_doors_tx.send(true).unwrap();
                            
                        }
                        else if elevator_orders.order_in_direction(state.floor, state.direction) {
                            elevator.motor_direction(state.direction.to_motor_direction());
                            motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
                        }
                        else if elevator_orders.cab_at_floor(state.floor) || elevator_orders.hall_at_floor_in_direction(state.floor, state.direction.opposite()) {
                            state.behaviour = state::Behaviour::DoorOpen;
                            state.direction = state.direction.opposite();
                            elevator_orders.order_done(floor, state.direction, &completed_order_tx);
                            
                            elevator.motor_direction(elev::DIRN_STOP);
                            open_doors_tx.send(true).unwrap(); 
                            
                        }
                        else if elevator_orders.order_in_direction(state.floor, state.direction.opposite()) {
                            state.direction = state.direction.opposite();
                            elevator.motor_direction(state.direction.to_motor_direction());
                            motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
                        }
                        else {
                            state.behaviour = state::Behaviour::Idle;
                            elevator.motor_direction(elev::DIRN_STOP);
                        }
                    },
                    _ => {
                        println!("Floor indicator received while in unexpected state");
                    }
                }
                new_state_tx.send(state.clone()).unwrap();
            },
            recv(close_doors_rx) -> _ => {
                match state.behaviour {
                    state::Behaviour::DoorOpen => {
                        if elevator_orders.order_in_direction(state.floor, state.direction) {
                            state.behaviour = state::Behaviour::Moving;
                            elevator.motor_direction(state.direction.to_motor_direction());
                            motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
                        }
                        else if elevator_orders.cab_at_floor(state.floor) || elevator_orders.hall_at_floor_in_direction(state.floor, state.direction.opposite()) {
                            state.direction = state.direction.opposite();
                            open_doors_tx.send(true).unwrap();
                            elevator_orders.order_done(state.floor, state.direction, &completed_order_tx);
                        }
                        else if elevator_orders.order_in_direction(state.floor, state.direction.opposite()) {
                            state.behaviour = state::Behaviour::Moving;
                            state.direction = state.direction.opposite();
                            elevator.motor_direction(state.direction.to_motor_direction());
                            motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
                        }
                        else {
                            state.behaviour = state::Behaviour::Idle;
                            motor_timer = cbc::never();
                        }
                        new_state_tx.send(state.clone()).unwrap();
                    },
                    _ => {
                        println!("Closing doors in unexpected state.");
                    }
                }

            },
            recv(call_button_rx) -> call_button_message => {
                let call_button = call_button_message.unwrap(); 
                new_order_tx.send(call_button).unwrap();
            },
            recv(obstruction_rx) -> obstructed_message => {
                state.obstructed = obstructed_message.unwrap();
                new_state_tx.send(state.clone()).unwrap();
                obstruct_doors_tx.send(state.obstructed).unwrap();
            },
            recv(stop_button_rx) -> stop_button_message => {
                let stop_button_pressed = stop_button_message.unwrap();
                if stop_button_pressed && !state.emergency_stop {
                    state.emergency_stop = true;
                    elevator.motor_direction(elev::DIRN_STOP);
                    motor_timer = cbc::never();

                    if state.behaviour == state::Behaviour::Idle || state.behaviour == state::Behaviour::DoorOpen  {
                        obstruct_doors_tx.send(true).unwrap();
                        open_doors_tx.send(true).unwrap();
                    }

                    new_state_tx.send(state.clone()).unwrap();
                    println!("Emergency stop activated");
                }
                else if stop_button_pressed && state.emergency_stop {
                    state.emergency_stop = false;
                    if state.behaviour == state::Behaviour::Moving {
                        elevator.motor_direction(state.direction.to_motor_direction());
                        motor_timer = cbc::after(config::MOTOR_TIMER_DURATION);
                    }
                    obstruct_doors_tx.send(false).unwrap();
                    new_state_tx.send(state.clone()).unwrap();
                    println!("Emergency stop deactivated");
                }
                elevator.stop_button_light(state.emergency_stop);
            }
            recv(motor_timer) -> _ => {
                if !state.motorstop {
                    state.motorstop = true;
                    new_state_tx.send(state.clone()).unwrap();

                    println!("Lost motor power.")
                }
            },
        }
    }
}