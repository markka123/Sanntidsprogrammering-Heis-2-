use crate::config::config;
use crate::elevio::elev;
use crate::elevio::poll;
use crate::elevator::orders;

use std::collections::HashMap;

pub const NEW_ORDER: u8 = 0;
pub const COMPLETED_ORDER: u8 = 1;

#[derive(Clone, Debug)]
pub struct AllOrders {
    pub hall_orders: orders::HallOrders,
    pub cab_orders: orders::CabOrders,
    pub unconfirmed_orders: Vec<(u8, poll::CallButton)>,
    pub assigned_orders_map: HashMap<u8, orders::Orders>,
    pub elevator_orders: orders::Orders,
}

impl AllOrders {
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
    pub fn add_order(&mut self, order: poll::CallButton, elevator_id: u8) {
        if order.call == elev::CAB {
            self.cab_orders[elevator_id as usize][order.floor as usize] = true;
        } else if order.call == elev::HALL_DOWN || order.call == elev::HALL_UP {
            self.hall_orders[order.floor as usize][order.call as usize] = true;
        }
    }

    pub fn remove_order(&mut self, order: poll::CallButton, elevator_id: u8) {
        if order.call == elev::CAB {
            self.cab_orders[elevator_id as usize][order.floor as usize] = false;
        } else if order.call == elev::HALL_DOWN || order.call == elev::HALL_UP {
            self.hall_orders[order.floor as usize][order.call as usize] = false;
        } else {
            //Handle error
        }
    }

    pub fn confirm_orders(&mut self, elevator_id: u8) {
        let assigned_orders_map = self.assigned_orders_map.clone();
        self.unconfirmed_orders.retain(|(order_type, order)| {
            let order_is_assigned = |floor: usize, call: usize| {
                assigned_orders_map
                    .iter()
                    .any(|(_, assigned_orders)| assigned_orders[floor][call])
            };

            let order_is_unassigned = |floor: usize, call: usize| {
                assigned_orders_map
                    .iter()
                    .all(|(_, assigned_orders)| !assigned_orders[floor][call])
            };

            match order.call {
                elev::HALL_UP | elev::HALL_DOWN => match order_type {
                    &NEW_ORDER => !order_is_assigned(order.floor as usize, order.call as usize),
                    &COMPLETED_ORDER => {
                        !order_is_unassigned(order.floor as usize, order.call as usize)
                    }
                    _ => true,
                },
                elev::CAB => {
                    if let Some(assigned_orders) = assigned_orders_map.get(&elevator_id) {
                        let cab_is_assigned = assigned_orders[order.floor as usize][order.call as usize];
                        return !((cab_is_assigned && *order_type == NEW_ORDER)
                            || (!cab_is_assigned && *order_type == COMPLETED_ORDER));
                    }
                    true
                }
                _ => true,
            }
        });
    }   

    pub fn confirm_offline_order(&mut self, order_completed: poll::CallButton) {
        self.unconfirmed_orders.retain(|(message, order)| *message != COMPLETED_ORDER || order.floor != order_completed.floor || order.call != order_completed.call);
    }

    pub fn get_assigned_hall_orders(&mut self) -> orders::HallOrders {
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
        for floor in 0..config::ELEV_NUM_FLOORS {
            for call in 0..(config::ELEV_NUM_BUTTONS-1) {
                self.hall_orders[floor as usize][call as usize] = self.elevator_orders[floor as usize][call as usize];
            }
        }

    }
}
