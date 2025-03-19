// TODO: FIX IMPORTS
#![allow(dead_code)]
use crate::config::config;
use crate::distributor::receiver;
use crate::distributor::transmitter;
use crate::elevator_controller::state;
use crate::elevator_controller::orders;
use crate::elevio::elev as e;
use crate::elevio::poll;
use crate::network::udp;
use crate::cost_function::cost_function;

use crossbeam_channel as cbc;
use std::sync::Arc;
use std::thread::*;
use std::time::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;


// TODO: FIND OUT WHERE THESE SHOULD BE:
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    CallMsg((u8, [u8; 3])),
    StateMsg((u8, state::State)),
    AllAssignedOrdersMsg((u8, Value)),
}

pub const NEW_ORDER: u8 = 0;
pub const COMPLETED_ORDER: u8 = 1;


pub fn distributor(
    elevator: &e::Elevator,
    elevator_id: u8,
    new_state_rx: cbc::Receiver<state::State>,
    order_completed_rx: cbc::Receiver<poll::CallButton>,
    new_order_tx: cbc::Sender<(orders::Orders, orders::HallOrders)>,
) {
    //TODO: REVIEW IF INITS CAN BE CLEANED UP: (Init function?)

    let mut distributor_orders = orders::DistributorOrders::init(); 

    let mut master_id = config::ELEV_NUM_ELEVATORS;
    let mut last_received_heartbeat = [Instant::now(); config::ELEV_NUM_ELEVATORS as usize];
    
    let unconfirmed_orders_ticker = cbc::tick(config::UNCONFIRMED_ORDERS_TRANSMIT_PERIOD);
    let mut master_ticker = cbc::never();
    let check_heartbeat_ticker = cbc::tick(config::NETWORK_TIMER_DURATION);


    let socket = udp::create_udp_socket().expect("Failed to create UDP socket");
    let socket_receiver = Arc::clone(&socket);
    let socket_transmitter = Arc::clone(&socket);
    
    let mut states: state::States = std::array::from_fn(|_| state::State {
        obstructed: false,
        motorstop: false,
        offline: false,
        emergency_stop: false,
        behaviour: state::Behaviour::Idle,
        floor: 0,
        direction: e::HALL_DOWN,
    });

    let (message_tx, message_rx) = cbc::unbounded::<Message>();
    let (is_online_tx, is_online_rx) = cbc::unbounded::<bool>();
    let (master_transmit_tx, master_transmit_rx) = cbc::unbounded::<String>();
    let (master_activate_tx, master_activate_rx) = cbc::unbounded::<bool>();
    let (call_button_tx, call_button_rx) = cbc::unbounded::<poll::CallButton>();
    let (call_msg_tx, call_msg_rx) = cbc::unbounded::<(u8, poll::CallButton)>();

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
                new_state_rx,
                master_transmit_rx,
                call_msg_rx,
                socket_transmitter,
            )
        });
    }
    
    loop {
        cbc::select! {
            recv(call_button_rx) -> call_button => {
                let call_button = call_button.unwrap();
                let msg_type = NEW_ORDER;

                distributor_orders.unconfirmed_orders.push((msg_type, call_button.clone()));
                
                if states[elevator_id as usize].offline {
                    distributor_orders.elevator_orders[call_button.floor as usize][call_button.call as usize] = true;
                    distributor_orders.add_order(call_button.clone(), elevator_id);
                    new_order_tx.send((distributor_orders.elevator_orders, distributor_orders.hall_orders)).unwrap();
                }
                call_msg_tx.send((msg_type, call_button)).unwrap();
            },
            recv(order_completed_rx) -> order_completed => {
                let order_completed = order_completed.unwrap();
                let msg_type = COMPLETED_ORDER;
                
                if states[elevator_id as usize].offline {
                    distributor_orders.elevator_orders[order_completed.floor as usize][order_completed.call as usize] = false;
                    distributor_orders.remove_order(order_completed.clone(), elevator_id);
                    distributor_orders.unconfirmed_orders.retain(|(msg, order)| *msg != msg_type || order.floor != order_completed.floor || order.call != order_completed.call);
                    new_order_tx.send((distributor_orders.elevator_orders, distributor_orders.hall_orders)).unwrap(); 
                }
                call_msg_tx.send((msg_type, order_completed)).unwrap();
            },
            recv(unconfirmed_orders_ticker) -> _ => {
                distributor_orders.unconfirmed_orders.iter().for_each(|(msg_type, call)| {
                    call_msg_tx.send((*msg_type, call.clone())).unwrap();
                });
            }
            recv(message_rx) -> message => {
                match message {
                    Ok(Message::CallMsg(call_msg)) => {
                        let (id, msg_array) = call_msg;
                        let msg_type = msg_array[0];
                        let new_order = poll::CallButton{
                            floor: msg_array[1],
                            call: msg_array[2],
                        };
                        match msg_type {
                            NEW_ORDER => {
                                distributor_orders.add_order(new_order, id);
                            },
                            COMPLETED_ORDER => {
                                distributor_orders.remove_order(new_order, id);
                            },
                            _ => {
                                //Handle error
                            }
                        }
                    },
                    Ok(Message::StateMsg(state_msg)) => {
                        let (id, state) = state_msg;
                        
                        if states[id as usize].offline {
                            println!("Elevator {} has come online again", id);
                            states[id as usize].offline = false;
                        }
                        
                        states[id as usize] = state;
                        last_received_heartbeat[id as usize] = Instant::now();
                    },
                    Ok(Message::AllAssignedOrdersMsg((master_id, all_assigned_orders_str))) => {
                        let previous_hall_orders = distributor_orders.get_assigned_hall_orders();
                        distributor_orders.assigned_orders_map = serde_json::from_value(all_assigned_orders_str).unwrap();
                        let new_hall_orders = distributor_orders.get_assigned_hall_orders();

                        let elevator_is_availible = states[elevator_id as usize].motorstop || states[elevator_id as usize].emergency_stop || states[elevator_id as usize].obstructed || states[elevator_id as usize].offline;

                        if !elevator_is_availible {
                            if let Some(new_elevator_orders) = distributor_orders.assigned_orders_map.get(&elevator_id) {
                                if (*new_elevator_orders != distributor_orders.elevator_orders)  || (new_hall_orders != previous_hall_orders) {
                                    distributor_orders.elevator_orders = *new_elevator_orders;
                                    distributor_orders.hall_orders = new_hall_orders;
                                    new_order_tx.send((distributor_orders.elevator_orders, distributor_orders.hall_orders)).unwrap();
                                }
                            } 
                        }
                        distributor_orders.confirm_orders(elevator_id);                        
                    }
                    Err(e) => {
                    }
                }
            },
            recv(master_activate_rx) -> _ => {
                master_ticker = cbc::tick(config::MASTER_TRANSMIT_PERIOD);
                // TODO: Review if we should merge motorstop obstruction and emergency stop into a "disconnected/unavailible" variable
                if (master_id as usize) < config::ELEV_NUM_ELEVATORS as usize { 
                    states[master_id as usize].offline = true; 
                }
                master_id = elevator_id;
            },
            recv(master_ticker) -> _ => {
                let assigned_orders_str = cost_function::assign_orders(&states, &distributor_orders.cab_orders, &distributor_orders.hall_orders);
                master_transmit_tx.send(assigned_orders_str).unwrap();
            },
            recv(check_heartbeat_ticker) -> _ => {
                let now = Instant::now();
                for (id, last_heartbeat) in last_received_heartbeat.iter().enumerate() {
                    if now.duration_since(*last_heartbeat) > config::NETWORK_TIMER_DURATION {
                        if !states[id].offline {
                            println!("Elevator {} has gone offline", id);
                            states[id].offline = true; // change to one "disconnected/unavailable" state?
                        }
                        if id == (elevator_id as usize) {
                            distributor_orders.init_offline_operation(elevator_id);
                            new_order_tx.send((distributor_orders.elevator_orders, distributor_orders.hall_orders)).unwrap();
                            master_ticker = cbc::never();
                        }
                    }
                }
            }
        }
    }
}
