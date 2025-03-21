use crate::elevio::elev;

// pub fn direction_opposite(direction:: Direction) -> u8 {
//     match direction {
//         HALL_UP => HALL_DOWN,
//         HALL_DOWN => HALL_UP,
//         _ => CAB, // fix error handling
//     }
// }

// pub fn call_to_md(direction:: Direction) -> u8 {
//     match direction {
//         HALL_DOWN => DIRN_DOWN,
//         HALL_UP => DIRN_UP,
//         _ => DIRN_STOP, //fix error handling
//     }
// }

// pub fn direction_to_string(direction:: Direction) -> String {
//     match direction {
//         HALL_DOWN => "down".to_string(),
//         HALL_UP => "up".to_string(),
//         _ => "stop".to_string(), //fix
//     }
// }

#[repr(u8)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    Up = elev::HALL_UP,
    Down = elev::HALL_DOWN,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }

    pub fn to_motor_direction(&self) -> u8 {
        match self {
            Direction::Up => elev::DIRN_UP,
            Direction::Down => elev::DIRN_DOWN,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Direction::Up => "up".to_string(),
            Direction::Down => "down".to_string(),
        }
    }
}
