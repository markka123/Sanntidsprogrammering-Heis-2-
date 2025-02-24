use std::thread::*;
use std::time::*;
use crossbeam_channel as cbc;
use driver_rust::elevio;
use driver_rust::elevio::elev as e;
use driver_rust::elevator_controller;
use driver_rust::elevator_controller::orders::{AllOrders, Orders};
use driver_rust::network::udp;
use driver_rust::offline_order_handler::offline_order_handler::{execute_offline_order};
use driver_rust::config::config;
use driver_rust::elevator_controller::lights;
use driver_rust::distributor;
use driver_rust::network;
use std::sync::Arc;

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





    let socket = udp::create_udp_socket().expect("Failed to create UDP socket");
    let socket_receiver = Arc::clone(&socket);
    let socket_transmitter = Arc::clone(&socket);

    let (master_activate_transmitter_tx, master_activate_transmitter_rx) = cbc::unbounded::<()>();
    let (master_activate_receiver_tx, master_activate_receiver_rx) = cbc::unbounded::<()>();
    let (master_deactivate_tx, master_deactivate_rx) = cbc::unbounded::<()>();
    let (new_order_tx, new_order_rx) = cbc::unbounded::<Orders>();
    let (new_order_2_tx, new_order_2_rx) = cbc::unbounded::<Orders>();
    let (delivered_order_tx, delivered_order_rx) = cbc::unbounded::<elevio::poll::CallButton>();
    let (new_state_tx, new_state_rx) = cbc::unbounded::<elevator_controller::fsm::State>();
    
    {
        spawn(move ||distributor::receiver::receiver(new_order_2_tx, master_activate_transmitter_tx, master_activate_receiver_tx, socket_receiver));
    }
    {
        spawn(move ||distributor::transmitter::transmitter(call_button_rx, delivered_order_rx, new_state_rx, socket_transmitter));
    }
    {
        spawn(move ||distributor::receiver::master_receiver(master_activate_receiver_rx, master_deactivate_tx));
    }
    {
        spawn(move ||distributor::transmitter::master_transmitter(master_activate_transmitter_rx, master_deactivate_rx));
    }
    {
        let elevator = elevator.clone();
        spawn(move || elevator_controller::fsm::fsm_elevator(&elevator, floor_sensor_rx, stop_button_rx, obstruction_rx, new_order_rx, delivered_order_tx, &new_state_tx));
    }
    

    loop {
        lights::set_lights(&all_orders, elevator.clone());
        // cbc::select! {
        //     recv(call_button_rx) -> a => {
        //         let call_button = a.unwrap();
        //         all_orders.add_order(call_button.clone(), config::elev_id as usize, &new_order_tx);
        //         //new_order_tx.send(call_button).unwrap();
        //         button_call_tx.send(call_button).unwrap();

        //     },
        //     recv(delivered_order_rx) -> a => {
        //         let call_button = a.unwrap();
        //         all_orders.remove_order(call_button, config::elev_id as usize, &new_order_tx);
        //     }
        // }
    }
}
