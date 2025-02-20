#![allow(dead_code)]
use crossbeam_channel as cbc;
use crate::elevio::poll::CallButton;
use crate::elevator_controller::fsm;


fn transmitter(call_button_rx: &cbc::Receiver<CallButton>, delivered_order_rx: &cbc::Receiver<CallButton>, new_state_rx: &cbc::Receiver<fsm::State>) {
    let msg_state = (); 
        loop {
            cbc::select! {
                recv(call_button_rx) -> a => {
                    call = a.unwrap();
                    msg = ()
                    send_bcast(msg);
                },
                recv(delivered_order_rx) -> a => {
                    delivered = a.unwrap();
                    msg = ()
                    send_bcast(msg);
                },
                recv(new_state_rx) -> a => {
                    let state = a.unwrap();
                    msg_state = ();
                }
            }

            if(Some(msg_state)) {
                send_bcast(msg_state);
            }
             
        }
}


fn master_transmitter(master_activate_rx: &cbc::Receiver<()>, master_deactivate_rx: &cbc::Receiver<()>) {
    loop {
        master_activate_rx.recv().unwrap();
        loop {
            //Sende 

            if let Ok(_) = master_deactivate_rx.try_recv() {
                break;
            }
        }
    }
}