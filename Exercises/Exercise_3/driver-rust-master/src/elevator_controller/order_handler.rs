use crate::elevio::elev;
use crate::elevio::poll as p;
use crate::button_handler::create_order as co;
use crate::elevator_controller::state_machine as sm;
use crate::elevio::elev::{HALL_UP, HALL_DOWN, CAB, DIRN_DOWN, DIRN_STOP, DIRN_UP};


pub fn setDirection(elevatorVariables: &sm::ElevatorVariables,  elevator: &elev::Elevator, order: &co::createOrder) { //endre fra CallButton type 
    //floor, stop and obstruction are updatded throug rx, the global varables can be used here
     {
        dirn =
            if elevatorVariables.currentFloor < order.floor {
                DIRN_UP;
                elevatorVariables.direction = DIRN_UP;
            } else if elevatorVariables.currentFloor > order.floor {
                DIRN_DOWN;
                elevatorVariables.direction = DIRN_DOWN;
            } else {
                DIRN_STOP;
                elevatorVariables.direction = DIRN_STOP;
            };
        elevator.motor_direction(dirn);
    }
    
}

#[derive(Clone, Debug)]
pub struct OrderArray {
    pub orderArr: Vec<&p::CallButton>,
    pub size: usize,
}

pub fn reorderOrderQueue(elevator: &ElevatorVariables, order_array: &mut OrderArray) {
    if order_array.size == 0 {
        return;
    }

    let current_order = &order_array.orderArr[0];
    for i in 1..order_array.size {
        let o = &order_array.orderArr[i];
        let should_reorder = match elevator.direction {
            DIRN_UP => (o.call == HALL_UP || o.call == CAB) && (elevator.currentFloor < o.floor && o.floor < current_order.floor),
            DIRN_DOWN => (o.call == HALL_DOWN || o.call == CAB) && (elevator.currentFloor > o.floor && o.floor > current_order.floor),
            _ => false,
        };

        if should_reorder {
            order_array.orderArr.insert(0, o.clone());
            order_array.orderArr.remove(i+1);
        }
    }
}