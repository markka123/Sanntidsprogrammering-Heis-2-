#![allow(dead_code)]
use crate::config::config;
use crate::elevio::elev::{CAB, HALL_DOWN, HALL_UP};
use crate::elevio::poll::CallButton;
use crossbeam_channel as cbc;

pub type Orders = [[bool; 3]; config::ELEV_NUM_FLOORS as usize];

#[derive(Clone, Debug)]
pub struct AllOrders {
    // Init with: let matrix = Matrix::new(rows, cols, false);
    pub hall_orders: Vec<[bool; 2]>,
    pub cab_orders: Vec<Vec<bool>>,
    pub orders: Orders,
}

impl AllOrders {
    pub fn init() -> Self {
        let hall_orders = vec![[false; 2]; config::ELEV_NUM_FLOORS as usize];
        let cab_orders = vec![
            vec![false; config::ELEV_NUM_FLOORS as usize];
            config::ELEV_NUM_ELEVATORS as usize
        ];
        let orders = [[false; 3]; config::ELEV_NUM_FLOORS as usize];
        Self {
            hall_orders,
            cab_orders,
            orders,
        }
    }
    pub fn add_order(&mut self, call_button: CallButton, elevator_id: usize) {
        if call_button.call == CAB {
            self.cab_orders[elevator_id][call_button.floor as usize] = true;
        } else if call_button.call == HALL_DOWN || call_button.call == HALL_UP {
            self.hall_orders[call_button.floor as usize][call_button.call as usize] = true;
        } else {
            //Handle error
        }
        println!("Btn: {:#?}", call_button);
        self.orders[call_button.floor as usize][call_button.call as usize] = true;
    }

    pub fn remove_order(&mut self, call_button: CallButton, elevator_id: usize) {
        if call_button.call == CAB {
            self.cab_orders[elevator_id][call_button.floor as usize] = false;
        } else if call_button.call == HALL_DOWN || call_button.call == HALL_UP {
            self.hall_orders[call_button.floor as usize][call_button.call as usize] = false;
        } else {
            //Handle error
        }
        self.orders[call_button.floor as usize][call_button.call as usize] = false;
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

pub fn order_done(
    floor: u8,
    direction: u8,
    orders: Orders,
    order_completed_tx: &cbc::Sender<CallButton>,
) {
    if orders[floor as usize][direction as usize] {
        let _ = order_completed_tx.send(CallButton {
            floor,
            call: direction,
        });
    }
    if orders[floor as usize][CAB as usize] {
        let _ = order_completed_tx.send(CallButton { floor, call: CAB });
    }
}
