use crate::elevio::elev as e;
use crate::elevio::poll as p;
use crate::state_machine as sm

pub fn setDirection(elevatorVariables: &sm::ElevatorVariables,  elevator: &e::Elevator, order: &p::CallButton) { //endre fra CallButton type 
    //floor, stop and obstruction are updatded throug rx, the global varables can be used here
     {
        dirn =
            if elevatorVariables.currentFloor < order.floor {
                e::DIRN_UP
            } else if elevatorVariables.currentFloor > order.floor {
                e::DIRN_DOWN
            } else {
                e::DIRN_STOP
            };
        elevator.motor_direction(dirn);
    }
    
}