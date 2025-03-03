use crossbeam_channel as cbc;
use driver_rust::config::config;
use driver_rust::distributor;
use driver_rust::elevator_controller;
use driver_rust::elevator_controller::lights;
use driver_rust::elevator_controller::orders::{AllOrders, Orders};
use driver_rust::cost_function::cost_function::{assign_requests};
use driver_rust::elevio;
use driver_rust::elevio::elev as e;
use driver_rust::network;
use driver_rust::network::udp;
use std::sync::Arc;
use std::thread::*;
use std::env;

fn main() -> std::io::Result<()> {
    // let elevator_variables = vec![vec!["moving".to_string(), "2".to_string(), "up".to_string()]];
    // let cab_requests = vec![vec![false, false, true, true]];
    // let hall_requests = vec![vec![false, false], vec![true, false], vec![false, false], vec![false, true]];


    // let result = assign_requests(&elevator_variables, &cab_requests, &hall_requests);
    // println!("Result from executable:\n{}", result);
    
    println!("{}", 1.to_string());

    let args: Vec<String> = env::args().collect();

    let default_port = 15657;
    let port: u16 = if args.len() > 1 {
        match args[1].parse() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Warning: Invalid port provided. Using default: {}", default_port);
                default_port
            }
        }
    } else {
        default_port
    };
    let addr = format!("localhost:{}", port);


    let elevator = e::Elevator::init(&addr, config::ELEV_NUM_FLOORS)?;
    println!("Elevator started:\n{:#?}", elevator);

    let (new_order_tx, new_order_rx) = cbc::unbounded::<Orders>();
    let (emergency_reset_tx, emergency_reset_rx) = cbc::unbounded::<bool>();
    let (new_state_tx, new_state_rx) = cbc::unbounded::<elevator_controller::elevator_fsm::State>();
    let (order_completed_tx, order_completed_rx) = cbc::unbounded::<elevio::poll::CallButton>();

    {
        let elevator = elevator.clone();
        spawn(move || {
            elevator_controller::elevator_fsm::elevator_fsm(
                &elevator,
                new_order_rx,
                order_completed_tx,
                emergency_reset_tx,
                &new_state_tx,
            )
        });
    }

    {
        let elevator = elevator.clone();
        spawn(move || {
            distributor::distributor::distributor(
                &elevator,
                new_state_rx,
                order_completed_rx,
                new_order_tx,
            )
        });
    }

    let mut all_orders = AllOrders::init();

    loop {
        // cbc::select! {
        // recv(call_button_rx) -> a => {
        //     let call_button = a.unwrap();
        //     all_orders.add_order(call_button, config::ELEV_ID as usize, &new_order_tx);
        // },
        // recv(order_completed_rx) -> a => {
        //     let call_button = a.unwrap();
        //     all_orders.remove_order(call_button, config::ELEV_ID as usize, &new_order_tx);
        // },
        //     recv(emergency_reset_rx) -> _ => {
        //         all_orders = AllOrders::init();
        //         new_order_tx.send(all_orders.orders).unwrap();
        //     }
        // }
    }
    Ok(())
}
