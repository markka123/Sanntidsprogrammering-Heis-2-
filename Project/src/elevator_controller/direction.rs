use crate::elevio::elev::{DIRN_DOWN, DIRN_UP, HALL_DOWN, HALL_UP, CAB};

pub fn direction_opposite(direction: u8) -> u8 {
    match direction {
        HALL_UP => HALL_DOWN,
        HALL_DOWN => HALL_UP,
        _ => CAB, // fix error handling
    }
}

pub fn call_to_md(direction: u8) -> u8 {
    match direction {
        HALL_DOWN => DIRN_DOWN,
        HALL_UP => DIRN_UP,
        _ => CAB, //fix error handling
    }
}