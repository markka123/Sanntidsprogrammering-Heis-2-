use crate::config::config;
use crate::cost_function::cost_function;
use crate::distributor::receiver;
use crate::distributor::transmitter;
use crate::distributor::all_orders;
use crate::distributor::udp_message;
use crate::elevator::orders;
use crate::elevator::state;
use crate::elevio::elev;
use crate::elevio::poll;
use crate::network::udp;

use crossbeam_channel as cbc;
use std::sync;
use std::thread::spawn;
use std::time;

pub fn distributor(
    elevator_id: u8,
    elevator_orders_tx: cbc::Sender<(orders::Orders, orders::HallOrders)>,
    order_completed_rx: cbc::Receiver<poll::CallButton>,
    order_new_rx: cbc::Receiver<poll::CallButton>,
    new_state_rx: cbc::Receiver<state::State>,
    
) {
    let mut distributor_orders = all_orders::AllOrders::init();
    let mut states: state::States = std::array::from_fn(|_| state::State::init());

    let mut last_received_heartbeat = [time::Instant::now(); config::ELEV_NUM_ELEVATORS as usize];

    let unconfirmed_orders_ticker = cbc::tick(config::UNCONFIRMED_ORDERS_TRANSMIT_PERIOD);
    let mut master_ticker = cbc::never();
    let check_heartbeat_ticker = cbc::tick(config::NETWORK_TIMER_DURATION);

    let socket = udp::create_udp_socket().expect("Failed to create UDP socket");
    let socket_receiver = sync::Arc::clone(&socket);
    let socket_transmitter = sync::Arc::clone(&socket);

    let (message_tx, message_rx) = cbc::unbounded::<udp_message::UdpMessage>();
    let (master_transmit_tx, master_transmit_rx) = cbc::unbounded::<String>();
    let (master_activate_tx, master_activate_rx) = cbc::unbounded::<bool>();
    let (call_message_tx, call_message_rx) = cbc::unbounded::<(u8, poll::CallButton)>();
    {
        spawn(move || {
            receiver::receiver(message_tx, master_activate_tx, socket_receiver, elevator_id)
        });
    }

    {
        spawn(move || {
            transmitter::transmitter(elevator_id, new_state_rx, master_transmit_rx, call_message_rx, socket_transmitter)
        });
    }

    loop {
        cbc::select! {
            recv(order_new_rx) -> order_new => {
                let order_new = order_new.unwrap();
                let message_type = all_orders::NEW_ORDER;

                distributor_orders.unconfirmed_orders.push((message_type, order_new.clone()));

                if states[elevator_id as usize].offline {
                    distributor_orders.elevator_orders[order_new.floor as usize][order_new.call as usize] = true;
                    distributor_orders.add_order(order_new.clone(), elevator_id);
                    elevator_orders_tx.send((distributor_orders.elevator_orders, distributor_orders.hall_orders)).unwrap();
                }

                call_message_tx.send((message_type, order_new)).unwrap();
            },
            recv(order_completed_rx) -> order_completed => {
                let order_completed = order_completed.unwrap();
                let message_type = all_orders::COMPLETED_ORDER;

                distributor_orders.unconfirmed_orders.push((message_type, order_completed.clone()));

                if states[elevator_id as usize].offline {
                    distributor_orders.elevator_orders[order_completed.floor as usize][order_completed.call as usize] = false;
                    distributor_orders.remove_order(order_completed.clone(), elevator_id);
                    distributor_orders.confirm_offline_order(order_completed.clone());
                    elevator_orders_tx.send((distributor_orders.elevator_orders, distributor_orders.hall_orders)).unwrap();
                }
                call_message_tx.send((message_type, order_completed)).unwrap();
            },
            recv(unconfirmed_orders_ticker) -> _ => {
                distributor_orders.unconfirmed_orders.iter().for_each(|(message_type, call)| {
                    call_message_tx.send((*message_type, call.clone())).unwrap();

                });
            }
            recv(message_rx) -> udp_message => {
                match udp_message {
                    Ok(udp_message::UdpMessage::Order((id, message_array))) => {
                        let message_type = message_array[0];
                        let new_order = poll::CallButton{
                            floor: message_array[1],
                            call: message_array[2],
                        };
                        match message_type {
                            all_orders::NEW_ORDER => {
                                distributor_orders.add_order(new_order, id);
                            },
                            all_orders::COMPLETED_ORDER => {
                   
                                distributor_orders.remove_order(new_order, id);
                            },
                            _ => {
                            }
                        }
                    },
                    Ok(udp_message::UdpMessage::State((id, state))) => {

                        if states[id as usize].offline {
                            println!("Elevator {} has come online again", id);
                            states[id as usize].offline = false;
                        }

                        states[id as usize] = state;
                        last_received_heartbeat[id as usize] = time::Instant::now();
                    },
                    Ok(udp_message::UdpMessage::AllAssignedOrders((_, all_assigned_orders_string))) => {
                        let previous_hall_orders = distributor_orders.get_assigned_hall_orders();
                        let previous_elevator_orders = distributor_orders.elevator_orders;
                        
                        distributor_orders.assigned_orders_map = serde_json::from_value(all_assigned_orders_string).unwrap();
                        distributor_orders.hall_orders = distributor_orders.get_assigned_hall_orders();

                        if let Some(new_elevator_orders) = distributor_orders.assigned_orders_map.get(&elevator_id) {
                            distributor_orders.elevator_orders = *new_elevator_orders;
                        }

                        let change_in_orders = distributor_orders.hall_orders != previous_hall_orders || distributor_orders.elevator_orders != previous_elevator_orders;
                        if change_in_orders {
                            elevator_orders_tx.send((distributor_orders.elevator_orders, distributor_orders.hall_orders)).unwrap();
                        }

                        distributor_orders.confirm_orders(elevator_id);
                    }
                    Err(e) => {
                        
                    }
                }
            },
            recv(master_activate_rx) -> _ => {
                if !states[elevator_id as usize].offline {
                    println!("Taking over as master.");
                    master_ticker = cbc::tick(config::MASTER_TRANSMIT_PERIOD);
                }
            },
            recv(master_ticker) -> _ => {
                //println!("state id 0: {:#?}", states[0].behaviour);
                if states.iter().any(|state| state.is_availible()) {
                    let assigned_orders_string = cost_function::assign_orders(&states, &distributor_orders.cab_orders, &distributor_orders.hall_orders);
                    master_transmit_tx.send(assigned_orders_string).unwrap();
                }
            },
            recv(check_heartbeat_ticker) -> _ => {
                let now = time::Instant::now();
                if (now.duration_since(last_received_heartbeat[elevator_id as usize]) > config::NETWORK_TIMER_DURATION) && !states[elevator_id as usize].offline {
                    states[elevator_id as usize].offline = true;
                    distributor_orders.init_offline_operation(elevator_id);
                    master_ticker = cbc::never();
                    elevator_orders_tx.send((distributor_orders.elevator_orders, distributor_orders.hall_orders)).unwrap();
                    println!("Lost network connection - starting offline operation");
                }
                for (id, last_heartbeat) in last_received_heartbeat.iter().enumerate() {
                    if (now.duration_since(*last_heartbeat) > config::NETWORK_TIMER_DURATION) && (!states[id].offline && !states[elevator_id as usize].offline) {
                        println!("Elevator {} has gone offline", id);
                        states[id].offline = true;
                    }
                }
            }
        }
    }
}
