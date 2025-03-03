#![allow(dead_code)]
use crate::config::config;
use crate::distributor::receiver;
use crate::distributor::transmitter;
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    CallMsg([u8; 3]),
    StateMsg((u8, State)),
    // AssignedOrders([Orders; config::ELEV_NUM_ELEVATORS as usize]),
    // HallOrders()
}

pub const NEW_ORDER: u8 = 0;
pub const COMPLETED_ORDER: u8 = 1;

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
    
    let master_ip = config::BROADCAST_IP;

    let (message_tx, message_rx) = cbc::unbounded::<Message>();
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
            socket_receiver
        ));
    }
    {
        spawn(move || {
            transmitter::transmitter(
                elevator_id,
                call_button_rx,
                new_state_rx,
                order_completed_rx,
                socket_transmitter,
                &master_ip,
            )
        });
    }

    let mut all_orders = AllOrders::init();
    let mut master_ticker = cbc::never();

    loop {
        lights::set_lights(&all_orders, elevator.clone());
        // sleep(Duration::from_millis(100));
        select! {
            recv(message_rx) -> message => {
                match message {
                    Ok(Message::CallMsg(msg_array)) => {
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
                    Ok(Message::StateMsg(state_msg)) => {
                        let (id, state) = state_msg;
                        // states[id] = state;
                        println!("Id: {}", id);
                        println!{"{:#?}", state}; 
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
                // call cost func
                // bcast results
            }
        }
    }
}