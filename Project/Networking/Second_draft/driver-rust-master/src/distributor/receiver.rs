#![allow(dead_code)]
use crossbeam_channel as cbc;
use crate::elevio::poll::CallButton;
use crate::elevator_controller::orders;


pub fn receiver(new_order_tx: &cbc::Sender<orders::Orders>, master_activate_tx: &cbc::Sender<()>) {

}

pub fn master_receiver(master_activate_rx: cbc::Receiver<()>, master_deactivate_tx: cbc::Sender<()>) {
    
}

