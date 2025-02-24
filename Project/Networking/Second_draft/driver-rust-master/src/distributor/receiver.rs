#![allow(dead_code)]
use crossbeam_channel as cbc;
use std::net::UdpSocket;
use crate::elevio::poll::CallButton;
use crate::elevator_controller::orders;
use crate::network::udp::*;
use std::sync::Arc;


pub fn receiver(new_order_tx: cbc::Sender<orders::Orders>, master_activate_transmitter_tx: cbc::Sender<()>, master_activate_receiver_tx: cbc::Sender<()>, socket: Arc<UdpSocket>) {
    loop {
        match receive_udp_message::<String>(&socket) {
            Some((message, string)) => {
                // Process the received message
                println!("Received: {:?}", message);
            }
            None => {
                // println!("No message received");
            }
        }
      
        }
}

pub fn master_receiver(master_activate_receiver_rx: cbc::Receiver<()>, master_deactivate_tx: cbc::Sender<()>) {
    
}

