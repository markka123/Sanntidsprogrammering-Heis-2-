use crate::config::config;
use crate::elevio::elev;
use crate::elevator::orders;


pub fn set_lights(elevator_orders: &orders::ElevatorOrders, elevator : elev::Elevator) {
    for floor in 0..config::ELEV_NUM_FLOORS {
        for call in 0..(config::ELEV_NUM_BUTTONS-1) {
                elevator.call_button_light(floor, call, elevator_orders.hall_orders[floor as usize][call as usize]);
        }
        elevator.call_button_light(floor, elev::CAB, elevator_orders.orders[floor as usize][elev::CAB as usize]);
    }
}