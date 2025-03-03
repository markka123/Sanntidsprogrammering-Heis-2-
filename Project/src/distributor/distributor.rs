#![allow(dead_code)]
use crate::config::config;
use crate::distributor::receiver;
use crate::distributor::transmitter;
use crate::elevator_controller::direction;
use crate::elevator_controller::elevator_fsm;
use crate::elevator_controller::elevator_fsm::State;
use crate::elevator_controller::lights;
use crate::elevator_controller::orders;
use crate::elevator_controller::orders::AllOrders;
use crate::elevio::elev as e;
use crate::elevio::poll;
use crate::elevio::poll::CallButton;
use crate::network::udp;
use crossbeam_channel as cbc;
use crossbeam_channel::select;
use std::array;
use std::sync::Arc;
use std::thread::*;
use std::time::*;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::distributor::receiver::Message;

pub const NEW_ORDER: u8 = 0;
pub const COMPLETED_ORDER: u8 = 1;
pub type States = [elevator_fsm::State; config::ELEV_NUM_ELEVATORS as usize];


pub fn distributor(
    elevator: &e::Elevator,
    new_state_rx: cbc::Receiver<State>,
    order_completed_rx: cbc::Receiver<CallButton>,
    new_order_tx: cbc::Sender<orders::Orders>,
) {
    let socket = udp::create_udp_socket().expect("Failed to create UDP socket");
    let socket_receiver = Arc::clone(&socket);
    let socket_transmitter = Arc::clone(&socket);
    
    let master_ip = config::BROADCAST_IP;    
    let states:States = create_states();

    let (order_msg_tx, order_msg_rx) = cbc::unbounded::<[u8; 3]>();
    let (master_activate_tx, master_activate_rx) = cbc::unbounded::<()>();
    let (call_button_tx, call_button_rx) = cbc::unbounded::<CallButton>();

    {
        let elevator = elevator.clone();
        spawn(move || poll::call_buttons(elevator, call_button_tx, config::POLL_PERIOD));
    }

    {
        spawn(move || receiver::receiver(
            order_msg_tx,
            master_activate_tx, 
            socket_receiver
        ));
    }
    {
        spawn(move || {
            transmitter::transmitter(
                call_button_rx,
                new_state_rx,
                order_completed_rx,
                master_activate_rx,
                socket_transmitter,
                &master_ip,
            )
        });
    }

    let mut all_orders = AllOrders::init();

    loop {
        lights::set_lights(&all_orders, elevator.clone());
        // sleep(Duration::from_millis(100));
        select! {
            recv(order_msg_rx) -> a => {
                let msg_array = a.unwrap();
                let msg_type = msg_array[0];
                let new_order = CallButton{
                    floor: msg_array[1],
                    call: msg_array[2],
                };
                match msg_type {
                    NEW_ORDER => {
                        all_orders.add_order(new_order, config::ELEV_ID as usize);
                    },
                    COMPLETED_ORDER => {
                        all_orders.remove_order(new_order, config::ELEV_ID as usize);
                    },
                    _ => {
                        //Handle error
                    }
                }
                new_order_tx.send(all_orders.orders).unwrap();
            },
            // recv(master_activate_rx) -> _ => {
            //     println!("Master activated");
            // },
        }
    }
}


pub fn create_states() -> States {
    std::array::from_fn(|_| elevator_fsm::State {
        obstructed: false,
        motorstop: true,
        emergency_stop: false,
        behaviour: elevator_fsm::Behaviour::Idle,
        floor: 0,
        direction: e::HALL_DOWN,
    })
}
