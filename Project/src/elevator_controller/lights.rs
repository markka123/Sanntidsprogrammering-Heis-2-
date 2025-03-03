#![allow(dead_code)]
use crate::config::config;
use crate::elevio::elev as e;
use crate::elevio::elev::{CAB};
use crate::elevator_controller::orders::{AllOrders};


pub fn set_lights(all_orders: &AllOrders, elevator: e::Elevator, elevator_id: u8) {
    for f in 0..config::ELEV_NUM_FLOORS {
        for b in 0..(config::ELEV_NUM_BUTTONS-1) {
                elevator.call_button_light(f, b, all_orders.hall_orders[f as usize][b as usize]);
        }
        elevator.call_button_light(f, CAB, all_orders.cab_orders[elevator_id as usize][f as usize]);
    }
}