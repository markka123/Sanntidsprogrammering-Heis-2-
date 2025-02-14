#![allow(dead_code)]
use crossbeam_channel as cbc;
use crate::elevio::{self, elev as e};
use crate::elevio::elev::{HALL_UP, HALL_DOWN, CAB, DIRN_DOWN, DIRN_STOP, DIRN_UP};
use crate::elevio::poll::{CallButton};
use crate::config::config;


pub type Orders = [[bool; 3]; config::elev_num_floors as usize];

#[derive(Clone, Debug)]
pub struct AllOrders {
    // Init with: let matrix = Matrix::new(rows, cols, false);
    pub hall_orders: Vec<[bool; 2]>,
    pub cab_orders: Vec<Vec<bool>>,
    pub orders: Orders,
}

impl AllOrders {
    pub fn init() -> Self {
        let hall_orders = vec![[false; 2]; config::elev_num_floors as usize];
        let cab_orders = vec![vec![false; config::elev_num_floors as usize]; config::elev_num_elevators as usize];
        let orders = [[false; 3]; config::elev_num_floors as usize];
        Self { hall_orders,  cab_orders, orders}
    }
    pub fn add_order(&mut self, call_button: CallButton, _elevator_nr: usize, new_order_tx: &cbc::Sender<Orders>) {
        if CAB == call_button.call {
            self.cab_orders[_elevator_nr][call_button.floor as usize] = true;
        }
        else if call_button.call == HALL_DOWN || call_button.call == HALL_UP {
            self.hall_orders[call_button.floor as usize][call_button.call as usize] = true;
        }
        else {
            //Handle error
        }
        self.orders[call_button.floor as usize][call_button.call as usize] = true;
        new_order_tx.send(self.orders).unwrap();
    }
}

pub fn order_in_direction(orders: &Orders, floor: u8, dir: u8) -> bool {
    match dir {
        DIRN_UP => {
            for f in (floor)..config::elev_num_floors {
                for b in 0..config::elev_num_buttons {
                    if orders[f as usize][b as usize] {
                        return true;
                    }
                }
            }
            false
        },
        DIRN_DOWN => {
            for f in (0..floor-1).rev() {
                for b in 0..config::elev_num_buttons {
                    if orders[f as usize][b as usize] {
                        return true;
                    }
                }
            }
            false
        },
        _ => false,
    }
}

pub fn order_done(floor: u8, dir: u8, orders: Orders, delivered_order_tx: cbc::Sender<elevio::poll::CallButton>) {
}
