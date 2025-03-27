use crate::config::config;
use crate::cost_function::cost_function;
use crate::distributor::receiver;
use crate::distributor::transmitter;
use crate::distributor::all_orders;
use crate::distributor::udp_message;
use crate::elevator::orders;
use crate::elevator::state;
use crate::elevio::poll;
use crate::network::udp;

use crossbeam_channel as cbc;
use std::sync;
use std::thread;
use std::time;

pub fn distributor(
    elevator_id: u8,
    local_orders_tx: cbc::Sender<(orders::Orders, orders::HallOrders)>,
    completed_order_rx: cbc::Receiver<poll::CallButton>,
    new_order_rx: cbc::Receiver<poll::CallButton>,
    new_state_rx: cbc::Receiver<state::State>,
    
) {
    let mut distributor_orders = all_orders::AllOrders::init();
    let mut states = std::array::from_fn(|_| state::State::init());
    let mut last_received_heartbeat = [time::Instant::now(); config::ELEV_NUM_ELEVATORS as usize];

    let unconfirmed_orders_ticker = cbc::tick(config::UNCONFIRMED_ORDERS_TRANSMIT_PERIOD);
    let check_heartbeat_ticker = cbc::tick(config::NETWORK_TIMER_DURATION);

    
    let mut master_id = config::ELEV_NUM_ELEVATORS-1;
    let mut master_ticker = cbc::never();
    let mut master_timer = cbc::after(config::MASTER_TIMER_DURATION);


    let socket = udp::create_socket().expect("Failed to create UDP socket");
    let socket_receiver = sync::Arc::clone(&socket);
    let socket_transmitter = sync::Arc::clone(&socket);

    let (udp_message_tx, udp_message_rx) = cbc::unbounded::<udp_message::UdpMessage>();
    {
        thread::spawn(move || {
            receiver::receiver(udp_message_tx, socket_receiver)
        });
    }

    
    let (master_transmit_tx, master_transmit_rx) = cbc::unbounded::<String>();
    let (order_message_tx, order_message_rx) = cbc::unbounded::<(u8, poll::CallButton)>();
    {
        thread::spawn(move || {
            transmitter::transmitter(new_state_rx, master_transmit_rx, order_message_rx, socket_transmitter, elevator_id)
        });
    }


    loop {
        cbc::select! {
            recv(master_timer) -> _ => {
                master_id = (master_id + 1) % config::ELEV_NUM_ELEVATORS;

                if elevator_id == master_id {
                    if !states[elevator_id as usize].offline {
                        master_ticker = cbc::tick(config::MASTER_TRANSMIT_PERIOD);
                        println!("Taking over as master.");
                    }
                }

                master_timer = cbc::after(config::MASTER_TIMER_DURATION);
            },
            recv(master_ticker) -> _ => {
                if states.iter().any(|state| state.is_availible()) {
                    let all_assigned_orders_string = cost_function::assign_orders(&states, &distributor_orders.cab_orders, &distributor_orders.hall_orders);
                    master_transmit_tx.send(all_assigned_orders_string).unwrap();
                }
            },
            recv(new_order_rx) -> new_order_message => {
                let new_order = new_order_message.unwrap();
                let order_status = all_orders::NEW_ORDER;

                distributor_orders.unconfirmed_orders.push((order_status, new_order.clone()));

                if states[elevator_id as usize].offline {
                    distributor_orders.add_offline_order(new_order.clone(), elevator_id);

                    local_orders_tx.send((distributor_orders.elevator_orders, distributor_orders.hall_orders)).unwrap();
                }

                order_message_tx.send((order_status, new_order)).unwrap();
            },
            recv(completed_order_rx) -> completed_order_message => {
                let completed_order = completed_order_message.unwrap();
                let order_status = all_orders::COMPLETED_ORDER;

                distributor_orders.unconfirmed_orders.push((order_status, completed_order.clone()));

                if states[elevator_id as usize].offline {
                    distributor_orders.remove_offline_order(completed_order.clone(), elevator_id);

                    local_orders_tx.send((distributor_orders.elevator_orders, distributor_orders.hall_orders)).unwrap();
                }

                order_message_tx.send((order_status, completed_order)).unwrap();
            },
            recv(unconfirmed_orders_ticker) -> _ => {
                distributor_orders.unconfirmed_orders.iter().for_each(|(order_status, order)| {
                    order_message_tx.send((*order_status, order.clone())).unwrap();
                });
            }
            recv(udp_message_rx) -> udp_message => {
                match udp_message {
                    Ok(udp_message::UdpMessage::Order((id, order_message_array))) => {
                        let order_status = order_message_array[0];
                        let new_order = poll::CallButton{
                            floor: order_message_array[1],
                            call: order_message_array[2],
                        };
                        match order_status {
                            all_orders::NEW_ORDER => {
                                distributor_orders.add_order(new_order, id);
                            },
                            all_orders::COMPLETED_ORDER => {
                                distributor_orders.remove_order(new_order, id);
                            },
                            _ => {
                                println!("Received unexpected order type.")
                            }
                        }
                    },
                    Ok(udp_message::UdpMessage::State((id, state))) => {
                        if states[id as usize].offline {
                            states[id as usize].offline = false;
                            println!("Elevator {} has come online again", id);
                        }

                        states[id as usize] = state;
                        last_received_heartbeat[id as usize] = time::Instant::now();
                    },
                    Ok(udp_message::UdpMessage::AllAssignedOrders((incoming_master_id, all_assigned_orders))) => {
                        let prev_hashmap = distributor_orders.assigned_orders_map.clone();

                        let change_in_local_orders = distributor_orders.update_orders(all_assigned_orders, elevator_id);
                        if change_in_local_orders {
                            local_orders_tx.send((distributor_orders.elevator_orders, distributor_orders.hall_orders)).unwrap();
                        }
                        
                        distributor_orders.confirm_orders(elevator_id);

                        master_id = incoming_master_id;
                        master_timer = cbc::after(config::MASTER_TIMER_DURATION);
                        if master_id != elevator_id {
                            master_ticker = cbc::never();
                        }


                    }
                    Err(e) => {
                        println!("Received unexpected udp message in distributor::distributor and it caused this error: {:#?}.", e);
                    }
                }
            },
            recv(check_heartbeat_ticker) -> _ => {
                let now = time::Instant::now();
                
                let local_elevator_lost_connection = (now.duration_since(last_received_heartbeat[elevator_id as usize]) > config::NETWORK_TIMER_DURATION) && !states[elevator_id as usize].offline;
                if local_elevator_lost_connection {
                    states[elevator_id as usize].offline = true;
                    master_ticker = cbc::never();

                    distributor_orders.init_offline_operation(elevator_id);
                    local_orders_tx.send((distributor_orders.elevator_orders, distributor_orders.hall_orders)).unwrap();
                    
                    println!("Lost network connection - starting offline operation.");
                }

                else { 
                    for (id, last_heartbeat) in last_received_heartbeat.iter().enumerate() {
                        let elevator_lost_connection = (now.duration_since(*last_heartbeat) > config::NETWORK_TIMER_DURATION) && (!states[id].offline && !states[elevator_id as usize].offline);
                        if elevator_lost_connection {
                            println!("Elevator {} has lost network connection.", id);
                            states[id].offline = true;
                        }
                    }
                }
            }
        }
    }
}
