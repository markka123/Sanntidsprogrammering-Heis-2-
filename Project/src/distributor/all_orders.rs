use crate::config::config;
use crate::elevio::elev;
use crate::elevio::poll;
use crate::elevator::orders;

use std::collections;
use serde_json;


pub const NEW_ORDER: u8 = 0;
pub const COMPLETED_ORDER: u8 = 1;


#[derive(Clone, Debug)]
pub struct AllOrders {
    pub hall_orders: orders::HallOrders,
    pub cab_orders: orders::CabOrders,
    pub unconfirmed_orders: Vec<(u8, poll::CallButton)>,
    pub assigned_orders_map: collections::HashMap<u8, orders::Orders>,
    pub elevator_orders: orders::Orders,
}

impl AllOrders {
    pub fn init() -> Self {
        let hall_orders = [[false; 2]; config::ELEV_NUM_FLOORS as usize];
        let cab_orders = [[false; config::ELEV_NUM_FLOORS as usize]; config::ELEV_NUM_ELEVATORS as usize];
        let unconfirmed_orders = Vec::new();
        let assigned_orders_map = collections::HashMap::new();
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
        }
    }

    pub fn add_offline_order(&mut self, order: poll::CallButton, elevator_id: u8 ) {
        self.add_order(order.clone(), elevator_id);
        self.elevator_orders[order.floor as usize][order.call as usize] = true;
        self.unconfirmed_orders.retain(|(order_status, unconfirmed_order)| !(*order_status == COMPLETED_ORDER && order == *unconfirmed_order));
    }

    pub fn remove_offline_order(&mut self, order: poll::CallButton, elevator_id: u8) {
        self.remove_order(order.clone(), elevator_id);
        self.elevator_orders[order.floor as usize][order.call as usize] = false;
        self.unconfirmed_orders.retain(|(order_status, unconfirmed_order)| !(*order_status == NEW_ORDER && order == *unconfirmed_order));
        self.unconfirmed_orders.retain(|(order_status, _)| !(*order_status == COMPLETED_ORDER && order.call != elev::CAB));
    }

    /// Remove orders from unconfirmed_orders if they appear in assigned_orders_map.
    pub fn confirm_orders(&mut self, elevator_id: u8) {
        let assigned_orders_map = self.assigned_orders_map.clone();
        self.unconfirmed_orders.retain(|(order_status, order)| {
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
                elev::HALL_UP | elev::HALL_DOWN => match order_status {
                    &NEW_ORDER => !order_is_assigned(order.floor as usize, order.call as usize),
                    &COMPLETED_ORDER => {
                        !order_is_unassigned(order.floor as usize, order.call as usize)
                    }
                    _ => true,
                },
                elev::CAB => {
                    if let Some(assigned_orders) = assigned_orders_map.get(&elevator_id) {
                        let cab_is_assigned = assigned_orders[order.floor as usize][order.call as usize];
                        let cab_order_confirmed = (cab_is_assigned && *order_status == NEW_ORDER) || (!cab_is_assigned && *order_status == COMPLETED_ORDER);
                        return !cab_order_confirmed
                    }
                    true
                }
                _ => true,
            }
        });
    }  


    pub fn get_assigned_hall_and_cab_orders(&mut self) -> (orders::HallOrders, orders::CabOrders) {
        let mut cab_orders = self.cab_orders;
        let mut hall_orders = [[false; 2]; config::ELEV_NUM_FLOORS as usize];

        for (elevator_id, orders) in &self.assigned_orders_map {
            for (floor, call) in orders.iter().enumerate() {
                hall_orders[floor][elev::HALL_UP as usize] |= call[elev::HALL_UP as usize];
                hall_orders[floor][elev::HALL_DOWN as usize] |= call[elev::HALL_DOWN as usize];
                cab_orders[*elevator_id as usize][floor] = call[elev::CAB as usize];
            }
        }
        (hall_orders, cab_orders)
    }

    pub fn update_orders(&mut self, all_assigned_orders: serde_json::Value, elevator_id: u8) -> bool {
        let (previous_hall_orders, _) = self.get_assigned_hall_and_cab_orders();
        let previous_elevator_orders = self.elevator_orders;
        
        self.assigned_orders_map = serde_json::from_value(all_assigned_orders).unwrap();
        (self.hall_orders, self.cab_orders) = self.get_assigned_hall_and_cab_orders();

        if let Some(new_elevator_orders) = self.assigned_orders_map.get(&elevator_id) {
            self.elevator_orders = *new_elevator_orders;
        } else {
            self.elevator_orders = [[false; 3]; config::ELEV_NUM_FLOORS as usize];
            for (floor, order) in self.cab_orders[elevator_id as usize].iter().enumerate() {
                self.elevator_orders[floor as usize][elev::CAB as usize] = *order;
            }
        } 
        
        (self.hall_orders != previous_hall_orders) || (self.elevator_orders != previous_elevator_orders)
    } 

    pub fn init_offline_operation(&mut self, elevator_id: u8) {


        for (floor, order) in self.cab_orders[elevator_id as usize].iter().enumerate() {
            self.elevator_orders[floor as usize][elev::CAB as usize] = *order;
        }
        for floor in 0..config::ELEV_NUM_FLOORS {
            for call in 0..(config::ELEV_NUM_BUTTONS-1) {
                self.elevator_orders[floor as usize][call as usize] = self.hall_orders[floor as usize][call as usize];
            }
        }
        for (order_status, order) in self.unconfirmed_orders.iter() {
            if *order_status == NEW_ORDER {
                self.elevator_orders[order.floor as usize][order.call as usize] = true;
            }
        }
    }
}
