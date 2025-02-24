#![allow(dead_code)]
use crossbeam_channel as cbc;
use crate::elevio::poll::CallButton;
use crate::elevator_controller::fsm;
use std::net::UdpSocket;
use crate::network::udp;
use std::sync::Arc;


pub fn transmitter(new_order_2_rx: cbc::Receiver<CallButton>, delivered_order_rx: cbc::Receiver<CallButton>, new_state_rx: cbc::Receiver<fsm::State>, socket: Arc<UdpSocket>) {
        // let mut bcast_state = false;
        // let mut state_init: fsm::State =  fsm::State{
        //     obstructed: false,
        //     motorstop: false,
        //     behaviour: fsm::Behaviour::Idle,
        //     floor: 0,
        //     direction: 0,
        // };
        // let mut msg_state = state_init;
        loop {
            cbc::select! {
                recv(new_order_2_rx) -> a => {
                    let call = a.unwrap();
                    // let msg_call = [0, call.floor, call.call];
                    let msg_call = "Hello World";
                    udp::broadcast_udp_message(&socket, &msg_call);
                },

                recv(delivered_order_rx) -> a => {
                    let delivered = a.unwrap();
                    let msg_delivered = [1, delivered.floor, delivered.call];
                    udp::broadcast_udp_message(&socket, &msg_delivered);
                },
                // recv(new_state_rx) -> a => {
                //     let state = a.unwrap();
                //     msg_state = state;
                //     bcast_state = true;
                // }
            }

            // if(bcast_state) {
            //     let msg_state_bytes = bincode::serialize(&msg_state).unwrap();
            //     udp::broadcast_udp_message(socket, &msg_state_bytes);
            // }
            
            
             
        }
}


pub fn master_transmitter(master_activate_transmitter_rx: cbc::Receiver<()>, master_deactivate_rx: cbc::Receiver<()>) {
    loop {
        master_activate_transmitter_rx.recv().unwrap();
        loop {
            //Sende 

            if let Ok(_) = master_deactivate_rx.try_recv() {
                break;
            }
        }
    }
}