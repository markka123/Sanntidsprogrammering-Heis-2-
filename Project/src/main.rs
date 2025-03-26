use driver_rust::config::config;
use driver_rust::config::fetch_arguments;
use driver_rust::elevator::orders;
use driver_rust::elevio::elev;
use driver_rust::elevio::poll;
use driver_rust::elevator::elevator_fsm;
use driver_rust::elevator::state;
use driver_rust::distributor::distributor;


use std::thread;
use crossbeam_channel as cbc;


fn main() -> std::io::Result<()> {
    
    let (port, elevator_id) = fetch_arguments::fetch_command_line_arguments();
    let address = format!("localhost:{}", port);

    let elevator = elev::Elevator::init(&address, config::ELEV_NUM_FLOORS)?;

    let (elevator_orders_tx, elevator_orders_rx) = cbc::unbounded::<(orders::Orders, orders::HallOrders)>();
    let (completed_order_tx, completed_order_rx) = cbc::unbounded::<poll::CallButton>();
    let (new_order_tx, new_order_rx) = cbc::unbounded::<poll::CallButton>();
    let (new_state_tx, new_state_rx) = cbc::unbounded::<state::State>();
    {
        let elevator = elevator.clone();
        thread::spawn(move || {
            elevator_fsm::elevator_fsm(
                &elevator,
                elevator_orders_rx,
                completed_order_tx,
                new_order_tx,
                new_state_tx,
            )
        });
    }

    {
        thread::spawn(move || {
            distributor::distributor(
                elevator_id,
                elevator_orders_tx,
                completed_order_rx,
                new_order_rx,
                new_state_rx,
                
            )
        });
    }

    loop {
        thread::park();
    }
}