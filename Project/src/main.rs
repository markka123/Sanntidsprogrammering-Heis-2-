use crossbeam_channel as cbc;
use driver_rust::config::config;
use driver_rust::distributor;
use driver_rust::elevator_controller;
use driver_rust::elevator_controller::lights;
use driver_rust::elevator_controller::orders;
use driver_rust::elevio;
use driver_rust::elevio::elev as e;
use driver_rust::network;
use driver_rust::network::udp;

use std::thread::*;
use std::env;

fn main() -> std::io::Result<()> {
    
    let (port, elevator_id) = fetch_command_line_args();

    let addr = format!("localhost:{}", port);

    let elevator = e::Elevator::init(&addr, config::ELEV_NUM_FLOORS)?;

    let (new_order_tx, new_order_rx) = cbc::unbounded::<orders::Orders>();
    let (new_state_tx, new_state_rx) = cbc::unbounded::<elevator_controller::state::State>();
    let (order_completed_tx, order_completed_rx) = cbc::unbounded::<elevio::poll::CallButton>();

    {
        let elevator = elevator.clone();
        spawn(move || {
            elevator_controller::elevator_fsm::elevator_fsm(
                &elevator,
                new_order_rx,
                order_completed_tx,
                &new_state_tx,
            )
        });
    }

    {
        let elevator = elevator.clone();
        spawn(move || {
            distributor::distributor::distributor(
                &elevator,
                elevator_id,
                new_state_rx,
                order_completed_rx,
                new_order_tx,
            )
        });
    }

    Ok(())
}



pub fn fetch_command_line_args() -> (u16, u8) {

    let default_port = 15657;
    let default_elevator_id = 0;

    let command_line_args: Vec<String> = env::args().collect();

    let port = command_line_args.get(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(default_port);

    let elevator_id = command_line_args.get(2)
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(default_elevator_id);

    (port, elevator_id)
}