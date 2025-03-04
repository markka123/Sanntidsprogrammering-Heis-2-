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
use crate::cost_function::cost_function;

use crossbeam_channel as cbc;
use crossbeam_channel::select;
use std::array;
use std::sync::Arc;
use std::thread::*;
use std::time::*;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    CallMsg((u8, [u8; 3])),
    StateMsg((u8, State)),
    AllAssignedOrdersMsg((u8, Value)),
    // AssignedOrders([Orders; config::ELEV_NUM_ELEVATORS as usize]),
    // HallOrders()
}

pub const NEW_ORDER: u8 = 0;
pub const COMPLETED_ORDER: u8 = 1;
pub type States = [elevator_fsm::State; config::ELEV_NUM_ELEVATORS as usize];


pub fn distributor(
    elevator: &e::Elevator,
    elevator_id: u8,
    new_state_rx: cbc::Receiver<State>,
    order_completed_rx: cbc::Receiver<CallButton>,
    new_order_tx: cbc::Sender<orders::Orders>,
) {
    let socket = udp::create_udp_socket().expect("Failed to create UDP socket");
    let socket_receiver = Arc::clone(&socket);
    let socket_transmitter = Arc::clone(&socket);
    let mut states:States = create_states();

    let (message_tx, message_rx) = cbc::unbounded::<Message>();
    let (master_transmit_tx, master_transmit_rx) = cbc::unbounded::<String>();
    let (master_activate_tx, master_activate_rx) = cbc::unbounded::<()>();
    let (call_button_tx, call_button_rx) = cbc::unbounded::<CallButton>();

    {
        let elevator = elevator.clone();
        spawn(move || poll::call_buttons(elevator, call_button_tx, config::POLL_PERIOD));
    }

    {
        spawn(move || receiver::receiver(
            message_tx,
            master_activate_tx, 
            socket_receiver,
            elevator_id
        ));
    }
    {
        spawn(move || {
            transmitter::transmitter(
                elevator_id,
                call_button_rx,
                new_state_rx,
                order_completed_rx,
                master_transmit_rx,
                socket_transmitter
            )
        });
    }

    let mut all_orders = AllOrders::init();
    
    let mut master_ticker = cbc::never();
    if elevator_id == 0 {
        master_ticker = cbc::tick(config::MASTER_TRANSMIT_PERIOD);
    }
    

    loop {
        lights::set_lights(&all_orders, elevator.clone(), elevator_id);
        // sleep(Duration::from_millis(100));
        select! {
            recv(message_rx) -> message => {
                match message {
                    Ok(Message::CallMsg(call_msg)) => {
                        let (id, msg_array) = call_msg;
                        let msg_type = msg_array[0];
                        let new_order = CallButton{
                            floor: msg_array[1],
                            call: msg_array[2],
                        };
                        match msg_type {
                            NEW_ORDER => {
                                all_orders.add_order(new_order, id as usize);
                            },
                            COMPLETED_ORDER => {
                                all_orders.remove_order(new_order, id as usize);
                            },
                            _ => {
                                //Handle error
                            }
                        }
                        // new_order_tx.send(all_orders.assigned_orders[elevator_id]).unwrap();
                    },
                    Ok(Message::StateMsg(state_msg)) => {
                        let (id, state) = state_msg;
                        states[id as usize] = state;
                    },
                    Ok(Message::AllAssignedOrdersMsg((_, all_assigned_orders_str))) => {

                        // let all_assigned_orders_map: HashMap<u8, [[bool; 3]; config::ELEV_NUM_FLOORS as usize]> = serde_json::from_str(&all_assigned_orders_str).unwrap();
                        // if !(states[elevator_id as usize].motorstop || states[elevator_id as usize].emergency_stop || states[elevator_id as usize].obstructed) {
                        //     if let Some(assigned_orders) = all_assigned_orders_map.get(&elevator_id) {
                        //         // println!("Assigned orders: {:#?}", assigned_order);
                        //         // all_orders.assigned_orders = assigned_orders;
                        //         new_order_tx.send(*assigned_orders).unwrap();
                        //     } else {
                        //         println!("ID not found!");
                        //     }
                        // }
                        println!("Recieving!");
                            
                    },
                    Err(e) => {
                        println!("Received message of unexpected format");
                        println!("{:#?}", e);
                    }
                }
            },
            recv(master_activate_rx) -> _ => {
                master_ticker = cbc::tick(config::MASTER_TRANSMIT_PERIOD);
            },
            recv(master_ticker) -> _ => {
                let assigned_orders_str = cost_function::assign_orders(&states, &all_orders.cab_orders, &all_orders.hall_orders);
            

                master_transmit_tx.send(assigned_orders_str).unwrap();
                // println!("Hei");
                
            }
        }
    }
}

pub fn create_states() -> States {
    std::array::from_fn(|_| elevator_fsm::State {
        obstructed: false,
        motorstop: false,
        emergency_stop: false,
        behaviour: elevator_fsm::Behaviour::Idle,
        floor: 0,
        direction: e::HALL_DOWN,
    })
}
