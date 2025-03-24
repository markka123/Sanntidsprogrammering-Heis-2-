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

    pub fn order_at_floor_in_direction(&mut self, floor: u8, direction: state::Direction) -> bool {
        self.orders[floor as usize][direction as usize] || self.orders[floor as usize][elev::CAB as usize]
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
        order_completed_tx: &cbc::Sender<poll::CallButton>,
    ) {
        if self.orders[current_floor as usize][direction as usize] {
            let _ = order_completed_tx.send(poll::CallButton {
                floor: current_floor as u8,
                call: direction as u8,
            });
        }
        if self.orders[current_floor as usize][elev::CAB as usize] {
            let _ = order_completed_tx.send(poll::CallButton {
                floor: current_floor as u8,
                call: elev::CAB,
            });
        }
    }
}
