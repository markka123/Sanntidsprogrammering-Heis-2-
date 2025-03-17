#![allow(dead_code)]
use crate::config::config;
use crate::elevator_controller::direction;
use crate::elevator_controller::doors;
use crate::elevator_controller::orders;
use crate::elevator_controller::state::{State, Behaviour};
use crate::elevio::elev::{CAB, DIRN_STOP, HALL_DOWN};
use crate::elevio::{self, elev as e};
use std::thread::*;
use serde::{Serialize, Deserialize};


use crossbeam_channel as cbc;


//new order,state idle

orders::order_done(state.floor, state.direction, orders, &order_completed_tx);
door_open_tx.send(true).unwrap();
new_state_tx.send(state.clone()).unwrap(); //2x

//floor sensor, state moving

elevator.motor_direction(DIRN_STOP);
orders::order_done(floor, state.direction, orders, &order_completed_tx);
door_open_tx.send(true).unwrap(); 
new_state_tx.send(state.clone()).unwrap(); //4x


//door close, Dooropen and new order,state idle

elevator.motor_direction(direction::call_to_md(state.direction));
new_state_tx.send(state.clone()).unwrap();
motor_timer = cbc::after(config::MOTOR_TIMER_DURATION); //4x

pub fn 