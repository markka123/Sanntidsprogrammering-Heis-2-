use std::thread::*;
use std::time::*;
use crossbeam_channel as cbc;
use driver_rust::elevio;
use driver_rust::elevio::elev as e;
use driver_rust::elevator_controller;
use driver_rust::elevator_controller::orders::{AllOrders, Orders};
use driver_rust::offline_order_handler::offline_order_handler::{execute_offline_order};
use driver_rust::config::config;

fn main() -> std::io::Result<()> {
    
    let elevator = e::Elevator::init("localhost:15657", config::elev_num_floors)?;
    println!("Elevator started:\n{:#?}", elevator);

    let poll_period = Duration::from_millis(25);

    let (call_button_tx, call_button_rx) = cbc::unbounded::<elevio::poll::CallButton>();
    {
        let elevator = elevator.clone();
        spawn(move || elevio::poll::call_buttons(elevator, call_button_tx, poll_period));
    }

    let (floor_sensor_tx, floor_sensor_rx) = cbc::unbounded::<u8>();
    {
        let elevator = elevator.clone();
        spawn(move || elevio::poll::floor_sensor(elevator, floor_sensor_tx, poll_period));
    }

    let (stop_button_tx, stop_button_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || elevio::poll::stop_button(elevator, stop_button_tx, poll_period));
    }

    let (obstruction_tx, obstruction_rx) = cbc::unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || elevio::poll::obstruction(elevator, obstruction_tx, poll_period));
    }

    let mut dirn = e::DIRN_UP;
    if elevator.floor_sensor().is_none() {
        elevator.motor_direction(dirn);
    }

    
    let mut all_orders = AllOrders::init();
    
  // Example orderQueue
    let mut orders = [[false,false,false],[true,false,false],[false,false,true],[false,true,true]];
  
    // execute_offline_order();

    let (new_order_tx, new_order_rx) = cbc::unbounded::<Orders>();
    let (delivered_order_tx, delivered_order_rx) = cbc::unbounded::<elevio::poll::CallButton>();
    
    {
        let elevator = elevator.clone();
        spawn(move || elevator_controller::fsm::fsm_elevator(&elevator, floor_sensor_rx, stop_button_rx, obstruction_rx, new_order_rx, delivered_order_tx));
    }
    

    loop {
        cbc::select! {
            recv(call_button_rx) -> a => {
                let call_button = a.unwrap();
                all_orders.add_order(call_button.clone(), 0, &new_order_tx);
                elevator.call_button_light(call_button.floor, call_button.call, true);
            },
            // recv(floor_sensor_rx) -> a => {
            //     let floor = a.unwrap();
            //     println!("Floor: {:#?}", floor);
            //     dirn =
            //         if floor == 0 {
            //             e::DIRN_UP
            //         } else if floor == config::elev_num_floors-1 {
            //             e::DIRN_DOWN
            //         } else {
            //             dirn
            //         };
            //     elevator.motor_direction(dirn);
            // },
            // recv(stop_button_rx) -> a => {
            //     let stop = a.unwrap();
            //     println!("Stop button: {:#?}", stop);
            //     for f in 0..config::elev_num_floors {
            //         for c in 0..3 {
            //             elevator.call_button_light(f, c, false);
            //         }
            //     }
            // },
            // recv(obstruction_rx) -> a => {
            //     let obstr = a.unwrap();
            //     println!("Obstruction: {:#?}", obstr);
            //     elevator.motor_direction(if obstr { e::DIRN_STOP } else { dirn });
            // },
        }
    }
}
