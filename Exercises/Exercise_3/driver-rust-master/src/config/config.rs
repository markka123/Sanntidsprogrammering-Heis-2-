use std::time::*;

pub const ELEV_NUM_FLOORS: u8 = 3;
pub const ELEV_NUM_ELEVATORS: u8 = 1;
pub const ELEV_NUM_BUTTONS: u8 = 3;
pub const ELEV_ID: u8 = 0;

pub const POLL_PERIOD: Duration = Duration::from_millis(25);
pub const MOTOR_TIMER_DURATION: Duration = Duration::from_secs(5);
pub const DOOR_TIMER_DURATION: Duration = Duration::from_secs(3);
