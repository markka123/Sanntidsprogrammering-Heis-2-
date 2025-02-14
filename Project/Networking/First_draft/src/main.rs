mod config;
mod networking;
mod message_variables;
mod master;
mod slave;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "master" {
        master::start_master();
    } else {
        let my_id = "10.100.23.17"; // Replace with actual IP
        slave::start_slave(my_id);
    }
}
