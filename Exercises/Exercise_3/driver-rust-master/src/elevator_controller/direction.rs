use crate::elevio::elev::{DIRN_DOWN, DIRN_STOP, DIRN_UP, HALL_DOWN, HALL_UP};

pub fn direction_opposite(direction: u8) -> u8 {
    match direction {
        DIRN_DOWN => DIRN_UP,
        DIRN_UP => DIRN_DOWN,
        _ => DIRN_STOP,
    }
}

pub fn motor_direction_to_hall_button(direction: u8) -> u8 {
    match direction {
        DIRN_DOWN => HALL_DOWN,
        DIRN_UP => HALL_UP,
        _ => 125, //fix error handling
    }
}