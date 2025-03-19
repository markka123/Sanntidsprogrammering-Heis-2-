#![allow(dead_code)]
use crate::config::config;
use crate::elevio::elev;
use crate::elevio::poll;

use crossbeam_channel as cbc;
use std::collections::HashMap;

pub type Orders = [[bool; 3]; config::ELEV_NUM_FLOORS as usize];
pub type HallOrders = [[bool; 2]; config::ELEV_NUM_FLOORS as usize];
pub type CabOrders = [[bool; config::ELEV_NUM_FLOORS as usize]; config::ELEV_NUM_ELEVATORS as usize];
pub const NEW_ORDER: u8 = 0;
pub const COMPLETED_ORDER: u8 = 1;

#[derive(Clone, Debug, Copy)]
pub struct ElevatorOrders {
    pub hall_orders: HallOrders,
    pub orders: Orders,
}
//functions
// init, order_at_floor_in_direction, order_in_direction, order_done

#[derive(Clone, Debug)]
pub struct DistributorOrders {
    pub hall_orders: HallOrders,
    pub cab_orders: CabOrders,
    pub unconfirmed_orders: Vec<(u8, poll::CallButton)>,
    pub assigned_orders_map: HashMap<u8, Orders>,
    pub elevator_orders: Orders,
}
// functions
// init, add, remove, confirm_orders, get_assigned_hall_orders, init_offline_operation (recv is_online)
impl DistributorOrders {
    pub fn init() -> Self {
        let hall_orders = [[false; 2]; config::ELEV_NUM_FLOORS as usize];
        let cab_orders = [[false; config::ELEV_NUM_FLOORS as usize]; config::ELEV_NUM_ELEVATORS as usize];
        let unconfirmed_orders = Vec::new();
        let assigned_orders_map = HashMap::new();
        let elevator_orders = [[false; 3]; config::ELEV_NUM_FLOORS as usize];
        Self {
            hall_orders,
            cab_orders,
            unconfirmed_orders,
            assigned_orders_map,
            elevator_orders,
        }
    }
    pub fn add_order(&mut self, call_button: poll::CallButton, elevator_id: u8) {
        if call_button.call == elev::CAB {
            self.cab_orders[elevator_id as usize][call_button.floor as usize] = true;
        } else if call_button.call == elev::HALL_DOWN || call_button.call == elev::HALL_UP {
            self.hall_orders[call_button.floor as usize][call_button.call as usize] = true;
        }
    }

    pub fn remove_order(&mut self, call_button: poll::CallButton, elevator_id: u8) {
        if call_button.call == elev::CAB {
            self.cab_orders[elevator_id as usize][call_button.floor as usize] = false;
        } else if call_button.call == elev::HALL_DOWN || call_button.call == elev::HALL_UP {
            self.hall_orders[call_button.floor as usize][call_button.call as usize] = false;
        }
    }

    pub fn confirm_orders(&mut self, elevator_id: u8) {
        let assigned_orders_map = self.assigned_orders_map.clone();
        self.unconfirmed_orders.retain(|(order_type, order)| {
            let order_is_assigned = |floor: usize, call: usize| 
                assigned_orders_map.iter().any(|(_, assigned_orders)| assigned_orders[floor][call]);
    
            let order_is_unassigned = |floor: usize, call: usize| 
                assigned_orders_map.iter().all(|(_, assigned_orders)| !assigned_orders[floor][call]);
    
            match order.call {
                elev::HALL_UP | elev::HALL_DOWN => match order_type {
                    &NEW_ORDER => !order_is_assigned(order.floor as usize, order.call as usize),
                    &COMPLETED_ORDER => order_is_unassigned(order.floor as usize, order.call as usize),
                    _ => true,
                },
                elev::CAB => {
                    if let Some(assigned_orders) = assigned_orders_map.get(&elevator_id) {
                        let cab_is_assigned = assigned_orders[order.floor as usize][order.call as usize];
                        return !((cab_is_assigned && *order_type == NEW_ORDER) || (!cab_is_assigned && *order_type == COMPLETED_ORDER));
                    }
                    true
                }
                _ => true,
            }
        });
    }

    pub fn get_assigned_hall_orders(&mut self) -> HallOrders {
        let mut all_hall_orders = [[false; 2]; config::ELEV_NUM_FLOORS as usize];
    
        for orders in self.assigned_orders_map.values() {
            for (floor, call) in orders.iter().enumerate() {
                all_hall_orders[floor][0] |= call[0];
                all_hall_orders[floor][1] |= call[1];
            }
        }
        all_hall_orders
    }

    pub fn init_offline_operation(&mut self, id: u8) {
        for (order_type, order) in self.unconfirmed_orders.iter() {
            if *order_type == NEW_ORDER {
                self.elevator_orders[order.floor as usize][order.call as usize] = true;
            } 
        }
        let mut floor = 0;
        for order in self.cab_orders[id as usize].iter() {
            self.elevator_orders[floor as usize][elev::CAB as usize] = *order;
            floor += 1;  
        }
        self.hall_orders = [[false; 2]; config::ELEV_NUM_FLOORS as usize];
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
    orders[floor as usize][(direction) as usize] || orders[floor as usize][elev::CAB as usize]
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
    if orders[floor as usize][elev::CAB as usize] {
        let _ = order_completed_tx.send(poll::CallButton { floor, call: elev::CAB });
    }
}
