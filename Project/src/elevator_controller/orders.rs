use crate::config::config;
use crate::elevio::elev;
use crate::elevio::poll;

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

    pub fn order_at_floor_in_direction(&mut self, floor: u8, direction: u8) -> bool {
        self.orders[floor as usize][(direction) as usize]
            || self.orders[floor as usize][elev::CAB as usize]
    }

    pub fn order_in_direction(&mut self, floor: u8, dir: u8) -> bool {
        match dir {
            elev::HALL_UP => {
                for f in (floor + 1)..config::ELEV_NUM_FLOORS {
                    for b in 0..config::ELEV_NUM_BUTTONS {
                        if self.orders[f as usize][b as usize] {
                            return true;
                        }
                    }
                }
                false
            }
            elev::HALL_DOWN => {
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
        if self.orders[floor as usize][elev::CAB as usize] {
            let _ = order_completed_tx.send(poll::CallButton {
                floor,
                call: elev::CAB,
            });
        }
    }
}
