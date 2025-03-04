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
use std::sync::Mutex;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    CallMsg((u8, [u8; 3])),
    StateMsg((u8, State)),
    AllAssignedOrdersMsg((u8, Value)),
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
    let mut all_orders = AllOrders::init();
    let mut offline_orders: orders::Orders = [[false; 3]; config::ELEV_NUM_FLOORS as usize];

    let mut master_ticker = cbc::tick(config::MASTER_TRANSMIT_PERIOD);

    let pending_orders: Arc<Mutex<Vec<(u8, CallButton)>>> = Arc::new(Mutex::new(Vec::new()));
    let pending_orders_clone = Arc::clone(&pending_orders);

    let mut is_online = true;


    let socket = udp::create_udp_socket().expect("Failed to create UDP socket");
    let socket_receiver = Arc::clone(&socket);
    let socket_transmitter = Arc::clone(&socket);
    let mut states:States = create_states();

    let (message_tx, message_rx) = cbc::unbounded::<Message>();
    let (is_online_tx, is_online_rx) = cbc::unbounded::<bool>();
    let (master_transmit_tx, master_transmit_rx) = cbc::unbounded::<String>();
    let (master_activate_tx, master_activate_rx) = cbc::unbounded::<bool>();
    let (call_button_tx, call_button_rx) = cbc::unbounded::<CallButton>();
    let (call_msg_tx, call_msg_rx) = cbc::unbounded::<(u8, CallButton)>();
    let (is_online_tx, is_online_rx) = cbc::unbounded::<bool>();



    {
        let elevator = elevator.clone();
        spawn(move || poll::call_buttons(elevator, call_button_tx, config::POLL_PERIOD));
    }

    {
        spawn(move || receiver::receiver(
            message_tx,
            master_activate_tx,
            is_online_tx, 
            socket_receiver,
            elevator_id
        ));
    }
    {
        spawn(move || {
            transmitter::transmitter(
                elevator_id,             
                new_state_rx,
                master_transmit_rx,
                call_msg_rx,
                pending_orders_clone,
                socket_transmitter,
            )
        });
    }

    let mut all_orders = AllOrders::init();
    
    let mut master_ticker = cbc::never();
    

    loop {
        lights::set_lights(&all_orders, elevator.clone(), elevator_id);
        select! {
            recv(call_button_rx) -> call_button => {
                let call_button = call_button.unwrap();
                let msg_type = NEW_ORDER;
                
                if !is_online {
                    offline_orders[call_button.floor as usize][call_button.call as usize] = true;
                    new_order_tx.send(offline_orders).unwrap();
                }
                call_msg_tx.send((msg_type, call_button)).unwrap();
            },
            recv(order_completed_rx) -> order_completed => {
                let order_completed = order_completed.unwrap();
                let msg_type = COMPLETED_ORDER;
                
                if !is_online {
                    offline_orders[order_completed.floor as usize][order_completed.call as usize] = false;
                    pending_orders.lock().unwrap().retain(|(msg, order)| *msg != msg_type || order.floor != order_completed.floor || order.call != order_completed.call);
                    new_order_tx.send(offline_orders).unwrap(); 
                }
                call_msg_tx.send((msg_type, order_completed)).unwrap();
            },
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
                    },
                    Ok(Message::StateMsg(state_msg)) => {
                        let (id, state) = state_msg;
                        states[id as usize] = state;
                    },
                    Ok(Message::AllAssignedOrdersMsg((master_id, all_assigned_orders_str))) => {

                        let all_assigned_orders_map: HashMap<u8, [[bool; 3]; config::ELEV_NUM_FLOORS as usize]> = serde_json::from_value(all_assigned_orders_str).unwrap();
                        if !(states[elevator_id as usize].motorstop || states[elevator_id as usize].emergency_stop || states[elevator_id as usize].obstructed) {
                            if let Some(assigned_orders) = all_assigned_orders_map.get(&elevator_id) {
                                new_order_tx.send(*assigned_orders).unwrap();
                            } else {
                            }
                        }
                        pending_orders.lock().unwrap().retain(|(order_type, order)| {
                            if order.call == e::HALL_UP || order.call == e::HALL_DOWN {
                                if *order_type == NEW_ORDER {
                                    let assigned_any = all_assigned_orders_map.iter().any(|(_, assigned_orders)| {
                                        assigned_orders[order.floor as usize][order.call as usize]
                                    });
                                    return !assigned_any;      
                                }
                                else if *order_type == COMPLETED_ORDER {
                                    let not_assigned_any = all_assigned_orders_map.iter().all(|(_, assigned_orders)| {
                                        !assigned_orders[order.floor as usize][order.call as usize]
                                    });
                                    return not_assigned_any;
                                }
                            }
                            else if order.call == e::CAB {
                                if let Some(assigned_orders) = all_assigned_orders_map.get(&elevator_id) {
                                    if (assigned_orders[order.floor as usize][order.call as usize] && *order_type == NEW_ORDER) 
                                    || (!assigned_orders[order.floor as usize][order.call as usize] && *order_type == COMPLETED_ORDER) {
                                        // //println!("Poped from pending orders");
                                        return false;
                                    }
                                }
                            } return true;
                            
                        });
                    }
                    Err(e) => {
                        //eprintln!("Received message of unexpected format");
                        //eprintln!("{:#?}", e);
                    }
                }
            },
            recv(master_activate_rx) -> _ => {
                master_ticker = cbc::tick(config::MASTER_TRANSMIT_PERIOD);
            },
            recv(master_ticker) -> _ => {
                let assigned_orders_str = cost_function::assign_orders(&states, &all_orders.cab_orders, &all_orders.hall_orders);
                master_transmit_tx.send(assigned_orders_str).unwrap();
            },
            recv(is_online_rx) -> is_online_msg => {
                let network_status = is_online_msg.unwrap();
                if network_status && !is_online {
                    offline_orders = [[false; 3]; config::ELEV_NUM_FLOORS as usize];
                    is_online = true;
                } else if !network_status && is_online {
                    for (order_type, order) in pending_orders.lock().unwrap().iter() {
                        if *order_type == NEW_ORDER {
                            offline_orders[order.floor as usize][order.call as usize] = true;
                        } 
                    }
                    let mut floor = 0;
                    for order in all_orders.cab_orders[elevator_id as usize].iter() {
                        offline_orders[floor as usize][e::CAB as usize] = *order;
                        floor += 1;  
                    }
                    is_online = false;
                    new_order_tx.send(offline_orders).unwrap();
                }
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
