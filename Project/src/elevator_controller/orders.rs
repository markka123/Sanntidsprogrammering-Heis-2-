#![allow(dead_code)]
use crate::config::config;
use crate::elevio::elev::{CAB, HALL_DOWN, HALL_UP};
use crate::elevio::poll;

use crossbeam_channel as cbc;
use std::collections::HashMap;

pub type Orders = [[bool; 3]; config::ELEV_NUM_FLOORS as usize];
pub type HallOrders = [[bool; 2]; config::ELEV_NUM_FLOORS as usize];
pub type CabOrders = [[bool; config::ELEV_NUM_FLOORS as usize]; config::ELEV_NUM_ELEVATORS as usize];

#[derive(Clone, Debug, Copy)]
pub struct AllOrders {
    // Init with: let matrix = Matrix::new(rows, cols, false);
    pub hall_orders: HallOrders,
    pub cab_orders: CabOrders,
    pub offline_orders: Orders,
}

#[derive(Clone, Debug, Copy)]
pub struct ElevatorOrders {
    pub hall_orders: HallOrders,
    pub orders: Orders,
}
//functions
// init, order_at_floor_in_direction, order_in_direction, order_done

pub struct DistributorOrders {
    pub hall_orders: HallOrders,
    pub cab_orders: CabOrders,
    pub unconfirmed_orders: Vec<(u8, poll::CallButton)>,
    pub all_assigned_orders: HashMap<u8, Orders>,
    pub elevator_orders: Orders,
}
// functions
// init, add, remove, confirm_orders, get_assigned_hall_orders, init_offline_operation (recv is_online)

impl AllOrders {
    pub fn init() -> Self {
        let hall_orders = [[false; 2]; config::ELEV_NUM_FLOORS as usize];
        let cab_orders = [[false; config::ELEV_NUM_FLOORS as usize]; config::ELEV_NUM_ELEVATORS as usize];
        let offline_orders = [[false; 3]; config::ELEV_NUM_FLOORS as usize];
        Self {
            hall_orders,
            cab_orders,
            offline_orders,
        }
    }
    pub fn add_order(&mut self, call_button: poll::CallButton, elevator_id: u8) {
        if call_button.call == CAB {
            self.cab_orders[elevator_id as usize][call_button.floor as usize] = true;
        } else if call_button.call == HALL_DOWN || call_button.call == HALL_UP {
            self.hall_orders[call_button.floor as usize][call_button.call as usize] = true;
        } else {
            //Handle error
        }
    }

    pub fn remove_order(&mut self, call_button: poll::CallButton, elevator_id: u8) {
        if call_button.call == CAB {
            self.cab_orders[elevator_id as usize][call_button.floor as usize] = false;
        } else if call_button.call == HALL_DOWN || call_button.call == HALL_UP {
            self.hall_orders[call_button.floor as usize][call_button.call as usize] = false;
        } else {
            //Handle error
        }
    }

    pub fn reset_offline_orders(&mut self) {
        self.offline_orders= [[false; 3]; config::ELEV_NUM_FLOORS as usize];
    }
}

pub fn order_in_direction(orders: &Orders, floor: u8, dir: u8) -> bool {
    match dir {
        HALL_UP => {
            for f in (floor + 1)..config::ELEV_NUM_FLOORS {
                for b in 0..config::ELEV_NUM_BUTTONS {
                    if orders[f as usize][b as usize] {
                        return true;
                    }
                }
            }
            false
        }
        HALL_DOWN => {
            for f in (0..floor).rev() {
                for b in 0..config::ELEV_NUM_BUTTONS {
                    if orders[f as usize][b as usize] {
                        return true;
                    }
                }
            }
            false
        }
        _ => false,
    }
}

pub fn order_at_floor_in_direction(orders: &Orders, floor: u8, direction: u8) ->  bool {
    orders[floor as usize][(direction) as usize] || orders[floor as usize][CAB as usize]
}

pub fn order_done(
    floor: u8,
    direction: u8,
    orders: Orders,
    order_completed_tx: &cbc::Sender<poll::CallButton>,
) {
    if orders[floor as usize][direction as usize] {
        let _ = order_completed_tx.send(poll::CallButton {
            floor,
            call: direction,
        });
    }
    if orders[floor as usize][CAB as usize] {
        let _ = order_completed_tx.send(poll::CallButton { floor, call: CAB });
    }
}
