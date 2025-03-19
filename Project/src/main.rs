use crossbeam_channel as cbc;
use driver_rust::config::config;
use driver_rust::elevator_controller::orders;
use driver_rust::elevio::elev;
use driver_rust::elevio::poll;
use driver_rust::elevator_controller::elevator_fsm;
use driver_rust::elevator_controller::state;
use driver_rust::distributor::distributor;

use std::thread::*;
use std::env;

fn main() -> std::io::Result<()> {
    
    let (port, elevator_id) = fetch_command_line_args();

    let addr = format!("localhost:{}", port);

    let elevator = elev::Elevator::init(&addr, config::ELEV_NUM_FLOORS)?;

    let (new_order_tx, new_order_rx) = cbc::unbounded::<(orders::Orders, orders::HallOrders)>();
    let (new_state_tx, new_state_rx) = cbc::unbounded::<state::State>();
    let (order_completed_tx, order_completed_rx) = cbc::unbounded::<poll::CallButton>();

    {
        let elevator = elevator.clone();
        spawn(move || {
            elevator_fsm::elevator_fsm(
                &elevator,
                new_order_rx,
                order_completed_tx,
                new_state_tx,
            )
        });
    }

    {
        let elevator = elevator.clone();
        spawn(move || {
            distributor::distributor(
                &elevator,
                elevator_id,
                new_state_rx,
                order_completed_rx,
                new_order_tx,
            )
        });
    }

    loop {

    }
    Ok(())
}



pub fn fetch_command_line_args() -> (u16, u8) {

    let default_port = 15657;
    let default_elevator_id = 0;

    let mut port = default_port;
    let mut elevator_id = default_elevator_id;

    for arg in env::args().skip(1) {
        if let Some(value) = arg.strip_prefix("port=") {
            if let Ok(parsed_port) = value.parse::<u16>() {
                port = parsed_port;
            }
        } else if let Some(value) = arg.strip_prefix("id=") {
            if let Ok(parsed_id) = value.parse::<u8>() {
                elevator_id = parsed_id;
            }
        }
    }
    (port, elevator_id)
}