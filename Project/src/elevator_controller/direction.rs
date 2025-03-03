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

pub fn direction_to_string(direction: u8) -> String {
    match direction {
        HALL_DOWN => "down".to_string(),
        HALL_UP => "up".to_string(),
        _ => "stop".to_string(), //fix
    }
}