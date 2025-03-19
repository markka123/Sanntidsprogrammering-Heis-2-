use crate::config::config;
use crate::elevio::elev as e;
use crate::elevator_controller::orders;

pub fn set_lights(elevator_orders: &orders::ElevatorOrders, elevator : e::Elevator) {
    for f in 0..config::ELEV_NUM_FLOORS {
        for b in 0..(config::ELEV_NUM_BUTTONS-1) {
                elevator.call_button_light(f, b, elevator_orders.hall_orders[f as usize][b as usize]);
        }
        elevator.call_button_light(f, e::CAB, elevator_orders.orders[f as usize][e::CAB as usize]);
    }
}