use std::time;

pub const ELEV_NUM_FLOORS: u8 = 4;
pub const ELEV_NUM_ELEVATORS: u8 = 3;
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