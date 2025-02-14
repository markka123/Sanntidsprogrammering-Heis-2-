pub const MASTER_IP: &str = "10.100.23.10"; // Change if needed
pub const BROADCAST_IP: &str = "10.100.23.255"; // Matches network config

pub const MASTER_HEARTBEAT_PORT: &str = "8000"; // Master listens here
pub const SLAVE_HEARTBEAT_PORT: &str = "8001";  // Slaves listen here
pub const ORDER_ASSIGNMENT_PORT: &str = "8002"; // Orders go here

pub const TIMEOUT_SECS: u64 = 5;