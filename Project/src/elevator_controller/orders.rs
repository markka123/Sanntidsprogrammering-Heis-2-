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
        } else {
            //Handle error
        }
    }

    pub fn reset_offline_orders(&mut self) {
        self.offline_orders= [[false; 3]; config::ELEV_NUM_FLOORS as usize];
    }
}

#[derive(Clone, Debug, Copy)]
pub struct ElevatorOrders {
    pub hall_orders: HallOrders,
    pub orders: Orders,
}

impl ElevatorOrders {
    pub fn init() -> Self {
        let hall_orders = [[false; 2]; config::ELEV_NUM_FLOORS as usize];
        let orders = [[false; 3]; config::ELEV_NUM_FLOORS as usize];
        Self {
            hall_orders,
            orders,
        }
    }

    pub fn order_at_floor_in_direction(&mut self, floor: u8, direction: u8) ->  bool {
        self.orders[floor as usize][(direction) as usize] || self.orders[floor as usize][CAB as usize]
    }

    pub fn order_in_direction(&mut self, floor: u8, dir: u8) -> bool {
        match dir {
            HALL_UP => {
                for f in (floor + 1)..config::ELEV_NUM_FLOORS {
                    for b in 0..config::ELEV_NUM_BUTTONS {
                        if self.orders[f as usize][b as usize] {
                            return true;
                        }
                    }
                }
                false
            }
            HALL_DOWN => {
                for f in (0..floor).rev() {
                    for b in 0..config::ELEV_NUM_BUTTONS {
                        if self.orders[f as usize][b as usize] {
                            return true;
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn order_done(
        &mut self, 
        floor: u8,
        direction: u8,
        order_completed_tx: &cbc::Sender<poll::CallButton>,
    ) {
        if self.orders[floor as usize][direction as usize] {
            let _ = order_completed_tx.send(poll::CallButton {
                floor,
                call: direction,
            });
        }
        if self.orders[floor as usize][CAB as usize] {
            let _ = order_completed_tx.send(poll::CallButton { floor, call: CAB });
        }
    }
    
}
