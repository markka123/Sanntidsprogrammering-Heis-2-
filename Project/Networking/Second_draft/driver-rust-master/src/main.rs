use std::thread::*;
use std::time::*;
use crossbeam_channel as cbc;
use driver_rust::elevio;
use driver_rust::elevio::elev as e;
use driver_rust::elevator_controller;
use driver_rust::elevator_controller::orders::{AllOrders, Orders};
use driver_rust::offline_order_handler::offline_order_handler::{execute_offline_order};
use driver_rust::config::config;
use driver_rust::elevator_controller::lights;
use driver_rust::distributor;

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

    let (new_state_tx, new_state_rx) = cbc::unbounded::<elevator_controller::fsm::State>();
    let (new_order_tx, new_order_rx) = cbc::unbounded::<Orders>();
    let (delivered_order_tx, delivered_order_rx) = cbc::unbounded::<elevio::poll::CallButton>();
    let (master_activate_tx, master_activate_rx) = cbc::unbounded::<()>();
    let (master_deactivate_tx, master_deactivate_rx) = cbc::unbounded::<()>();
    
    {
        spawn(move ||distributor::receiver::receiver(&new_order_tx, &master_activate_tx));
    }

    {
        spawn(move ||distributor::transmitter::transmitter(&call_button_rx, &delivered_order_rx, &new_state_rx));
    }

    {
        spawn(move ||distributor::receiver::master_receiver(&master_activate_rx, &master_deactivate_tx, &master_deactivate_rx));
    }

    {
        spawn(move ||distributor::transmitter::master_transmitter(&master_activate_rx, &master_deactivate_rx));
    }

    {
        let elevator = elevator.clone();
        spawn(move || elevator_controller::fsm::fsm_elevator(&elevator, floor_sensor_rx, stop_button_rx, obstruction_rx, new_order_rx, delivered_order_tx, &new_state_tx));
    }
    

    loop {
        lights::set_lights(&all_orders, elevator.clone());
        cbc::select! {
            recv(call_button_rx) -> a => {
                let call_button = a.unwrap();
                all_orders.add_order(call_button.clone(), config::elev_id as usize, &new_order_tx);
            },
            recv(delivered_order_rx) -> a => {
                let call_button = a.unwrap();
                all_orders.remove_order(call_button, config::elev_id as usize, &new_order_tx);
            },
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
