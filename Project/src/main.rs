use project::config::config;
use project::config::fetch_arguments;
use project::elevator::orders;
use project::elevio::elev;
use project::elevio::poll;
use project::elevator::elevator_fsm;
use project::elevator::state;
use project::distributor::distributor;


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