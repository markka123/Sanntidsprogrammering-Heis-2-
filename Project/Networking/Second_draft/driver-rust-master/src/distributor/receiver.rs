#![allow(dead_code)]
use crossbeam_channel as cbc;
use crate::elevio::poll::CallButton;


fn receiver(new_order_tx: &cbc::Sender<CallButton>, master_activate_tx: &cbc::Sender<()>) {

}

fn master_receiver(master_activate_tx: &cbc::Sender<()>, master_deactivate_tx: &cbc::Sender<()>) {
    
}

