use crossbeam_channel as cbc;
use driver_rust::config::config;
use driver_rust::elevator_controller;
use driver_rust::elevator_controller::lights;
use driver_rust::elevator_controller::orders::{AllOrders, Orders};
use driver_rust::elevio;
use driver_rust::elevio::elev as e;
use std::thread::*;

fn main() -> std::io::Result<()> {
    let elevator = e::Elevator::init("localhost:15657", config::ELEV_NUM_FLOORS)?;
    println!("Elevator started:\n{:#?}", elevator);

    let (call_button_tx, call_button_rx) = cbc::unbounded::<elevio::poll::CallButton>();
    {
        let elevator = elevator.clone();
        spawn(move || elevio::poll::call_buttons(elevator, call_button_tx, config::POLL_PERIOD));
    }

    let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
    {
        let elevator = elevator.clone();
        spawn(move || elevio::poll::floor_sensor(elevator, floor_sensor_tx, config::POLL_PERIOD));
    }

    let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || elevio::poll::stop_button(elevator, stop_button_tx, config::POLL_PERIOD));
    }

    let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || elevio::poll::obstruction(elevator, obstruction_tx, config::POLL_PERIOD));
    }

    let (new_order_tx, new_order_rx) = cbc::unbounded::<Orders>();
    let (order_completed_tx, order_completed_rx) = cbc::unbounded::<elevio::poll::CallButton>();
    let (emergency_reset_tx, emergency_reset_rx) = cbc::unbounded::<bool>();

    {
        let elevator = elevator.clone();
        spawn(move || {
            elevator_controller::elevator_fsm::elevator_fsm(
                &elevator,
                floor_sensor_rx,
                stop_button_rx,
                obstruction_rx,
                new_order_rx,
                order_completed_tx,
                emergency_reset_tx,
            )
        });
    }

    let mut all_orders = AllOrders::init();

    loop {
        lights::set_lights(&all_orders, elevator.clone());

        cbc::select! {
            recv(call_button_rx) -> a => {
                let call_button = a.unwrap();
                all_orders.add_order(call_button, config::ELEV_ID as usize, &new_order_tx);
            },
            recv(order_completed_rx) -> a => {
                let call_button = a.unwrap();
                all_orders.remove_order(call_button, config::ELEV_ID as usize, &new_order_tx);
            },
            recv(emergency_reset_rx) -> _ => {
                all_orders = AllOrders::init();
                new_order_tx.send(all_orders.orders).unwrap();
            }
        }
    }
}
