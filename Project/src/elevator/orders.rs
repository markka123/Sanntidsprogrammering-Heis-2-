use crate::config::config;
use crate::elevio::elev;
use crate::elevio::poll;
use crate::elevator::state;

use crossbeam_channel as cbc;


pub type Orders = [[bool; 3]; config::ELEV_NUM_FLOORS as usize];
pub type HallOrders = [[bool; 2]; config::ELEV_NUM_FLOORS as usize];
pub type CabOrders = [[bool; config::ELEV_NUM_FLOORS as usize]; config::ELEV_NUM_ELEVATORS as usize];


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

    pub fn hall_at_floor_in_direction(&mut self, floor: u8, direction: state::Direction) -> bool {
        self.orders[floor as usize][direction as usize]
    }

    pub fn cab_at_floor(&mut self, floor: u8) -> bool {
        self.orders[floor as usize][elev::CAB as usize]
    }

    pub fn order_in_direction(&mut self, current_floor: u8, direction: state::Direction) -> bool {
        match direction {
            state::Direction::Up => {
                for floor in (current_floor + 1)..config::ELEV_NUM_FLOORS {
                    for call in 0..config::ELEV_NUM_BUTTONS {
                        if self.orders[floor as usize][call as usize] {
                            return true;
                        }
                    }
                }
                false
            }
            state::Direction::Down => {
                for floor in (0..current_floor).rev() {
                    for call in 0..config::ELEV_NUM_BUTTONS {
                        if self.orders[floor as usize][call as usize] {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }

    pub fn order_done(
        &mut self,
        current_floor: u8,
        direction: state::Direction,
        completed_order_tx: &cbc::Sender<poll::CallButton>,
    ) {        
        if self.orders[current_floor as usize][direction as usize] {
            let completed_order = poll::CallButton {
                floor: current_floor as u8,
                call: direction as u8,
            };
            completed_order_tx.send(completed_order).unwrap();
        }
        if self.orders[current_floor as usize][elev::CAB as usize] {
            let completed_order = poll::CallButton {
                floor: current_floor as u8,
                call: elev::CAB,
            };
            completed_order_tx.send(completed_order).unwrap();
        }
    }

    pub fn set_lights(&self, elevator : elev::Elevator) {
        for floor in 0..config::ELEV_NUM_FLOORS {
            for call in 0..(config::ELEV_NUM_BUTTONS-1) {
                    elevator.call_button_light(floor, call, self.hall_orders[floor as usize][call as usize]);
            }
            elevator.call_button_light(floor, elev::CAB, self.orders[floor as usize][elev::CAB as usize]);
        }
    }
}

