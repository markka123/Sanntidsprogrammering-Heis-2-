use std::time;
use std::env;

pub const ELEV_NUM_FLOORS: u8 = 4;
pub const ELEV_NUM_ELEVATORS: u8 = 2;
pub const ELEV_NUM_BUTTONS: u8 = 3;

pub const ELEV_IP: &str = "10.100.23.33";
pub const ELEV_ID: u8 = 0;

pub const DOOR_TIMER_DURATION: time::Duration = time::Duration::from_secs(3);
pub const MOTOR_TIMER_DURATION: time::Duration = time::Duration::from_secs(4);
pub const MASTER_TIMER_DURATION: time::Duration = time::Duration::from_secs(2);
pub const NETWORK_TIMER_DURATION: time::Duration = time::Duration::from_secs(1);

pub const POLL_PERIOD: time::Duration = time::Duration::from_millis(25);
pub const UDP_POLL_PERIOD: time::Duration = time::Duration::from_millis(1);

pub const UNCONFIRMED_ORDERS_TRANSMIT_PERIOD: time::Duration = time::Duration::from_millis(30);
pub const STATE_TRANSMIT_PERIOD: time::Duration = time::Duration::from_millis(10);
pub const MASTER_TRANSMIT_PERIOD: time::Duration = time::Duration::from_millis(20);

pub const UDP_PORT: u16 = 15058;
pub const BROADCAST_IP: &str = "255.255.255.255";



pub fn fetch_command_line_arguments() -> (u16, u8) {
    let default_port = 15657;
    let default_elevator_id = 0;

    let mut port = default_port;
    let mut elevator_id = default_elevator_id;

    for arguments in env::args().skip(1) {
        if let Some(value) = arguments.strip_prefix("port=") {
            if let Ok(parsed_port) = value.parse::<u16>() {
                port = parsed_port;
            }
        } else if let Some(value) = arguments.strip_prefix("id=") {
            if let Ok(parsed_id) = value.parse::<u8>() {
                elevator_id = parsed_id;
            }
        }
    }
    
    (port, elevator_id)
}