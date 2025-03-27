use crate::config::config;
use crate::elevio::elev;

use crossbeam_channel as cbc;


pub fn doors(
    elevator: elev::Elevator,
    open_doors_rx: cbc::Receiver<bool>,
    close_doors_tx: cbc::Sender<bool>,
    obstruct_doors_rx: cbc::Receiver<bool>
) {

    elevator.door_light(false);

    let mut doors_obstructed = false;
    let mut doors_open = false;
    let mut door_timer = cbc::never();

    loop {
        cbc::select! {
            recv(open_doors_rx) -> _ => {
                elevator.door_light(true);

                if doors_obstructed {
                    door_timer = cbc::never();
                    println!("Doors are obstructed.");
                }
                else {
                    door_timer = cbc::after(config::DOOR_TIMER_DURATION);
                }

                doors_open = true;
            },
            recv(obstruct_doors_rx) -> obstruction_message => {
                doors_obstructed = obstruction_message.unwrap();

                if doors_obstructed && doors_open {
                    door_timer = cbc::never();
                    println!("Doors are obstructed.");
                    
                } else if doors_open {
                    door_timer = cbc::after(config::DOOR_TIMER_DURATION);
                    println!("Doors are no longer obstructed.");
                }
            },

            recv(door_timer) -> _ => {
                doors_open = false;
                elevator.door_light(false);
                
                close_doors_tx.send(true).unwrap();
            }
        }
    }
}