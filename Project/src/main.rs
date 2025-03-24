use driver_rust::config::config;
use driver_rust::elevator::orders;
use driver_rust::elevio::elev;
use driver_rust::elevio::poll;
use driver_rust::elevator::elevator_fsm;
use driver_rust::elevator::state;
use driver_rust::distributor::distributor;

use std::thread::spawn;
use crossbeam_channel as cbc;


fn main() -> std::io::Result<()> {
    
    let (port, elevator_id) = config::fetch_command_line_arguments();

    let address = format!("localhost:{}", port);

    let elevator = elev::Elevator::init(&address, config::ELEV_NUM_FLOORS)?;

    let (elevator_orders_tx, elevator_orders_rx) = cbc::unbounded::<(orders::Orders, orders::HallOrders)>();
    let (new_state_tx, new_state_rx) = cbc::unbounded::<state::State>();
    let (order_completed_tx, order_completed_rx) = cbc::unbounded::<poll::CallButton>();
    let (order_new_tx, order_new_rx) = cbc::unbounded::<poll::CallButton>();

    {
        let elevator = elevator.clone();
        spawn(move || {
            elevator_fsm::elevator_fsm(
                &elevator,
                elevator_orders_rx,
                order_completed_tx,
                order_new_tx,
                new_state_tx,
            )
        });
    }

    {
        spawn(move || {
            distributor::distributor(
                elevator_id,
                elevator_orders_tx,
                order_completed_rx,
                order_new_rx,
                new_state_rx,
                
            )
        });
    }

    loop {

    }

    Ok(())
}