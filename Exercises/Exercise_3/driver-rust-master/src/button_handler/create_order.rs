#![allow(dead_code)]

use crate::elevio::elev as e;
use crate::elevio::elev::{HALL_UP, HALL_DOWN, CAB, DIRN_DOWN, DIRN_STOP, DIRN_UP};
use crate::elevio::poll::{CallButton};

#[derive(Clone, Debug)]
pub struct AllOrders {
    // Init with: let matrix = Matrix::new(rows, cols, false);
    pub hall_orders: Vec<[bool; 2]>,
    pub cab_orders: Vec<Vec<bool>>,
}

impl AllOrders {
    pub fn init(nr_of_elevators: usize, nr_of_floors: usize) -> Self {
        let hall_orders = vec![[false; 2]; nr_of_floors];
        let cab_orders = vec![vec![false; nr_of_floors]; nr_of_elevators];
        Self { hall_orders,  cab_orders}
    }
    pub fn add_order(&mut self, call_button: CallButton, _elevator_nr: usize) {
        if CAB == call_button.call {
            self.cab_orders[_elevator_nr][call_button.floor as usize] = true;
        }
        else if call_button.call == HALL_DOWN || call_button.call == HALL_UP {
            self.hall_orders[call_button.floor as usize][call_button.call as usize] = true;
        }
        else {
            //Handle error
        }
    }
}