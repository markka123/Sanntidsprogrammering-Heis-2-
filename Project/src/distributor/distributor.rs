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

    let mut all_orders = orders::AllOrders::init();
    let mut offline_orders: orders::Orders = [[false; 3]; config::ELEV_NUM_FLOORS as usize];
    let unconfirmed_orders: Arc<Mutex<Vec<(u8, poll::CallButton)>>> = Arc::new(Mutex::new(Vec::new()));
    let mut assigned_orders = [[false; 3]; config::ELEV_NUM_FLOORS as usize];
    let mut all_hall_orders = [[false; 2]; config::ELEV_NUM_FLOORS as usize];

    let mut master_id = config::ELEV_NUM_ELEVATORS;
    let mut is_online = true;
    let mut last_received_heartbeat = [Instant::now(); config::ELEV_NUM_ELEVATORS as usize];
    
    let mut master_ticker = cbc::never();
    let check_slaves_heartbeat_ticker = cbc::tick(config::NETWORK_TIMER_DURATION);


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
            is_online_tx, 
            socket_receiver,
            elevator_id
        ));
    }

    let unconfirmed_orders_clone = Arc::clone(&unconfirmed_orders);
    {
        spawn(move || {
            transmitter::transmitter(
                elevator_id,             
                new_state_rx,
                master_transmit_rx,
                call_msg_rx,
                unconfirmed_orders_clone,
                socket_transmitter,
            )
        });
    }
    
    loop {
        cbc::select! {
            recv(call_button_rx) -> call_button => {
                let call_button = call_button.unwrap();
                let msg_type = NEW_ORDER;
                
                if !is_online {
                    offline_orders[call_button.floor as usize][call_button.call as usize] = true;
                    new_order_tx.send((offline_orders, all_orders.hall_orders)).unwrap();
                }
                call_msg_tx.send((msg_type, call_button)).unwrap();
            },
            recv(order_completed_rx) -> order_completed => {
                let order_completed = order_completed.unwrap();
                let msg_type = COMPLETED_ORDER;
                
                if !is_online {
                    offline_orders[order_completed.floor as usize][order_completed.call as usize] = false;
                    unconfirmed_orders.lock().unwrap().retain(|(msg, order)| *msg != msg_type || order.floor != order_completed.floor || order.call != order_completed.call);
                    new_order_tx.send((offline_orders, all_orders.hall_orders)).unwrap(); 
                }
                call_msg_tx.send((msg_type, order_completed)).unwrap();
            },
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
                        
                        if states[id as usize].offline {
                            println!("Elevator {} has come online again", id);
                            states[id as usize].offline = false;
                        }
                        
                        states[id as usize] = state;
                        last_received_heartbeat[id as usize] = Instant::now();
                    },
                    Ok(Message::AllAssignedOrdersMsg((master_id, all_assigned_orders_str))) => {

                        let all_assigned_orders_map: HashMap<u8, orders::Orders> = serde_json::from_value(all_assigned_orders_str).unwrap();

                        let new_all_hall_orders = get_all_hall_orders(&all_assigned_orders_map);

                        let elevator_is_availible = states[elevator_id as usize].motorstop || states[elevator_id as usize].emergency_stop || states[elevator_id as usize].obstructed || states[elevator_id as usize].offline;

                        if !elevator_is_availible {
                            if let Some(new_assigned_orders) = all_assigned_orders_map.get(&elevator_id) {
                                if (*new_assigned_orders != assigned_orders)  || (new_all_hall_orders != all_hall_orders) {
                                    assigned_orders = *new_assigned_orders;
                                    all_hall_orders = new_all_hall_orders;
                                    new_order_tx.send((assigned_orders, all_hall_orders)).unwrap();
                                }
                            } 
                        }
                        confirm_orders(&unconfirmed_orders, &all_assigned_orders_map, elevator_id);                        
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
                let assigned_orders_str = cost_function::assign_orders(&states, &all_orders.cab_orders, &all_orders.hall_orders);
                master_transmit_tx.send(assigned_orders_str).unwrap();
            },
            recv(check_slaves_heartbeat_ticker) -> _ => {
                let now = Instant::now();
                if states[elevator_id].online {

                }
                for (id, last_heartbeat) in last_received_heartbeat.iter().enumerate() {
                    if now.duration_since(*last_heartbeat) > config::NETWORK_TIMER_DURATION {
                        if !states[id].offline {
                            println!("Elevator {} has gone offline", id);
                            states[id].offline = true; // change to one "disconnected/unavailable" state?
                        }
                    }
                }
            },
            recv(is_online_rx) -> is_online_msg => {
                let network_status = is_online_msg.unwrap();
                if network_status && !is_online {
                    offline_orders = [[false; 3]; config::ELEV_NUM_FLOORS as usize];
                    is_online = true;
                } else if !network_status && is_online {
                    for (order_type, order) in unconfirmed_orders.lock().unwrap().iter() {
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
                    new_order_tx.send((offline_orders, all_orders.hall_orders)).unwrap();
                }
            }
        }
    }
}

fn get_all_hall_orders(map: &HashMap<u8, orders::Orders>) -> orders::HallOrders {
    let mut all_hall_orders = [[false; 2]; config::ELEV_NUM_FLOORS as usize];

    for orders in map.values() {
        for (floor, call) in orders.iter().enumerate() {
            all_hall_orders[floor][0] |= call[0];
            all_hall_orders[floor][1] |= call[1];
        }
    }
    
    all_hall_orders
}

// TODO: Where should this be placed? (orders.rs?)

fn confirm_orders(
    unconfirmed_orders: &Arc<Mutex<Vec<(u8, poll::CallButton)>>>,
    all_assigned_orders_map: &HashMap<u8, [[bool; 3]; config::ELEV_NUM_FLOORS as usize]>, 
    elevator_id: u8,
) {
    unconfirmed_orders.lock().unwrap().retain(|(order_type, order)| {
        let order_is_assigned = |floor: usize, call: usize| 
            all_assigned_orders_map.iter().any(|(_, assigned_orders)| assigned_orders[floor][call]);

        let order_is_unassigned = |floor: usize, call: usize| 
            all_assigned_orders_map.iter().all(|(_, assigned_orders)| !assigned_orders[floor][call]);

        match order.call {
            e::HALL_UP | e::HALL_DOWN => match order_type {
                &NEW_ORDER => !order_is_assigned(order.floor as usize, order.call as usize),
                &COMPLETED_ORDER => order_is_unassigned(order.floor as usize, order.call as usize),
                _ => true,
            },
            e::CAB => {
                if let Some(assigned_orders) = all_assigned_orders_map.get(&elevator_id) {
                    let cab_is_assigned = assigned_orders[order.floor as usize][order.call as usize];
                    return !((cab_is_assigned && *order_type == NEW_ORDER) || (!cab_is_assigned && *order_type == COMPLETED_ORDER));
                }
                true
            }
            _ => true,
        }
    });
}