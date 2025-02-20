use std::time::*;

pub const elev_num_floors: u8 = 3;
pub const elev_num_elevators: u8 = 1;
pub const elev_num_buttons: u8 = 3;
pub const elev_id: u8 = 0;

pub const door_open_duration: Duration = Duration::from_secs(3);

pub const udp_port: u16 = 15000;
pub const broadcast_ip: &str = "255.255.255.255";
